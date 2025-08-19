use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    block::Block,
    blockchain::Blockchain,
    database::BlockchainDatabase,
    mempool::Mempool,
    transaction::SignedTransaction,
};

/// P2P protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Magic bytes for QuantumCoin network
pub const MAGIC_BYTES: [u8; 4] = [0x51, 0x54, 0x43, 0x4D]; // "QTCM"

/// Maximum peers to connect to
pub const MAX_PEERS: usize = 8;

/// Connection timeout
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Ping interval
pub const PING_INTERVAL: Duration = Duration::from_secs(30);

/// Peer timeout (no activity)
pub const PEER_TIMEOUT: Duration = Duration::from_secs(120);

/// P2P message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// Version handshake
    Version,
    VerAck,
    
    /// Ping/Pong for keepalive
    Ping,
    Pong,
    
    /// Block and transaction propagation
    NewBlock,
    NewTransaction,
    GetBlocks,
    BlockResponse,
    GetMempool,
    MempoolResponse,
    
    /// Peer discovery
    GetPeers,
    PeersResponse,
    
    /// Blockchain synchronization
    GetHeaders,
    HeadersResponse,
    GetBlock,
}

/// P2P network message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub magic: [u8; 4],
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub checksum: u32,
}

impl P2PMessage {
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let checksum = crc32fast::hash(&payload);
        
        Self {
            magic: MAGIC_BYTES,
            message_type,
            payload,
            timestamp,
            checksum,
        }
    }
    
    pub fn verify_checksum(&self) -> bool {
        crc32fast::hash(&self.payload) == self.checksum
    }
    
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).context("Failed to serialize P2P message")
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).context("Failed to deserialize P2P message")
    }
}

/// Version message for handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMessage {
    pub protocol_version: u32,
    pub services: u64,
    pub timestamp: u64,
    pub user_agent: String,
    pub start_height: u64,
    pub relay: bool,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub version: Option<VersionMessage>,
    pub last_seen: SystemTime,
    pub connected_at: SystemTime,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_outbound: bool,
}

impl PeerInfo {
    pub fn new(address: SocketAddr, is_outbound: bool) -> Self {
        let now = SystemTime::now();
        Self {
            address,
            version: None,
            last_seen: now,
            connected_at: now,
            bytes_sent: 0,
            bytes_received: 0,
            is_outbound,
        }
    }
    
    pub fn is_timeout(&self) -> bool {
        SystemTime::now().duration_since(self.last_seen).unwrap_or_default() > PEER_TIMEOUT
    }
}

/// P2P Network Node
pub struct P2PNode {
    /// Local listening address
    listen_addr: SocketAddr,
    
    /// Connected peers
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    
    /// Known peer addresses for discovery
    known_peers: Arc<RwLock<HashSet<SocketAddr>>>,
    
    /// Message channels
    message_tx: mpsc::UnboundedSender<(SocketAddr, P2PMessage)>,
    message_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<(SocketAddr, P2PMessage)>>>>,
    
    /// Blockchain reference
    blockchain: Arc<RwLock<Blockchain>>,
    
    /// Database reference
    database: Arc<RwLock<Option<BlockchainDatabase>>>,
    
    /// Mempool reference
    mempool: Arc<RwLock<Mempool>>,
    
    /// Node ID
    node_id: Uuid,
    
    /// Running state
    is_running: Arc<RwLock<bool>>,
}

impl P2PNode {
    pub fn new(
        listen_addr: SocketAddr,
        blockchain: Arc<RwLock<Blockchain>>,
        mempool: Arc<RwLock<Mempool>>,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        Self {
            listen_addr,
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_peers: Arc::new(RwLock::new(HashSet::new())),
            message_tx,
            message_rx: Arc::new(RwLock::new(Some(message_rx))),
            blockchain,
            database: Arc::new(RwLock::new(None)),
            mempool,
            node_id: Uuid::new_v4(),
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    pub async fn set_database(&self, database: BlockchainDatabase) {
        let mut db_guard = self.database.write().await;
        *db_guard = Some(database);
    }
    
    /// Start the P2P node
    pub async fn start(&self) -> Result<()> {
        info!("Starting P2P node on {}", self.listen_addr);
        
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }
        
        // Start listening for incoming connections
        let listener = TcpListener::bind(self.listen_addr).await
            .context("Failed to bind TCP listener")?;
        
        // Start background tasks
        self.start_message_handler().await;
        self.start_peer_maintenance().await;
        
        // Accept incoming connections
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Incoming connection from {}", addr);
                    self.handle_incoming_connection(stream, addr).await;
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
            
            // Check if we should stop
            let running = *self.is_running.read().await;
            if !running {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Stop the P2P node
    pub async fn stop(&self) {
        info!("Stopping P2P node");
        let mut running = self.is_running.write().await;
        *running = false;
    }
    
    /// Connect to a peer
    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        info!("Connecting to peer {}", addr);
        
        // Check if already connected
        {
            let peers = self.peers.read().await;
            if peers.contains_key(&addr) {
                debug!("Already connected to {}", addr);
                return Ok(());
            }
            
            if peers.len() >= MAX_PEERS {
                debug!("Max peers reached, not connecting to {}", addr);
                return Ok(());
            }
        }
        
        // Connect with timeout
        match timeout(CONNECTION_TIMEOUT, TcpStream::connect(addr)).await {
            Ok(Ok(stream)) => {
                info!("Connected to peer {}", addr);
                self.handle_outgoing_connection(stream, addr).await;
                Ok(())
            }
            Ok(Err(e)) => {
                warn!("Failed to connect to {}: {}", addr, e);
                Err(e.into())
            }
            Err(_) => {
                warn!("Connection to {} timed out", addr);
                Err(anyhow::anyhow!("Connection timeout"))
            }
        }
    }
    
    /// Add known peer addresses
    pub async fn add_known_peers(&self, peers: &[SocketAddr]) {
        let mut known_peers = self.known_peers.write().await;
        for &peer in peers {
            if peer != self.listen_addr {
                known_peers.insert(peer);
            }
        }
        info!("Added {} known peers", peers.len());
    }
    
    /// Get connected peer count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
    
    /// Broadcast a message to all peers
    pub async fn broadcast(&self, message_type: MessageType, payload: Vec<u8>) {
        let message = P2PMessage::new(message_type, payload);
        let peers: Vec<SocketAddr> = {
            let peers_guard = self.peers.read().await;
            peers_guard.keys().copied().collect()
        };
        
        debug!("Broadcasting {:?} to {} peers", message.message_type, peers.len());
        
        for peer_addr in peers {
            if let Err(e) = self.message_tx.send((peer_addr, message.clone())) {
                error!("Failed to send message to {}: {}", peer_addr, e);
            }
        }
    }
    
    /// Broadcast new block
    pub async fn broadcast_block(&self, block: &Block) -> Result<()> {
        let payload = bincode::serialize(block)?;
        self.broadcast(MessageType::NewBlock, payload).await;
        Ok(())
    }
    
    /// Broadcast new transaction
    pub async fn broadcast_transaction(&self, transaction: &SignedTransaction) -> Result<()> {
        let payload = bincode::serialize(transaction)?;
        self.broadcast(MessageType::NewTransaction, payload).await;
        Ok(())
    }
    
    /// Handle incoming connection
    async fn handle_incoming_connection(&self, stream: TcpStream, addr: SocketAddr) {
        let peer_info = PeerInfo::new(addr, false);
        
        {
            let mut peers = self.peers.write().await;
            peers.insert(addr, peer_info);
        }
        
        // TODO: Handle connection protocol
        // For now, just add to peers list
        debug!("Added incoming peer {}", addr);
    }
    
    /// Handle outgoing connection
    async fn handle_outgoing_connection(&self, stream: TcpStream, addr: SocketAddr) {
        let peer_info = PeerInfo::new(addr, true);
        
        {
            let mut peers = self.peers.write().await;
            peers.insert(addr, peer_info);
        }
        
        // Send version handshake
        if let Err(e) = self.send_version_handshake(addr).await {
            error!("Failed to send version to {}: {}", addr, e);
        }
        
        debug!("Added outgoing peer {}", addr);
    }
    
    /// Send version handshake
    async fn send_version_handshake(&self, addr: SocketAddr) -> Result<()> {
        let blockchain_height = {
            let blockchain = self.blockchain.read().await;
            blockchain.chain.len() as u64
        };
        
        let version_msg = VersionMessage {
            protocol_version: PROTOCOL_VERSION,
            services: 1, // NODE_NETWORK
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            user_agent: "QuantumCoin/2.0".to_string(),
            start_height: blockchain_height,
            relay: true,
        };
        
        let payload = bincode::serialize(&version_msg)?;
        let message = P2PMessage::new(MessageType::Version, payload);
        
        self.message_tx.send((addr, message))?;
        Ok(())
    }
    
    /// Start message handler task
    async fn start_message_handler(&self) {
        let message_rx = {
            let mut rx_guard = self.message_rx.write().await;
            rx_guard.take()
        };
        
        if let Some(mut message_rx) = message_rx {
            let blockchain = Arc::clone(&self.blockchain);
            let mempool = Arc::clone(&self.mempool);
            let database = Arc::clone(&self.database);
            
            tokio::spawn(async move {
                while let Some((addr, message)) = message_rx.recv().await {
                    if let Err(e) = Self::handle_message(addr, message, &blockchain, &mempool, &database).await {
                        error!("Error handling message from {}: {}", addr, e);
                    }
                }
            });
        }
    }
    
    /// Handle received P2P message
    async fn handle_message(
        addr: SocketAddr,
        message: P2PMessage,
        blockchain: &Arc<RwLock<Blockchain>>,
        mempool: &Arc<RwLock<Mempool>>,
        database: &Arc<RwLock<Option<BlockchainDatabase>>>,
    ) -> Result<()> {
        if !message.verify_checksum() {
            warn!("Invalid checksum from {}", addr);
            return Ok(());
        }
        
        debug!("Received {:?} from {}", message.message_type, addr);
        
        match message.message_type {
            MessageType::Version => {
                let version_msg: VersionMessage = bincode::deserialize(&message.payload)?;
                info!("Peer {} version: {}", addr, version_msg.user_agent);
                // TODO: Send VerAck
            }
            
            MessageType::NewBlock => {
                let block: Block = bincode::deserialize(&message.payload)?;
                info!("Received new block {} from {}", block.hash, addr);
                
                // Validate and add block
                let mut blockchain_guard = blockchain.write().await;
                if let Err(e) = blockchain_guard.add_block(block.clone()) {
                    warn!("Failed to add block from {}: {}", addr, e);
                } else {
                    // Store in database if available
                    let db_guard = database.read().await;
                    if let Some(db) = db_guard.as_ref() {
                        // TODO: Extract transactions from block
                        if let Err(e) = db.store_block(&block, &[]).await {
                            error!("Failed to store block in database: {}", e);
                        }
                    }
                }
            }
            
            MessageType::NewTransaction => {
                let transaction: SignedTransaction = bincode::deserialize(&message.payload)?;
                info!("Received new transaction {} from {}", transaction.id, addr);
                
                // Add to mempool
                let mut mempool_guard = mempool.write().await;
                if let Err(e) = mempool_guard.add_transaction(transaction) {
                    warn!("Failed to add transaction from {}: {}", addr, e);
                }
            }
            
            MessageType::GetBlocks => {
                // TODO: Send blocks to peer
                debug!("Peer {} requested blocks", addr);
            }
            
            MessageType::Ping => {
                // TODO: Send Pong response
                debug!("Ping from {}", addr);
            }
            
            _ => {
                debug!("Unhandled message type {:?} from {}", message.message_type, addr);
            }
        }
        
        Ok(())
    }
    
    /// Start peer maintenance task
    async fn start_peer_maintenance(&self) {
        let peers = Arc::clone(&self.peers);
        let known_peers = Arc::clone(&self.known_peers);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let running = *is_running.read().await;
                if !running {
                    break;
                }
                
                // Remove timed out peers
                let mut peers_guard = peers.write().await;
                let timed_out: Vec<SocketAddr> = peers_guard
                    .iter()
                    .filter(|(_, peer)| peer.is_timeout())
                    .map(|(&addr, _)| addr)
                    .collect();
                
                for addr in timed_out {
                    info!("Removing timed out peer {}", addr);
                    peers_guard.remove(&addr);
                }
                
                // Try to connect to more peers if needed
                let peer_count = peers_guard.len();
                drop(peers_guard);
                
                if peer_count < MAX_PEERS {
                    let known_peers_guard = known_peers.read().await;
                    let available_peers: Vec<SocketAddr> = known_peers_guard
                        .iter()
                        .filter(|&&addr| {
                            let peers_guard = futures::executor::block_on(peers.read());
                            !peers_guard.contains_key(&addr)
                        })
                        .take(MAX_PEERS - peer_count)
                        .copied()
                        .collect();
                    drop(known_peers_guard);
                    
                    for addr in available_peers {
                        info!("Trying to connect to discovered peer {}", addr);
                        // TODO: Connect to peer (would need self reference)
                        break; // For now, just try one
                    }
                }
            }
        });
    }
    
    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        let peers_guard = self.peers.read().await;
        let known_peers_guard = self.known_peers.read().await;
        
        NetworkStats {
            connected_peers: peers_guard.len(),
            known_peers: known_peers_guard.len(),
            inbound_peers: peers_guard.values().filter(|p| !p.is_outbound).count(),
            outbound_peers: peers_guard.values().filter(|p| p.is_outbound).count(),
            total_bytes_sent: peers_guard.values().map(|p| p.bytes_sent).sum(),
            total_bytes_received: peers_guard.values().map(|p| p.bytes_received).sum(),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub known_peers: usize,
    pub inbound_peers: usize,
    pub outbound_peers: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_p2p_message_serialization() {
        let message = P2PMessage::new(MessageType::Ping, vec![1, 2, 3, 4]);
        
        assert!(message.verify_checksum());
        
        let serialized = message.serialize().unwrap();
        let deserialized = P2PMessage::deserialize(&serialized).unwrap();
        
        assert_eq!(message.message_type, deserialized.message_type);
        assert_eq!(message.payload, deserialized.payload);
        assert!(deserialized.verify_checksum());
    }
    
    #[tokio::test]
    async fn test_version_message() {
        let version = VersionMessage {
            protocol_version: PROTOCOL_VERSION,
            services: 1,
            timestamp: 12345,
            user_agent: "Test/1.0".to_string(),
            start_height: 100,
            relay: true,
        };
        
        let payload = bincode::serialize(&version).unwrap();
        let message = P2PMessage::new(MessageType::Version, payload);
        
        assert!(message.verify_checksum());
    }
    
    #[tokio::test]
    async fn test_peer_info() {
        let addr = "127.0.0.1:8333".parse().unwrap();
        let peer = PeerInfo::new(addr, true);
        
        assert_eq!(peer.address, addr);
        assert!(peer.is_outbound);
        assert!(!peer.is_timeout());
    }
}
