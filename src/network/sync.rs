use crate::network::{NetworkError, NetworkConfig};
use crate::network::peer::{PeerManager, Peer};
use crate::network::protocol::{Message, MessageType, MessagePayload, BlockHeader};
use crate::blockchain::{Blockchain, BlockchainError};
use crate::block::Block;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::{RwLock, Mutex};
use tokio::time::{Duration, timeout, sleep, Instant};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::Mutex as ParkingMutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SyncState {
    Syncing,
    Synced,
    Stalled,
    Error(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyncProgress {
    pub current_height: u64,
    pub target_height: u64,
    pub progress_percentage: f64,
    pub blocks_per_second: f64,
    pub eta_seconds: Option<u64>,
    pub active_downloads: usize,
}

#[derive(Clone, Debug)]
pub struct BlockRequest {
    pub height: u64,
    pub hash: String,
    pub peer: SocketAddr,
    pub requested_at: Instant,
    pub retry_count: u32,
}

#[derive(Clone, Debug)]
pub struct SyncPeer {
    pub address: SocketAddr,
    pub best_height: u64,
    pub best_hash: String,
    pub is_syncing: bool,
    pub last_response: Instant,
    pub requests_sent: u32,
    pub blocks_received: u32,
    pub sync_score: f64,
}

impl SyncPeer {
    pub fn new(address: SocketAddr, best_height: u64, best_hash: String) -> Self {
        Self {
            address,
            best_height,
            best_hash,
            is_syncing: false,
            last_response: Instant::now(),
            requests_sent: 0,
            blocks_received: 0,
            sync_score: 1.0,
        }
    }

    pub fn update_score(&mut self) {
        let response_ratio = if self.requests_sent == 0 {
            1.0
        } else {
            self.blocks_received as f64 / self.requests_sent as f64
        };
        
        let time_penalty = if self.last_response.elapsed() > Duration::from_secs(60) {
            0.5
        } else {
            1.0
        };
        
        self.sync_score = response_ratio * time_penalty;
    }
}

pub struct BlockchainSync {
    blockchain: Arc<Mutex<Blockchain>>,
    peer_manager: Arc<PeerManager>,
    config: NetworkConfig,
    
    // Sync state
    sync_state: Arc<RwLock<SyncState>>,
    sync_progress: Arc<RwLock<SyncProgress>>,
    
    // Peer tracking
    sync_peers: Arc<DashMap<SocketAddr, SyncPeer>>,
    
    // Request management
    pending_requests: Arc<DashMap<u64, BlockRequest>>, // height -> request
    block_buffer: Arc<DashMap<u64, Block>>, // height -> block
    
    // Sync parameters
    max_concurrent_requests: usize,
    request_timeout: Duration,
    max_retries: u32,
    batch_size: usize,
    
    // Statistics
    sync_start_time: Arc<ParkingMutex<Option<Instant>>>,
    blocks_synced: Arc<ParkingMutex<u64>>,
}

impl BlockchainSync {
    pub fn new(
        blockchain: Arc<Mutex<Blockchain>>,
        peer_manager: Arc<PeerManager>,
        config: NetworkConfig,
    ) -> Self {
        Self {
            blockchain,
            peer_manager,
            config,
            sync_state: Arc::new(RwLock::new(SyncState::Synced)),
            sync_progress: Arc::new(RwLock::new(SyncProgress {
                current_height: 0,
                target_height: 0,
                progress_percentage: 100.0,
                blocks_per_second: 0.0,
                eta_seconds: None,
                active_downloads: 0,
            })),
            sync_peers: Arc::new(DashMap::new()),
            pending_requests: Arc::new(DashMap::new()),
            block_buffer: Arc::new(DashMap::new()),
            max_concurrent_requests: 50,
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
            batch_size: 10,
            sync_start_time: Arc::new(ParkingMutex::new(None)),
            blocks_synced: Arc::new(ParkingMutex::new(0)),
        }
    }

    pub async fn start_sync(&self) -> Result<(), NetworkError> {
        println!("Starting blockchain synchronization...");
        
        // Reset sync state
        {
            let mut state = self.sync_state.write().await;
            *state = SyncState::Syncing;
        }
        
        {
            let mut start_time = self.sync_start_time.lock();
            *start_time = Some(Instant::now());
        }
        
        // Start sync process
        let sync_clone = self.clone_sync_data();
        tokio::spawn(async move {
            Self::run_sync_loop(sync_clone).await;
        });
        
        Ok(())
    }

    async fn run_sync_loop(data: SyncData) {
        let mut sync_interval = tokio::time::interval(Duration::from_secs(5));
        
        loop {
            sync_interval.tick().await;
            
            if let Err(e) = Self::sync_iteration(&data).await {
                println!("Sync iteration failed: {}", e);
                {
                    let mut state = data.sync_state.write().await;
                    *state = SyncState::Error(e.to_string());
                }
                sleep(Duration::from_secs(10)).await;
            }
            
            // Check if we're synced
            if Self::is_synced(&data).await {
                {
                    let mut state = data.sync_state.write().await;
                    *state = SyncState::Synced;
                }
                sleep(Duration::from_secs(30)).await; // Check less frequently when synced
            }
        }
    }

    async fn sync_iteration(data: &SyncData) -> Result<(), NetworkError> {
        // Update peer information
        Self::update_sync_peers(data).await?;
        
        // Get current blockchain height
        let current_height = {
            let blockchain = data.blockchain.lock().await;
            blockchain.get_chain_height()
        };
        
        // Find the best height among peers
        let target_height = Self::get_target_height(data).await;
        
        if target_height <= current_height {
            return Ok(()); // We're synced
        }
        
        // Update progress
        Self::update_sync_progress(data, current_height, target_height).await;
        
        // Process received blocks
        Self::process_block_buffer(data).await?;
        
        // Request missing blocks
        Self::request_missing_blocks(data, current_height, target_height).await?;
        
        // Cleanup expired requests
        Self::cleanup_expired_requests(data).await;
        
        Ok(())
    }

    async fn update_sync_peers(data: &SyncData) -> Result<(), NetworkError> {
        let connected_peers = data.peer_manager.get_connected_peers().await;
        
        for peer in connected_peers {
            let peer_info = peer.info.read().await;
            let address = peer_info.address;
            let best_height = peer_info.best_height;
            
            if best_height > 0 {
                if let Some(mut sync_peer) = data.sync_peers.get_mut(&address) {
                    sync_peer.best_height = best_height;
                    sync_peer.update_score();
                } else {
                    let sync_peer = SyncPeer::new(address, best_height, String::new());
                    data.sync_peers.insert(address, sync_peer);
                }
            }
        }
        
        // Remove disconnected peers
        let connected_addresses: std::collections::HashSet<_> = 
            data.peer_manager.get_connected_peers().await
                .iter()
                .map(|p| async { p.info.read().await.address })
                .collect::<futures::future::FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect();
        
        data.sync_peers.retain(|addr, _| connected_addresses.contains(addr));
        
        Ok(())
    }

    async fn get_target_height(data: &SyncData) -> u64 {
        data.sync_peers
            .iter()
            .map(|entry| entry.value().best_height)
            .max()
            .unwrap_or(0)
    }

    async fn update_sync_progress(data: &SyncData, current: u64, target: u64) {
        let progress_percentage = if target == 0 {
            100.0
        } else {
            (current as f64 / target as f64) * 100.0
        };
        
        let blocks_per_second = Self::calculate_sync_speed(data).await;
        
        let eta_seconds = if blocks_per_second > 0.0 && current < target {
            Some(((target - current) as f64 / blocks_per_second) as u64)
        } else {
            None
        };
        
        let active_downloads = data.pending_requests.len();
        
        {
            let mut progress = data.sync_progress.write().await;
            progress.current_height = current;
            progress.target_height = target;
            progress.progress_percentage = progress_percentage;
            progress.blocks_per_second = blocks_per_second;
            progress.eta_seconds = eta_seconds;
            progress.active_downloads = active_downloads;
        }
    }

    async fn calculate_sync_speed(data: &SyncData) -> f64 {
        let start_time = data.sync_start_time.lock();
        let blocks_synced = *data.blocks_synced.lock();
        
        if let Some(start) = *start_time {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                blocks_synced as f64 / elapsed
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    async fn process_block_buffer(data: &SyncData) -> Result<(), NetworkError> {
        let blockchain = data.blockchain.lock().await;
        let current_height = blockchain.get_chain_height();
        drop(blockchain);
        
        let next_height = current_height + 1;
        
        // Process blocks in order
        while let Some((_, block)) = data.block_buffer.remove(&next_height) {
            match Self::add_block_to_chain(data, block).await {
                Ok(_) => {
                    {
                        let mut blocks_synced = data.blocks_synced.lock();
                        *blocks_synced += 1;
                    }
                    println!("Added block at height {}", next_height);
                }
                Err(e) => {
                    println!("Failed to add block at height {}: {}", next_height, e);
                    return Err(NetworkError::SyncFailed(e.to_string()));
                }
            }
        }
        
        Ok(())
    }

    async fn add_block_to_chain(data: &SyncData, block: Block) -> Result<(), BlockchainError> {
        let mut blockchain = data.blockchain.lock().await;
        
        // Validate the block
        blockchain.validate_block(&block)?;
        
        // Add to chain
        blockchain.chain.push(block);
        
        Ok(())
    }

    async fn request_missing_blocks(
        data: &SyncData,
        current_height: u64,
        target_height: u64,
    ) -> Result<(), NetworkError> {
        let active_requests = data.pending_requests.len();
        
        if active_requests >= data.max_concurrent_requests {
            return Ok(()); // Already at max concurrent requests
        }
        
        let mut requests_to_make = data.max_concurrent_requests - active_requests;
        let mut height = current_height + 1;
        
        while requests_to_make > 0 && height <= target_height {
            // Skip if already requested or in buffer
            if data.pending_requests.contains_key(&height) || data.block_buffer.contains_key(&height) {
                height += 1;
                continue;
            }
            
            // Find best peer for this request
            if let Some(peer_addr) = Self::select_sync_peer(data, height).await {
                let request = BlockRequest {
                    height,
                    hash: String::new(), // Will be filled when we have header
                    peer: peer_addr,
                    requested_at: Instant::now(),
                    retry_count: 0,
                };
                
                data.pending_requests.insert(height, request);
                
                // Send block request
                if let Err(e) = Self::send_block_request(data, peer_addr, height).await {
                    println!("Failed to send block request: {}", e);
                    data.pending_requests.remove(&height);
                }
                
                requests_to_make -= 1;
            }
            
            height += 1;
        }
        
        Ok(())
    }

    async fn select_sync_peer(data: &SyncData, height: u64) -> Option<SocketAddr> {
        let mut best_peer: Option<(SocketAddr, f64)> = None;
        
        for entry in data.sync_peers.iter() {
            let sync_peer = entry.value();
            
            // Peer must have the block we need
            if sync_peer.best_height < height {
                continue;
            }
            
            // Prefer peers that aren't busy
            let busy_penalty = if sync_peer.is_syncing { 0.5 } else { 1.0 };
            let score = sync_peer.sync_score * busy_penalty;
            
            match best_peer {
                None => best_peer = Some((sync_peer.address, score)),
                Some((_, best_score)) if score > best_score => {
                    best_peer = Some((sync_peer.address, score));
                }
                _ => {}
            }
        }
        
        best_peer.map(|(addr, _)| addr)
    }

    async fn send_block_request(
        data: &SyncData,
        peer_addr: SocketAddr,
        height: u64,
    ) -> Result<(), NetworkError> {
        let message = Message::new(
            MessageType::GetBlockData,
            SocketAddr::from(([0, 0, 0, 0], 0)), // Our address (placeholder)
            MessagePayload::Empty, // Would contain height/hash in real implementation
        );
        
        data.peer_manager.send_to_peer(&peer_addr, message).await?;
        
        // Update peer sync state
        if let Some(mut sync_peer) = data.sync_peers.get_mut(&peer_addr) {
            sync_peer.is_syncing = true;
            sync_peer.requests_sent += 1;
        }
        
        Ok(())
    }

    async fn cleanup_expired_requests(data: &SyncData) {
        let now = Instant::now();
        let mut expired_requests = Vec::new();
        
        for entry in data.pending_requests.iter() {
            let request = entry.value();
            if now.duration_since(request.requested_at) > data.request_timeout {
                expired_requests.push(*entry.key());
            }
        }
        
        for height in expired_requests {
            if let Some((_, request)) = data.pending_requests.remove(&height) {
                println!("Request for block {} expired, retrying...", height);
                
                // Retry if under limit
                if request.retry_count < data.max_retries {
                    let mut retry_request = request;
                    retry_request.retry_count += 1;
                    retry_request.requested_at = now;
                    
                    // Try different peer for retry
                    if let Some(peer_addr) = Self::select_sync_peer(data, height).await {
                        retry_request.peer = peer_addr;
                        data.pending_requests.insert(height, retry_request);
                        
                        let _ = Self::send_block_request(data, peer_addr, height).await;
                    }
                } else {
                    println!("Max retries reached for block {}", height);
                }
            }
        }
    }

    async fn is_synced(data: &SyncData) -> bool {
        let current_height = {
            let blockchain = data.blockchain.lock().await;
            blockchain.get_chain_height()
        };
        
        let target_height = Self::get_target_height(data).await;
        
        // Consider synced if within 2 blocks of target
        current_height + 2 >= target_height
    }

    pub async fn handle_block_message(&self, peer_addr: SocketAddr, block: Block) -> Result<(), NetworkError> {
        // Calculate block height (this would need proper implementation)
        let block_height = self.calculate_block_height(&block).await;
        
        // Add to buffer
        self.block_buffer.insert(block_height, block);
        
        // Update peer stats
        if let Some(mut sync_peer) = self.sync_peers.get_mut(&peer_addr) {
            sync_peer.blocks_received += 1;
            sync_peer.last_response = Instant::now();
            sync_peer.is_syncing = false;
            sync_peer.update_score();
        }
        
        // Remove from pending requests
        self.pending_requests.remove(&block_height);
        
        Ok(())
    }

    async fn calculate_block_height(&self, _block: &Block) -> u64 {
        // This would calculate height based on block data
        // For now, return placeholder
        0
    }

    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    pub async fn get_sync_progress(&self) -> SyncProgress {
        self.sync_progress.read().await.clone()
    }

    fn clone_sync_data(&self) -> SyncData {
        SyncData {
            blockchain: self.blockchain.clone(),
            peer_manager: self.peer_manager.clone(),
            sync_state: self.sync_state.clone(),
            sync_progress: self.sync_progress.clone(),
            sync_peers: self.sync_peers.clone(),
            pending_requests: self.pending_requests.clone(),
            block_buffer: self.block_buffer.clone(),
            max_concurrent_requests: self.max_concurrent_requests,
            request_timeout: self.request_timeout,
            max_retries: self.max_retries,
            sync_start_time: self.sync_start_time.clone(),
            blocks_synced: self.blocks_synced.clone(),
        }
    }

    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("sync_peers".to_string(), self.sync_peers.len() as u64);
        stats.insert("pending_requests".to_string(), self.pending_requests.len() as u64);
        stats.insert("buffered_blocks".to_string(), self.block_buffer.len() as u64);
        stats.insert("blocks_synced".to_string(), *self.blocks_synced.lock());
        stats
    }
}

#[derive(Clone)]
struct SyncData {
    blockchain: Arc<Mutex<Blockchain>>,
    peer_manager: Arc<PeerManager>,
    sync_state: Arc<RwLock<SyncState>>,
    sync_progress: Arc<RwLock<SyncProgress>>,
    sync_peers: Arc<DashMap<SocketAddr, SyncPeer>>,
    pending_requests: Arc<DashMap<u64, BlockRequest>>,
    block_buffer: Arc<DashMap<u64, Block>>,
    max_concurrent_requests: usize,
    request_timeout: Duration,
    max_retries: u32,
    sync_start_time: Arc<ParkingMutex<Option<Instant>>>,
    blocks_synced: Arc<ParkingMutex<u64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_sync_peer() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut peer = SyncPeer::new(addr, 100, "hash".to_string());
        
        assert_eq!(peer.sync_score, 1.0);
        
        peer.requests_sent = 10;
        peer.blocks_received = 8;
        peer.update_score();
        
        assert_eq!(peer.sync_score, 0.8);
    }
}
