use crate::{Blockchain, Block, NetworkMessage};
use crate::network::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, warn, error, debug};

pub struct BlockchainSync {
    blockchain: Arc<RwLock<Blockchain>>,
    peers: Arc<RwLock<HashMap<SocketAddr, Peer>>>,
    sync_in_progress: Arc<RwLock<bool>>,
}

impl BlockchainSync {
    pub fn new(
        blockchain: Arc<RwLock<Blockchain>>,
        peers: Arc<RwLock<HashMap<SocketAddr, Peer>>>,
    ) -> Self {
        Self {
            blockchain,
            peers,
            sync_in_progress: Arc::new(RwLock::new(false)),
        }
    }
    
    pub async fn start_sync(&self) -> Result<()> {
        // Check if sync is already in progress
        {
            let mut sync_in_progress = self.sync_in_progress.write().await;
            if *sync_in_progress {
                return Ok(());
            }
            *sync_in_progress = true;
        }
        
        info!("Starting blockchain synchronization");
        
        let result = self.perform_sync().await;
        
        // Clear sync flag
        {
            let mut sync_in_progress = self.sync_in_progress.write().await;
            *sync_in_progress = false;
        }
        
        result
    }
    
    async fn perform_sync(&self) -> Result<()> {
        // Get our current chain info
        let (our_height, our_best_hash) = {
            let blockchain = self.blockchain.read().await;
            (blockchain.chain.len() as u64, blockchain.get_latest_block_hash())
        };
        
        info!("Our chain height: {}, best hash: {}", our_height, our_best_hash);
        
        // Get chain info from all peers
        let peer_chain_infos = self.get_peer_chain_infos().await;
        
        if peer_chain_infos.is_empty() {
            warn!("No peers available for sync");
            return Ok(());
        }
        
        // Find the best chain (highest total work)
        let best_peer = peer_chain_infos
            .iter()
            .max_by_key(|(_, info)| info.total_work)
            .map(|(addr, _)| *addr);
        
        if let Some(best_peer_addr) = best_peer {
            let best_info = &peer_chain_infos[&best_peer_addr];
            
            if best_info.height > our_height {
                info!(
                    "Syncing from peer {} (height: {}, our height: {})",
                    best_peer_addr, best_info.height, our_height
                );
                
                self.sync_from_peer(best_peer_addr, our_height).await?;
            } else {
                info!("We are up to date (height: {})", our_height);
            }
        }
        
        Ok(())
    }
    
    async fn get_peer_chain_infos(&self) -> HashMap<SocketAddr, ChainInfo> {
        let mut chain_infos = HashMap::new();
        let peers = self.peers.read().await;
        
        for (addr, peer) in peers.iter() {
            // Request chain info from peer
            // This is simplified - in real implementation, we'd handle timeouts and errors
            debug!("Requesting chain info from {}", addr);
            // For now, we'll skip actual implementation
        }
        
        chain_infos
    }
    
    async fn sync_from_peer(&self, peer_addr: SocketAddr, start_height: u64) -> Result<()> {
        let mut current_height = start_height;
        const BATCH_SIZE: usize = 500;
        
        loop {
            // Request blocks from peer
            let blocks = self.request_blocks_from_peer(
                peer_addr,
                current_height,
                BATCH_SIZE,
            ).await?;
            
            if blocks.is_empty() {
                info!("Sync complete");
                break;
            }
            
            // Validate and add blocks to our chain
            let mut blockchain = self.blockchain.write().await;
            let mut added_count = 0;
            
            for block in blocks {
                match blockchain.add_block(block.clone()) {
                    Ok(_) => {
                        added_count += 1;
                        current_height += 1;
                    }
                    Err(e) => {
                        error!("Failed to add block during sync: {}", e);
                        // In a real implementation, we might want to handle forks here
                        break;
                    }
                }
            }
            
            info!("Added {} blocks, current height: {}", added_count, current_height);
            
            if added_count == 0 {
                warn!("No blocks were added, stopping sync");
                break;
            }
        }
        
        Ok(())
    }
    
    async fn request_blocks_from_peer(
        &self,
        peer_addr: SocketAddr,
        start_height: u64,
        limit: usize,
    ) -> Result<Vec<Block>> {
        let peers = self.peers.read().await;
        
        if let Some(peer) = peers.get(&peer_addr) {
            // Get the hash of the block at start_height
            let start_hash = {
                let blockchain = self.blockchain.read().await;
                if start_height == 0 {
                    "0".to_string()
                } else if let Some(block) = blockchain.chain.get((start_height - 1) as usize) {
                    block.hash.clone()
                } else {
                    return Err(anyhow::anyhow!("Invalid start height"));
                }
            };
            
            // This would send a GetBlocks message and wait for response
            // For now, return empty vector
            warn!("Block request from peer {} not fully implemented", peer_addr);
            Ok(Vec::new())
        } else {
            Err(anyhow::anyhow!("Peer not found: {}", peer_addr))
        }
    }
    
    pub async fn handle_new_block(&self, block: Block, source_peer: SocketAddr) -> Result<bool> {
        let mut blockchain = self.blockchain.write().await;
        
        // Check if we already have this block
        if blockchain.chain.iter().any(|b| b.hash == block.hash) {
            debug!("Already have block {}", block.hash);
            return Ok(false);
        }
        
        // Try to add the block
        match blockchain.add_block(block.clone()) {
            Ok(_) => {
                info!("Added new block {} from peer {}", block.hash, source_peer);
                
                // Broadcast to other peers (excluding source)
                self.broadcast_block_to_peers(block, Some(source_peer)).await;
                
                Ok(true)
            }
            Err(e) => {
                warn!("Failed to add new block from {}: {}", source_peer, e);
                
                // Block might be part of a longer chain - trigger sync
                if blockchain.chain.len() as u64 > 1 {
                    tokio::spawn({
                        let sync = self.clone();
                        async move {
                            let _ = sync.start_sync().await;
                        }
                    });
                }
                
                Ok(false)
            }
        }
    }
    
    async fn broadcast_block_to_peers(&self, block: Block, exclude_peer: Option<SocketAddr>) {
        let mut peers = self.peers.write().await;
        
        for (addr, peer) in peers.iter_mut() {
            if exclude_peer.map_or(true, |ex| *addr != ex) && peer.info.connected {
                let message = NetworkMessage::NewBlock(block.clone());
                if let Err(e) = peer.send_message(&message).await {
                    error!("Failed to broadcast block to {}: {}", addr, e);
                }
            }
        }
    }
    
    pub async fn is_syncing(&self) -> bool {
        *self.sync_in_progress.read().await
    }
}

impl Clone for BlockchainSync {
    fn clone(&self) -> Self {
        Self {
            blockchain: Arc::clone(&self.blockchain),
            peers: Arc::clone(&self.peers),
            sync_in_progress: Arc::clone(&self.sync_in_progress),
        }
    }
}

#[derive(Debug, Clone)]
struct ChainInfo {
    height: u64,
    best_hash: String,
    total_work: u64,
}
