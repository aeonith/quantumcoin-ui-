// Comprehensive peer management with scoring and DoS protection
use crate::network::{ChainSpec, SecurityManager, SecureTransport, NetworkMetrics, SecureConnection};
use crate::network::protocol::{NetworkMessage, ProtocolVersion};
use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use uuid::Uuid;

/// Production-grade peer manager with scoring and DoS protection
pub struct PeerManager {
    chain_spec: Arc<ChainSpec>,
    security_manager: Arc<SecurityManager>,
    transport: Arc<SecureTransport>,
    metrics: Arc<NetworkMetrics>,
    peers: Arc<RwLock<HashMap<SocketAddr, Peer>>>,
    peer_scores: Arc<RwLock<HashMap<SocketAddr, PeerScore>>>,
    banned_peers: Arc<RwLock<HashMap<SocketAddr, BanRecord>>>,
    connection_pool: Arc<RwLock<ConnectionPool>>,
    message_queue: Arc<RwLock<VecDeque<PendingMessage>>>,
    sync_state: Arc<RwLock<SyncState>>,
    shutdown_signal: mpsc::Sender<()>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub address: SocketAddr,
    pub node_id: String,
    pub protocol_version: u32,
    pub services: u64,
    pub user_agent: String,
    pub height: u64,
    pub connected_at: Instant,
    pub last_seen: Instant,
    pub connection_type: ConnectionType,
    pub state: PeerState,
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerState {
    Connecting,
    Handshaking,
    Connected,
    Syncing,
    Ready,
    Disconnecting,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct PeerScore {
    pub overall_score: i32,
    pub reliability_score: f32,
    pub latency_score: f32,
    pub bandwidth_score: f32,
    pub protocol_compliance: f32,
    pub uptime_score: f32,
    pub last_updated: Instant,
    pub connection_successes: u32,
    pub connection_failures: u32,
    pub protocol_violations: u32,
    pub successful_syncs: u32,
}

#[derive(Debug, Clone)]
pub struct BanRecord {
    pub reason: String,
    pub banned_at: SystemTime,
    pub ban_duration: Duration,
    pub violation_count: u32,
}

#[derive(Debug)]
pub struct ConnectionPool {
    pub max_inbound: usize,
    pub max_outbound: usize,
    pub current_inbound: usize,
    pub current_outbound: usize,
    pub connection_slots: HashMap<SocketAddr, ConnectionSlot>,
}

#[derive(Debug)]
pub struct ConnectionSlot {
    pub reserved_at: Instant,
    pub connection_type: ConnectionType,
}

#[derive(Debug)]
pub struct PendingMessage {
    pub target: SocketAddr,
    pub message: NetworkMessage,
    pub priority: MessagePriority,
    pub created_at: Instant,
    pub retry_count: u32,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum MessagePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

#[derive(Debug)]
pub struct SyncState {
    pub is_syncing: bool,
    pub sync_peers: HashSet<SocketAddr>,
    pub best_height: u64,
    pub sync_start_time: Option<Instant>,
    pub blocks_downloaded: u64,
    pub sync_progress: f32,
}

impl PeerManager {
    pub fn new(
        chain_spec: Arc<ChainSpec>,
        security_manager: Arc<SecurityManager>,
        transport: Arc<SecureTransport>,
        metrics: Arc<NetworkMetrics>,
    ) -> Self {
        let (tx, _rx) = mpsc::channel(1);
        
        Self {
            chain_spec,
            security_manager,
            transport,
            metrics,
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_scores: Arc::new(RwLock::new(HashMap::new())),
            banned_peers: Arc::new(RwLock::new(HashMap::new())),
            connection_pool: Arc::new(RwLock::new(ConnectionPool {
                max_inbound: 75,
                max_outbound: 50,
                current_inbound: 0,
                current_outbound: 0,
                connection_slots: HashMap::new(),
            })),
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            sync_state: Arc::new(RwLock::new(SyncState {
                is_syncing: false,
                sync_peers: HashSet::new(),
                best_height: 0,
                sync_start_time: None,
                blocks_downloaded: 0,
                sync_progress: 0.0,
            })),
            shutdown_signal: tx,
        }
    }

    pub async fn start(&self) -> Result<()> {
        log::info!("Starting peer manager");
        
        // Start peer maintenance
        let manager = self.clone();
        tokio::spawn(async move {
            manager.peer_maintenance_loop().await;
        });
        
        // Start message processing
        let manager = self.clone();
        tokio::spawn(async move {
            manager.message_processing_loop().await;
        });
        
        // Start connection management
        let manager = self.clone();
        tokio::spawn(async move {
            manager.connection_management_loop().await;
        });
        
        // Start peer scoring
        let manager = self.clone();
        tokio::spawn(async move {
            manager.peer_scoring_loop().await;
        });

        Ok(())
    }

    /// Connect to a peer with full validation and scoring
    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        log::debug!("Attempting to connect to peer {}", addr);
        
        // Check if banned
        if self.is_peer_banned(addr).await {
            return Err(anyhow::anyhow!("Peer {} is banned", addr));
        }
        
        // Check connection limits
        if !self.can_connect_outbound().await {
            return Err(anyhow::anyhow!("Maximum outbound connections reached"));
        }
        
        // Reserve connection slot
        self.reserve_connection_slot(addr, ConnectionType::Outbound).await;
        
        // Attempt secure connection
        let connection_result = self.transport.connect_secure(addr).await;
        match connection_result {
            Ok(connection) => {
                self.on_connection_established(addr, ConnectionType::Outbound, connection).await?;
                Ok(())
            }
            Err(e) => {
                self.release_connection_slot(addr).await;
                self.record_connection_failure(addr, &e.to_string()).await;
                Err(e)
            }
        }
    }

    /// Try to connect without strict error handling (for discovery)
    pub async fn try_connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        if let Err(e) = self.connect_to_peer(addr).await {
            log::debug!("Failed to connect to {}: {}", addr, e);
            return Err(e);
        }
        Ok(())
    }

    /// Accept incoming peer connection
    pub async fn accept_peer_connection(&self, addr: SocketAddr, connection: SecureConnection) -> Result<()> {
        log::debug!("Accepting connection from {}", addr);
        
        // Check if banned
        if self.is_peer_banned(addr).await {
            return Err(anyhow::anyhow!("Peer {} is banned", addr));
        }
        
        // Check connection limits
        if !self.can_accept_inbound().await {
            return Err(anyhow::anyhow!("Maximum inbound connections reached"));
        }
        
        // DoS protection
        if !self.security_manager.allow_connection(addr).await {
            return Err(anyhow::anyhow!("Connection rejected by security manager"));
        }
        
        self.on_connection_established(addr, ConnectionType::Inbound, connection).await?;
        Ok(())
    }

    /// Handle established connection
    async fn on_connection_established(
        &self,
        addr: SocketAddr,
        connection_type: ConnectionType,
        connection: SecureConnection,
    ) -> Result<()> {
        // Create peer record
        let peer = Peer {
            address: addr,
            node_id: Uuid::new_v4().to_string(), // Will be updated during handshake
            protocol_version: 0,
            services: 0,
            user_agent: String::new(),
            height: 0,
            connected_at: Instant::now(),
            last_seen: Instant::now(),
            connection_type: connection_type.clone(),
            state: PeerState::Handshaking,
        };
        
        // Add to peers
        self.peers.write().await.insert(addr, peer);
        
        // Initialize peer score
        if !self.peer_scores.read().await.contains_key(&addr) {
            self.initialize_peer_score(addr).await;
        }
        
        // Update connection pool
        match connection_type {
            ConnectionType::Inbound => {
                self.connection_pool.write().await.current_inbound += 1;
            }
            ConnectionType::Outbound => {
                self.connection_pool.write().await.current_outbound += 1;
            }
        }
        
        // Start peer protocol handshake
        self.initiate_peer_handshake(addr).await?;
        
        log::info!("Peer connection established: {}", addr);
        self.metrics.increment_peer_connections().await;
        
        Ok(())
    }

    /// Initiate protocol handshake
    async fn initiate_peer_handshake(&self, addr: SocketAddr) -> Result<()> {
        let version_message = NetworkMessage::Version {
            version: self.chain_spec.protocol_version,
            services: 1, // NODE_NETWORK
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            user_agent: "QuantumCoin/2.0.0".to_string(),
            start_height: 0, // Will be filled by blockchain
            relay: true,
        };
        
        self.send_message_to_peer(addr, version_message).await
    }

    /// Send message to specific peer
    pub async fn send_message_to_peer(&self, addr: SocketAddr, message: NetworkMessage) -> Result<()> {
        // Serialize message
        let data = message.serialize()?;
        
        // Add to message queue for rate limiting
        let pending = PendingMessage {
            target: addr,
            message,
            priority: MessagePriority::Normal,
            created_at: Instant::now(),
            retry_count: 0,
        };
        
        self.message_queue.write().await.push_back(pending);
        Ok(())
    }

    /// Broadcast message to all connected peers
    pub async fn broadcast_message(&self, message: NetworkMessage) -> Result<()> {
        let peers = self.get_ready_peers().await;
        for peer_addr in peers {
            let _ = self.send_message_to_peer(peer_addr, message.clone()).await;
        }
        Ok(())
    }

    /// Get peers ready for communication
    async fn get_ready_peers(&self) -> Vec<SocketAddr> {
        self.peers
            .read()
            .await
            .values()
            .filter(|p| p.state == PeerState::Ready)
            .map(|p| p.address)
            .collect()
    }

    /// Request full blockchain sync
    pub async fn request_full_sync(&self) -> Result<()> {
        let ready_peers = self.get_ready_peers().await;
        if ready_peers.is_empty() {
            return Err(anyhow::anyhow!("No peers available for sync"));
        }
        
        // Select best peers for sync
        let sync_peers = self.select_sync_peers(&ready_peers, 3).await;
        
        let mut sync_state = self.sync_state.write().await;
        sync_state.is_syncing = true;
        sync_state.sync_peers = sync_peers.clone();
        sync_state.sync_start_time = Some(Instant::now());
        sync_state.blocks_downloaded = 0;
        drop(sync_state);
        
        // Request headers from sync peers
        for peer_addr in sync_peers {
            let headers_request = NetworkMessage::GetHeaders {
                start_hash: "0".repeat(64), // Genesis
                stop_hash: "0".repeat(64),   // Latest
            };
            
            let _ = self.send_message_to_peer(peer_addr, headers_request).await;
        }
        
        log::info!("Started full blockchain sync");
        Ok(())
    }

    /// Select best peers for syncing
    async fn select_sync_peers(&self, candidates: &[SocketAddr], count: usize) -> HashSet<SocketAddr> {
        let scores = self.peer_scores.read().await;
        let mut scored_peers: Vec<_> = candidates
            .iter()
            .filter_map(|addr| {
                scores.get(addr).map(|score| (*addr, score.overall_score))
            })
            .collect();
        
        scored_peers.sort_by(|a, b| b.1.cmp(&a.1));
        scored_peers
            .into_iter()
            .take(count)
            .map(|(addr, _)| addr)
            .collect()
    }

    /// Check connection limits
    async fn can_connect_outbound(&self) -> bool {
        let pool = self.connection_pool.read().await;
        pool.current_outbound < pool.max_outbound
    }

    async fn can_accept_inbound(&self) -> bool {
        let pool = self.connection_pool.read().await;
        pool.current_inbound < pool.max_inbound
    }

    /// Connection slot management
    async fn reserve_connection_slot(&self, addr: SocketAddr, connection_type: ConnectionType) {
        let mut pool = self.connection_pool.write().await;
        pool.connection_slots.insert(addr, ConnectionSlot {
            reserved_at: Instant::now(),
            connection_type,
        });
    }

    async fn release_connection_slot(&self, addr: SocketAddr) {
        self.connection_pool.write().await.connection_slots.remove(&addr);
    }

    /// Ban management
    async fn is_peer_banned(&self, addr: SocketAddr) -> bool {
        if let Some(ban_record) = self.banned_peers.read().await.get(&addr) {
            let now = SystemTime::now();
            if let Ok(elapsed) = now.duration_since(ban_record.banned_at) {
                return elapsed < ban_record.ban_duration;
            }
        }
        false
    }

    pub async fn ban_peer(&self, addr: SocketAddr, reason: String, duration: Duration) {
        log::warn!("Banning peer {} for {}: {}", addr, duration.as_secs(), reason);
        
        let ban_record = BanRecord {
            reason,
            banned_at: SystemTime::now(),
            ban_duration: duration,
            violation_count: 1,
        };
        
        self.banned_peers.write().await.insert(addr, ban_record);
        
        // Disconnect if currently connected
        self.disconnect_peer(addr, "banned").await;
    }

    /// Disconnect peer
    pub async fn disconnect_peer(&self, addr: SocketAddr, reason: &str) {
        log::info!("Disconnecting peer {}: {}", addr, reason);
        
        // Update peer state
        if let Some(peer) = self.peers.write().await.get_mut(&addr) {
            peer.state = PeerState::Disconnected;
        }
        
        // Update connection pool
        let mut pool = self.connection_pool.write().await;
        if let Some(slot) = pool.connection_slots.remove(&addr) {
            match slot.connection_type {
                ConnectionType::Inbound => pool.current_inbound -= 1,
                ConnectionType::Outbound => pool.current_outbound -= 1,
            }
        }
        
        self.metrics.increment_peer_disconnections().await;
    }

    /// Initialize peer scoring
    async fn initialize_peer_score(&self, addr: SocketAddr) {
        let score = PeerScore {
            overall_score: 50, // Start neutral
            reliability_score: 0.5,
            latency_score: 0.5,
            bandwidth_score: 0.5,
            protocol_compliance: 1.0,
            uptime_score: 0.5,
            last_updated: Instant::now(),
            connection_successes: 0,
            connection_failures: 0,
            protocol_violations: 0,
            successful_syncs: 0,
        };
        
        self.peer_scores.write().await.insert(addr, score);
    }

    /// Record connection failure for scoring
    async fn record_connection_failure(&self, addr: SocketAddr, _error: &str) {
        if let Some(score) = self.peer_scores.write().await.get_mut(&addr) {
            score.connection_failures += 1;
            score.reliability_score *= 0.9; // Reduce reliability
            score.overall_score = ((score.reliability_score * 100.0) as i32).max(0);
        }
    }

    /// Maintenance loops
    async fn peer_maintenance_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Clean up disconnected peers
            self.cleanup_disconnected_peers().await;
            
            // Update peer last_seen
            self.update_peer_activity().await;
            
            // Remove expired bans
            self.cleanup_expired_bans().await;
        }
    }

    async fn message_processing_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(10));
        
        loop {
            interval.tick().await;
            
            // Process message queue with rate limiting
            if let Some(pending) = self.message_queue.write().await.pop_front() {
                if let Err(e) = self.transport.send_secure(pending.target, &pending.message.serialize().unwrap_or_default()).await {
                    log::debug!("Failed to send message to {}: {}", pending.target, e);
                    
                    // Retry logic
                    if pending.retry_count < 3 {
                        let mut retry_msg = pending;
                        retry_msg.retry_count += 1;
                        self.message_queue.write().await.push_back(retry_msg);
                    }
                }
            }
        }
    }

    async fn connection_management_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            // Ensure we have enough outbound connections
            let current_outbound = self.connection_pool.read().await.current_outbound;
            let target_outbound = 8; // Minimum outbound connections
            
            if current_outbound < target_outbound {
                log::debug!("Need more outbound connections: {} < {}", current_outbound, target_outbound);
                // Trigger peer discovery
            }
        }
    }

    async fn peer_scoring_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        
        loop {
            interval.tick().await;
            
            // Update all peer scores
            self.update_all_peer_scores().await;
        }
    }

    async fn cleanup_disconnected_peers(&self) {
        let now = Instant::now();
        let mut peers = self.peers.write().await;
        
        peers.retain(|_, peer| {
            if peer.state == PeerState::Disconnected {
                now.duration_since(peer.last_seen) < Duration::from_secs(300)
            } else {
                true
            }
        });
    }

    async fn update_peer_activity(&self) {
        // This would be called when receiving messages from peers
        // For now, just a placeholder
    }

    async fn cleanup_expired_bans(&self) {
        let now = SystemTime::now();
        let mut banned = self.banned_peers.write().await;
        
        banned.retain(|_, ban_record| {
            if let Ok(elapsed) = now.duration_since(ban_record.banned_at) {
                elapsed < ban_record.ban_duration
            } else {
                false
            }
        });
    }

    async fn update_all_peer_scores(&self) {
        // Update peer scores based on recent activity, latency, etc.
        let mut scores = self.peer_scores.write().await;
        for (_, score) in scores.iter_mut() {
            score.last_updated = Instant::now();
            // Update scoring logic here
        }
    }

    // Public API methods
    pub async fn get_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub async fn get_connected_peers(&self) -> Vec<String> {
        self.peers
            .read()
            .await
            .values()
            .filter(|p| p.state == PeerState::Ready)
            .map(|p| p.address.to_string())
            .collect()
    }

    pub async fn get_sync_progress(&self) -> f32 {
        self.sync_state.read().await.sync_progress
    }

    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down peer manager");
        let _ = self.shutdown_signal.send(()).await;
        Ok(())
    }
}

impl Clone for PeerManager {
    fn clone(&self) -> Self {
        Self {
            chain_spec: self.chain_spec.clone(),
            security_manager: self.security_manager.clone(),
            transport: self.transport.clone(),
            metrics: self.metrics.clone(),
            peers: self.peers.clone(),
            peer_scores: self.peer_scores.clone(),
            banned_peers: self.banned_peers.clone(),
            connection_pool: self.connection_pool.clone(),
            message_queue: self.message_queue.clone(),
            sync_state: self.sync_state.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
        }
    }
}
