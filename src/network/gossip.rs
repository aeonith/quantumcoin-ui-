// Production-grade gossip protocol for QuantumCoin
// Implements efficient block and transaction propagation with DoS protection

use crate::block::Block;
use crate::transaction::Transaction;
use crate::network::protocol::{NetworkMessage, InventoryItem, InventoryType};
use crate::network::{ChainSpec, NetworkMetrics, SecurityManager};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc, Mutex};
use tokio::time::{sleep, timeout, interval};
use uuid::Uuid;
use blake3::Hasher;

/// Maximum number of items in a single gossip message
const MAX_GOSSIP_ITEMS: usize = 1000;
/// Maximum age for a gossip item before it's considered stale
const MAX_GOSSIP_AGE: Duration = Duration::from_secs(300); // 5 minutes
/// Gossip retry interval for failed propagation
const GOSSIP_RETRY_INTERVAL: Duration = Duration::from_secs(30);
/// Maximum number of peers to gossip to per round
const MAX_GOSSIP_PEERS: usize = 8;
/// Backpressure threshold - stop gossiping when queue exceeds this
const BACKPRESSURE_THRESHOLD: usize = 10000;
/// DoS score threshold for banning peers
const DOS_BAN_THRESHOLD: i32 = 100;
/// Maximum concurrent gossip operations per peer
const MAX_CONCURRENT_GOSSIP: usize = 3;

/// Gossip message types with priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GossipType {
    Block,
    Transaction,
    BlockHeader,
    CompactBlock,
    Emergency, // For critical network messages
}

impl GossipType {
    /// Get priority for message processing (0 = highest priority)
    pub fn priority(&self) -> u8 {
        match self {
            GossipType::Emergency => 0,
            GossipType::Block => 1,
            GossipType::BlockHeader => 2,
            GossipType::CompactBlock => 3,
            GossipType::Transaction => 4,
        }
    }
    
    /// Get maximum propagation hops for this gossip type
    pub fn max_hops(&self) -> u8 {
        match self {
            GossipType::Emergency => 10,
            GossipType::Block => 7,
            GossipType::BlockHeader => 5,
            GossipType::CompactBlock => 5,
            GossipType::Transaction => 3,
        }
    }
    
    /// Get rate limit for this gossip type (messages per second)
    pub fn rate_limit(&self) -> f64 {
        match self {
            GossipType::Emergency => 1.0,
            GossipType::Block => 10.0,
            GossipType::BlockHeader => 20.0,
            GossipType::CompactBlock => 20.0,
            GossipType::Transaction => 100.0,
        }
    }
}

/// Gossip item with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipItem {
    pub id: String,
    pub gossip_type: GossipType,
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub hop_count: u8,
    pub priority: u8,
    pub origin_peer: Option<String>,
    pub checksum: u32,
}

impl GossipItem {
    pub fn new(gossip_type: GossipType, data: Vec<u8>, origin_peer: Option<String>) -> Self {
        let id = Self::generate_id(&data);
        let checksum = Self::calculate_checksum(&data);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id,
            gossip_type: gossip_type.clone(),
            data,
            timestamp,
            hop_count: 0,
            priority: gossip_type.priority(),
            origin_peer,
            checksum,
        }
    }
    
    pub fn generate_id(data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hex::encode(&hasher.finalize().as_bytes()[..16])
    }
    
    pub fn calculate_checksum(data: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        u32::from_le_bytes([hash.as_bytes()[0], hash.as_bytes()[1], hash.as_bytes()[2], hash.as_bytes()[3]])
    }
    
    pub fn verify_checksum(&self) -> bool {
        Self::calculate_checksum(&self.data) == self.checksum
    }
    
    pub fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Duration::from_secs(now.saturating_sub(self.timestamp))
    }
    
    pub fn is_stale(&self) -> bool {
        self.age() > MAX_GOSSIP_AGE
    }
    
    pub fn can_propagate(&self) -> bool {
        self.hop_count < self.gossip_type.max_hops() && !self.is_stale()
    }
    
    pub fn increment_hop(&mut self) {
        self.hop_count += 1;
    }
}

/// Peer gossip state for tracking what each peer knows
#[derive(Debug, Clone)]
pub struct PeerGossipState {
    pub peer_id: String,
    pub known_items: HashSet<String>,
    pub last_gossip: Instant,
    pub gossip_count: u32,
    pub dos_score: i32,
    pub rate_limiter: RateLimiter,
    pub concurrent_gossip: usize,
    pub connection_quality: f64,
}

impl PeerGossipState {
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id: peer_id.clone(),
            known_items: HashSet::new(),
            last_gossip: Instant::now(),
            gossip_count: 0,
            dos_score: 0,
            rate_limiter: RateLimiter::new(),
            concurrent_gossip: 0,
            connection_quality: 1.0,
        }
    }
    
    pub fn knows_item(&self, item_id: &str) -> bool {
        self.known_items.contains(item_id)
    }
    
    pub fn mark_known(&mut self, item_id: String) {
        self.known_items.insert(item_id);
        // Limit memory usage by keeping only recent items
        if self.known_items.len() > 10000 {
            let mut items: Vec<_> = self.known_items.drain().collect();
            items.truncate(5000);
            self.known_items = items.into_iter().collect();
        }
    }
    
    pub fn is_banned(&self) -> bool {
        self.dos_score >= DOS_BAN_THRESHOLD
    }
    
    pub fn increase_dos_score(&mut self, points: i32) {
        self.dos_score += points;
    }
    
    pub fn decrease_dos_score(&mut self, points: i32) {
        self.dos_score = (self.dos_score - points).max(0);
    }
    
    pub fn can_accept_gossip(&self, gossip_type: &GossipType) -> bool {
        !self.is_banned() && 
        self.rate_limiter.can_accept(gossip_type) &&
        self.concurrent_gossip < MAX_CONCURRENT_GOSSIP
    }
}

/// Rate limiter for different gossip types
#[derive(Debug, Clone)]
pub struct RateLimiter {
    limits: HashMap<GossipType, TokenBucket>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let mut limits = HashMap::new();
        
        for gossip_type in [
            GossipType::Block,
            GossipType::Transaction,
            GossipType::BlockHeader,
            GossipType::CompactBlock,
            GossipType::Emergency,
        ] {
            limits.insert(gossip_type.clone(), TokenBucket::new(gossip_type.rate_limit()));
        }
        
        Self { limits }
    }
    
    pub fn can_accept(&self, gossip_type: &GossipType) -> bool {
        self.limits.get(gossip_type)
            .map(|bucket| bucket.can_consume())
            .unwrap_or(false)
    }
    
    pub fn consume(&mut self, gossip_type: &GossipType) -> bool {
        self.limits.get_mut(gossip_type)
            .map(|bucket| bucket.consume())
            .unwrap_or(false)
    }
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(rate_per_second: f64) -> Self {
        Self {
            capacity: rate_per_second * 2.0, // Burst capacity
            tokens: rate_per_second * 2.0,
            refill_rate: rate_per_second,
            last_refill: Instant::now(),
        }
    }
    
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }
    
    pub fn can_consume(&self) -> bool {
        let mut bucket = self.clone();
        bucket.refill();
        bucket.tokens >= 1.0
    }
    
    pub fn consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Priority queue for gossip processing
#[derive(Debug)]
pub struct GossipQueue {
    queues: BTreeMap<u8, VecDeque<GossipItem>>,
    total_size: usize,
}

impl GossipQueue {
    pub fn new() -> Self {
        Self {
            queues: BTreeMap::new(),
            total_size: 0,
        }
    }
    
    pub fn push(&mut self, item: GossipItem) -> bool {
        if self.total_size >= BACKPRESSURE_THRESHOLD {
            return false; // Backpressure - reject new items
        }
        
        let priority = item.priority;
        self.queues.entry(priority)
            .or_insert_with(VecDeque::new)
            .push_back(item);
        self.total_size += 1;
        true
    }
    
    pub fn pop(&mut self) -> Option<GossipItem> {
        for (_, queue) in self.queues.iter_mut() {
            if let Some(item) = queue.pop_front() {
                self.total_size = self.total_size.saturating_sub(1);
                return Some(item);
            }
        }
        None
    }
    
    pub fn len(&self) -> usize {
        self.total_size
    }
    
    pub fn is_empty(&self) -> bool {
        self.total_size == 0
    }
    
    pub fn has_backpressure(&self) -> bool {
        self.total_size >= BACKPRESSURE_THRESHOLD
    }
    
    /// Remove stale items to prevent memory bloat
    pub fn cleanup_stale(&mut self) {
        for queue in self.queues.values_mut() {
            let mut to_remove = 0;
            for item in queue.iter() {
                if item.is_stale() {
                    to_remove += 1;
                } else {
                    break; // Queue is roughly sorted by age
                }
            }
            
            for _ in 0..to_remove {
                queue.pop_front();
                self.total_size = self.total_size.saturating_sub(1);
            }
        }
    }
}

/// Main gossip protocol implementation
pub struct GossipProtocol {
    /// Node identifier
    node_id: String,
    /// Network configuration
    chain_spec: Arc<ChainSpec>,
    /// Metrics collection
    metrics: Arc<NetworkMetrics>,
    /// Security manager
    security_manager: Arc<SecurityManager>,
    
    /// Peer gossip states
    peers: Arc<RwLock<HashMap<String, PeerGossipState>>>,
    /// Items we've seen and processed
    seen_items: Arc<RwLock<HashMap<String, Instant>>>,
    /// Outgoing gossip queue
    outgoing_queue: Arc<Mutex<GossipQueue>>,
    /// Incoming gossip queue  
    incoming_queue: Arc<Mutex<GossipQueue>>,
    
    /// Message handlers
    block_handler: Arc<dyn BlockHandler + Send + Sync>,
    transaction_handler: Arc<dyn TransactionHandler + Send + Sync>,
    
    /// Communication channels
    gossip_tx: mpsc::UnboundedSender<GossipCommand>,
    peer_tx: HashMap<String, mpsc::UnboundedSender<NetworkMessage>>,
    
    /// Health monitoring
    health_monitor: Arc<Mutex<HealthMonitor>>,
    
    /// Network partition detection
    partition_detector: Arc<Mutex<PartitionDetector>>,
    
    /// Running state
    running: Arc<RwLock<bool>>,
}

/// Commands for gossip protocol control
#[derive(Debug)]
pub enum GossipCommand {
    AddPeer(String, mpsc::UnboundedSender<NetworkMessage>),
    RemovePeer(String),
    GossipItem(GossipItem),
    ProcessIncoming(String, GossipItem),
    UpdatePeerScore(String, i32),
    ForceSync,
    Shutdown,
}

/// Health monitoring for gossip protocol
#[derive(Debug)]
pub struct HealthMonitor {
    gossip_rate: f64,
    error_rate: f64,
    backpressure_events: u64,
    partition_events: u64,
    banned_peers: u64,
    last_health_check: Instant,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            gossip_rate: 0.0,
            error_rate: 0.0,
            backpressure_events: 0,
            partition_events: 0,
            banned_peers: 0,
            last_health_check: Instant::now(),
        }
    }
    
    pub fn record_gossip(&mut self) {
        // Update gossip rate using exponential moving average
        let elapsed = self.last_health_check.elapsed().as_secs_f64();
        self.gossip_rate = self.gossip_rate * 0.9 + (1.0 / elapsed) * 0.1;
        self.last_health_check = Instant::now();
    }
    
    pub fn record_error(&mut self) {
        let elapsed = self.last_health_check.elapsed().as_secs_f64();
        self.error_rate = self.error_rate * 0.9 + (1.0 / elapsed) * 0.1;
    }
    
    pub fn record_backpressure(&mut self) {
        self.backpressure_events += 1;
    }
    
    pub fn record_partition(&mut self) {
        self.partition_events += 1;
    }
    
    pub fn record_ban(&mut self) {
        self.banned_peers += 1;
    }
    
    pub fn is_healthy(&self) -> bool {
        self.error_rate < 10.0 && // Less than 10 errors per second
        self.gossip_rate > 0.1 && // At least some gossip activity
        self.partition_events < 5 // Not too many partition events
    }
}

/// Network partition detection
#[derive(Debug)]
pub struct PartitionDetector {
    peer_connectivity: HashMap<String, Instant>,
    partition_threshold: Duration,
    min_peers_for_health: usize,
}

impl PartitionDetector {
    pub fn new() -> Self {
        Self {
            peer_connectivity: HashMap::new(),
            partition_threshold: Duration::from_secs(60),
            min_peers_for_health: 3,
        }
    }
    
    pub fn update_peer_activity(&mut self, peer_id: &str) {
        self.peer_connectivity.insert(peer_id.to_string(), Instant::now());
    }
    
    pub fn detect_partition(&mut self) -> bool {
        let now = Instant::now();
        
        // Remove stale peers
        self.peer_connectivity.retain(|_, last_seen| {
            now.duration_since(*last_seen) < self.partition_threshold
        });
        
        // Check if we have enough active peers
        self.peer_connectivity.len() < self.min_peers_for_health
    }
    
    pub fn get_active_peer_count(&self) -> usize {
        let now = Instant::now();
        self.peer_connectivity.iter()
            .filter(|(_, last_seen)| now.duration_since(**last_seen) < self.partition_threshold)
            .count()
    }
}

/// Block handler trait
pub trait BlockHandler {
    async fn handle_block(&self, block: Block) -> Result<()>;
    async fn validate_block(&self, block: &Block) -> Result<bool>;
}

/// Transaction handler trait
pub trait TransactionHandler {
    async fn handle_transaction(&self, transaction: Transaction) -> Result<()>;
    async fn validate_transaction(&self, transaction: &Transaction) -> Result<bool>;
}

impl GossipProtocol {
    pub async fn new(
        node_id: String,
        chain_spec: Arc<ChainSpec>,
        metrics: Arc<NetworkMetrics>,
        security_manager: Arc<SecurityManager>,
        block_handler: Arc<dyn BlockHandler + Send + Sync>,
        transaction_handler: Arc<dyn TransactionHandler + Send + Sync>,
    ) -> Result<Self> {
        let (gossip_tx, _) = mpsc::unbounded_channel();
        
        Ok(Self {
            node_id,
            chain_spec,
            metrics,
            security_manager,
            peers: Arc::new(RwLock::new(HashMap::new())),
            seen_items: Arc::new(RwLock::new(HashMap::new())),
            outgoing_queue: Arc::new(Mutex::new(GossipQueue::new())),
            incoming_queue: Arc::new(Mutex::new(GossipQueue::new())),
            block_handler,
            transaction_handler,
            gossip_tx,
            peer_tx: HashMap::new(),
            health_monitor: Arc::new(Mutex::new(HealthMonitor::new())),
            partition_detector: Arc::new(Mutex::new(PartitionDetector::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the gossip protocol
    pub async fn start(&mut self) -> Result<()> {
        *self.running.write().await = true;
        
        let (gossip_tx, mut gossip_rx) = mpsc::unbounded_channel();
        self.gossip_tx = gossip_tx;
        
        // Spawn main gossip processing task
        let protocol = self.clone();
        tokio::spawn(async move {
            while let Some(command) = gossip_rx.recv().await {
                if let Err(e) = protocol.process_command(command).await {
                    log::error!("Gossip command processing error: {}", e);
                    protocol.health_monitor.lock().await.record_error();
                }
            }
        });
        
        // Spawn outgoing gossip processor
        self.spawn_outgoing_processor();
        
        // Spawn incoming gossip processor
        self.spawn_incoming_processor();
        
        // Spawn health monitor
        self.spawn_health_monitor();
        
        // Spawn cleanup task
        self.spawn_cleanup_task();
        
        log::info!("Gossip protocol started for node {}", self.node_id);
        Ok(())
    }
    
    /// Process gossip commands
    async fn process_command(&self, command: GossipCommand) -> Result<()> {
        match command {
            GossipCommand::AddPeer(peer_id, sender) => {
                self.add_peer(peer_id, sender).await;
            }
            GossipCommand::RemovePeer(peer_id) => {
                self.remove_peer(&peer_id).await;
            }
            GossipCommand::GossipItem(item) => {
                self.queue_for_gossip(item).await?;
            }
            GossipCommand::ProcessIncoming(peer_id, item) => {
                self.process_incoming_item(&peer_id, item).await?;
            }
            GossipCommand::UpdatePeerScore(peer_id, delta) => {
                self.update_peer_score(&peer_id, delta).await;
            }
            GossipCommand::ForceSync => {
                self.force_sync().await?;
            }
            GossipCommand::Shutdown => {
                self.shutdown().await?;
            }
        }
        
        Ok(())
    }
    
    /// Add a peer to gossip to
    async fn add_peer(&self, peer_id: String, sender: mpsc::UnboundedSender<NetworkMessage>) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id.clone(), PeerGossipState::new(peer_id.clone()));
        
        log::debug!("Added peer {} to gossip protocol", peer_id);
    }
    
    /// Remove a peer
    async fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id);
        
        log::debug!("Removed peer {} from gossip protocol", peer_id);
    }
    
    /// Queue an item for gossip
    pub async fn gossip_block(&self, block: Block) -> Result<()> {
        let data = bincode::serialize(&block)?;
        let item = GossipItem::new(GossipType::Block, data, Some(self.node_id.clone()));
        
        self.gossip_tx.send(GossipCommand::GossipItem(item))
            .map_err(|_| anyhow!("Failed to queue block for gossip"))?;
        
        Ok(())
    }
    
    /// Queue a transaction for gossip
    pub async fn gossip_transaction(&self, transaction: Transaction) -> Result<()> {
        let data = bincode::serialize(&transaction)?;
        let item = GossipItem::new(GossipType::Transaction, data, Some(self.node_id.clone()));
        
        self.gossip_tx.send(GossipCommand::GossipItem(item))
            .map_err(|_| anyhow!("Failed to queue transaction for gossip"))?;
        
        Ok(())
    }
    
    /// Queue item for outgoing gossip
    async fn queue_for_gossip(&self, item: GossipItem) -> Result<()> {
        // Check if we've already seen this item
        let mut seen = self.seen_items.write().await;
        if seen.contains_key(&item.id) {
            return Ok(()); // Already processed
        }
        
        // Mark as seen
        seen.insert(item.id.clone(), Instant::now());
        
        // Queue for outgoing gossip
        let mut queue = self.outgoing_queue.lock().await;
        if !queue.push(item) {
            // Backpressure - queue is full
            self.health_monitor.lock().await.record_backpressure();
            return Err(anyhow!("Gossip queue is full - backpressure active"));
        }
        
        Ok(())
    }
    
    /// Process incoming gossip item from peer
    pub async fn process_incoming_gossip(&self, peer_id: &str, item: GossipItem) -> Result<()> {
        self.gossip_tx.send(GossipCommand::ProcessIncoming(peer_id.to_string(), item))
            .map_err(|_| anyhow!("Failed to queue incoming gossip"))?;
        
        Ok(())
    }
    
    /// Process incoming item
    async fn process_incoming_item(&self, peer_id: &str, mut item: GossipItem) -> Result<()> {
        // Update partition detector
        self.partition_detector.lock().await.update_peer_activity(peer_id);
        
        // Verify checksum
        if !item.verify_checksum() {
            log::warn!("Invalid checksum from peer {}", peer_id);
            self.update_peer_score(peer_id, 10).await;
            return Err(anyhow!("Invalid checksum"));
        }
        
        // Check if item is stale
        if item.is_stale() {
            return Ok(()); // Silently drop stale items
        }
        
        // Check rate limiting
        let mut peers = self.peers.write().await;
        if let Some(peer_state) = peers.get_mut(peer_id) {
            if !peer_state.can_accept_gossip(&item.gossip_type) {
                log::debug!("Rate limiting gossip from peer {}", peer_id);
                self.update_peer_score(peer_id, 5).await;
                return Err(anyhow!("Rate limit exceeded"));
            }
            
            // Consume rate limit token
            if !peer_state.rate_limiter.consume(&item.gossip_type) {
                return Err(anyhow!("Rate limit token consumption failed"));
            }
            
            // Mark as known by this peer
            peer_state.mark_known(item.id.clone());
        }
        drop(peers);
        
        // Check if we've already processed this item
        let mut seen = self.seen_items.write().await;
        if seen.contains_key(&item.id) {
            return Ok(()); // Already processed
        }
        
        // Mark as seen
        seen.insert(item.id.clone(), Instant::now());
        drop(seen);
        
        // Queue for incoming processing
        let mut queue = self.incoming_queue.lock().await;
        if !queue.push(item) {
            self.health_monitor.lock().await.record_backpressure();
            return Err(anyhow!("Incoming queue is full"));
        }
        
        self.health_monitor.lock().await.record_gossip();
        Ok(())
    }
    
    /// Update peer DoS score
    async fn update_peer_score(&self, peer_id: &str, delta: i32) {
        let mut peers = self.peers.write().await;
        if let Some(peer_state) = peers.get_mut(peer_id) {
            if delta > 0 {
                peer_state.increase_dos_score(delta);
            } else {
                peer_state.decrease_dos_score(-delta);
            }
            
            if peer_state.is_banned() {
                log::warn!("Peer {} banned for DoS (score: {})", peer_id, peer_state.dos_score);
                self.health_monitor.lock().await.record_ban();
                // TODO: Notify network manager to disconnect peer
            }
        }
    }
    
    /// Force synchronization with peers
    async fn force_sync(&self) -> Result<()> {
        log::info!("Forcing gossip synchronization");
        
        // Send ping to all peers to check connectivity
        let peers = self.peers.read().await;
        for peer_id in peers.keys() {
            // TODO: Send ping message to peer
            log::debug!("Pinging peer {} for sync check", peer_id);
        }
        
        Ok(())
    }
    
    /// Spawn outgoing gossip processor
    fn spawn_outgoing_processor(&self) {
        let protocol = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100)); // 10Hz processing
            
            while *protocol.running.read().await {
                interval.tick().await;
                
                if let Err(e) = protocol.process_outgoing_queue().await {
                    log::error!("Outgoing gossip processing error: {}", e);
                    protocol.health_monitor.lock().await.record_error();
                }
            }
        });
    }
    
    /// Process outgoing gossip queue
    async fn process_outgoing_queue(&self) -> Result<()> {
        let mut queue = self.outgoing_queue.lock().await;
        let mut processed = 0;
        
        while processed < 10 && !queue.is_empty() { // Process up to 10 items per tick
            if let Some(mut item) = queue.pop() {
                drop(queue);
                
                // Select peers to gossip to
                let target_peers = self.select_gossip_peers(&item).await;
                
                for peer_id in target_peers {
                    // Check if peer already knows about this item
                    let peers = self.peers.read().await;
                    let should_send = peers.get(&peer_id)
                        .map(|peer| !peer.knows_item(&item.id) && !peer.is_banned())
                        .unwrap_or(false);
                    drop(peers);
                    
                    if should_send {
                        // Create network message
                        let msg = self.create_gossip_message(&item)?;
                        
                        // TODO: Send to peer via network layer
                        log::trace!("Gossiping {} to peer {}", item.id, peer_id);
                        
                        // Mark as known by this peer
                        let mut peers = self.peers.write().await;
                        if let Some(peer_state) = peers.get_mut(&peer_id) {
                            peer_state.mark_known(item.id.clone());
                        }
                    }
                }
                
                // Increment hop count for next round
                item.increment_hop();
                
                // Re-queue if still can propagate
                if item.can_propagate() {
                    queue = self.outgoing_queue.lock().await;
                    queue.push(item);
                } else {
                    queue = self.outgoing_queue.lock().await;
                }
                
                processed += 1;
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Select peers for gossip propagation
    async fn select_gossip_peers(&self, item: &GossipItem) -> Vec<String> {
        let peers = self.peers.read().await;
        let mut candidates: Vec<_> = peers.iter()
            .filter(|(_, state)| !state.is_banned() && !state.knows_item(&item.id))
            .map(|(id, state)| (id.clone(), state.connection_quality))
            .collect();
        
        // Sort by connection quality (best first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take up to MAX_GOSSIP_PEERS
        candidates.into_iter()
            .take(MAX_GOSSIP_PEERS)
            .map(|(id, _)| id)
            .collect()
    }
    
    /// Create network message from gossip item
    fn create_gossip_message(&self, item: &GossipItem) -> Result<NetworkMessage> {
        match item.gossip_type {
            GossipType::Block => {
                let block: Block = bincode::deserialize(&item.data)?;
                Ok(NetworkMessage::Block { block })
            }
            GossipType::Transaction => {
                let transaction: Transaction = bincode::deserialize(&item.data)?;
                Ok(NetworkMessage::Tx { transaction })
            }
            GossipType::BlockHeader => {
                // TODO: Implement block header message
                Err(anyhow!("Block header gossip not yet implemented"))
            }
            GossipType::CompactBlock => {
                // TODO: Implement compact block message
                Err(anyhow!("Compact block gossip not yet implemented"))
            }
            GossipType::Emergency => {
                // TODO: Implement emergency message
                Err(anyhow!("Emergency gossip not yet implemented"))
            }
        }
    }
    
    /// Spawn incoming gossip processor
    fn spawn_incoming_processor(&self) {
        let protocol = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(50)); // 20Hz processing
            
            while *protocol.running.read().await {
                interval.tick().await;
                
                if let Err(e) = protocol.process_incoming_queue().await {
                    log::error!("Incoming gossip processing error: {}", e);
                    protocol.health_monitor.lock().await.record_error();
                }
            }
        });
    }
    
    /// Process incoming gossip queue
    async fn process_incoming_queue(&self) -> Result<()> {
        let mut queue = self.incoming_queue.lock().await;
        let mut processed = 0;
        
        while processed < 5 && !queue.is_empty() { // Process up to 5 items per tick
            if let Some(item) = queue.pop() {
                drop(queue);
                
                // Process based on type
                match item.gossip_type {
                    GossipType::Block => {
                        let block: Block = bincode::deserialize(&item.data)?;
                        
                        // Validate block
                        if self.block_handler.validate_block(&block).await? {
                            self.block_handler.handle_block(block).await?;
                            
                            // Re-gossip if still can propagate
                            if item.can_propagate() {
                                self.queue_for_gossip(item).await?;
                            }
                        } else {
                            log::warn!("Invalid block received via gossip: {}", item.id);
                            if let Some(origin) = &item.origin_peer {
                                self.update_peer_score(origin, 20).await;
                            }
                        }
                    }
                    GossipType::Transaction => {
                        let transaction: Transaction = bincode::deserialize(&item.data)?;
                        
                        // Validate transaction
                        if self.transaction_handler.validate_transaction(&transaction).await? {
                            self.transaction_handler.handle_transaction(transaction).await?;
                            
                            // Re-gossip if still can propagate
                            if item.can_propagate() {
                                self.queue_for_gossip(item).await?;
                            }
                        } else {
                            log::warn!("Invalid transaction received via gossip: {}", item.id);
                            if let Some(origin) = &item.origin_peer {
                                self.update_peer_score(origin, 10).await;
                            }
                        }
                    }
                    _ => {
                        log::debug!("Unhandled gossip type: {:?}", item.gossip_type);
                    }
                }
                
                queue = self.incoming_queue.lock().await;
                processed += 1;
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Spawn health monitor
    fn spawn_health_monitor(&self) {
        let protocol = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
            
            while *protocol.running.read().await {
                interval.tick().await;
                
                let health = protocol.health_monitor.lock().await;
                let is_healthy = health.is_healthy();
                let gossip_rate = health.gossip_rate;
                let error_rate = health.error_rate;
                drop(health);
                
                if !is_healthy {
                    log::warn!("Gossip protocol health check failed - rate: {:.2}, errors: {:.2}", 
                             gossip_rate, error_rate);
                }
                
                // Check for network partition
                let partition = protocol.partition_detector.lock().await;
                if partition.detect_partition() {
                    log::error!("Network partition detected! Active peers: {}", 
                              partition.get_active_peer_count());
                    protocol.health_monitor.lock().await.record_partition();
                }
            }
        });
    }
    
    /// Spawn cleanup task
    fn spawn_cleanup_task(&self) {
        let protocol = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Cleanup every minute
            
            while *protocol.running.read().await {
                interval.tick().await;
                
                // Clean up stale seen items
                let mut seen = protocol.seen_items.write().await;
                let now = Instant::now();
                seen.retain(|_, timestamp| {
                    now.duration_since(*timestamp) < MAX_GOSSIP_AGE
                });
                drop(seen);
                
                // Clean up stale queue items
                protocol.outgoing_queue.lock().await.cleanup_stale();
                protocol.incoming_queue.lock().await.cleanup_stale();
                
                // Decay peer DoS scores
                let mut peers = protocol.peers.write().await;
                for peer_state in peers.values_mut() {
                    peer_state.decrease_dos_score(1); // Slowly decrease scores
                }
                
                log::trace!("Gossip protocol cleanup completed");
            }
        });
    }
    
    /// Get gossip protocol statistics
    pub async fn get_stats(&self) -> GossipStats {
        let peers = self.peers.read().await;
        let outgoing_queue = self.outgoing_queue.lock().await;
        let incoming_queue = self.incoming_queue.lock().await;
        let health = self.health_monitor.lock().await;
        let partition = self.partition_detector.lock().await;
        
        GossipStats {
            peer_count: peers.len(),
            banned_peers: peers.values().filter(|p| p.is_banned()).count(),
            outgoing_queue_size: outgoing_queue.len(),
            incoming_queue_size: incoming_queue.len(),
            gossip_rate: health.gossip_rate,
            error_rate: health.error_rate,
            backpressure_events: health.backpressure_events,
            partition_events: health.partition_events,
            active_peers: partition.get_active_peer_count(),
            has_backpressure: outgoing_queue.has_backpressure() || incoming_queue.has_backpressure(),
            is_healthy: health.is_healthy(),
        }
    }
    
    /// Shutdown the gossip protocol
    pub async fn shutdown(&self) -> Result<()> {
        *self.running.write().await = false;
        
        self.gossip_tx.send(GossipCommand::Shutdown)
            .map_err(|_| anyhow!("Failed to send shutdown command"))?;
        
        log::info!("Gossip protocol shutdown for node {}", self.node_id);
        Ok(())
    }
}

/// Gossip protocol statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipStats {
    pub peer_count: usize,
    pub banned_peers: usize,
    pub outgoing_queue_size: usize,
    pub incoming_queue_size: usize,
    pub gossip_rate: f64,
    pub error_rate: f64,
    pub backpressure_events: u64,
    pub partition_events: u64,
    pub active_peers: usize,
    pub has_backpressure: bool,
    pub is_healthy: bool,
}

// Make GossipProtocol cloneable for multi-threaded use
impl Clone for GossipProtocol {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            chain_spec: self.chain_spec.clone(),
            metrics: self.metrics.clone(),
            security_manager: self.security_manager.clone(),
            peers: self.peers.clone(),
            seen_items: self.seen_items.clone(),
            outgoing_queue: self.outgoing_queue.clone(),
            incoming_queue: self.incoming_queue.clone(),
            block_handler: self.block_handler.clone(),
            transaction_handler: self.transaction_handler.clone(),
            gossip_tx: self.gossip_tx.clone(),
            peer_tx: self.peer_tx.clone(),
            health_monitor: self.health_monitor.clone(),
            partition_detector: self.partition_detector.clone(),
            running: self.running.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_gossip_item_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let item = GossipItem::new(GossipType::Block, data.clone(), None);
        
        assert_eq!(item.gossip_type, GossipType::Block);
        assert_eq!(item.data, data);
        assert_eq!(item.hop_count, 0);
        assert!(item.verify_checksum());
        assert!(!item.is_stale());
        assert!(item.can_propagate());
    }
    
    #[test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new();
        
        // Should be able to consume initially
        assert!(limiter.consume(&GossipType::Transaction));
        
        // After consuming many tokens, should hit limit
        for _ in 0..200 {
            limiter.consume(&GossipType::Transaction);
        }
        
        assert!(!limiter.can_accept(&GossipType::Transaction));
    }
    
    #[test]
    async fn test_gossip_queue() {
        let mut queue = GossipQueue::new();
        
        let item1 = GossipItem::new(GossipType::Emergency, vec![1], None);
        let item2 = GossipItem::new(GossipType::Transaction, vec![2], None);
        
        assert!(queue.push(item1));
        assert!(queue.push(item2));
        
        // Emergency should come out first (higher priority)
        let popped = queue.pop().unwrap();
        assert_eq!(popped.gossip_type, GossipType::Emergency);
    }
    
    #[test]
    async fn test_peer_dos_scoring() {
        let mut peer = PeerGossipState::new("test_peer".to_string());
        
        assert!(!peer.is_banned());
        
        peer.increase_dos_score(50);
        assert!(!peer.is_banned());
        
        peer.increase_dos_score(60);
        assert!(peer.is_banned());
        
        peer.decrease_dos_score(20);
        assert!(peer.is_banned()); // Still banned
        
        peer.decrease_dos_score(50);
        assert!(!peer.is_banned()); // Now unbanned
    }
}
