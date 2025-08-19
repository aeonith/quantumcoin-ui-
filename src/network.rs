use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};

use crate::blockchain::{Blockchain, Block};
use crate::transaction::{Transaction, SignedTransaction};

const PROTOCOL_VERSION: u32 = 1;
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const HEARTBEAT_INTERVAL: u64 = 30; // seconds
const PEER_TIMEOUT: u64 = 120; // seconds

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Version {
        version: u32,
        timestamp: DateTime<Utc>,
        best_height: u64,
        node_id: String,
    },
    VerAck,
    GetBlocks {
        start_hash: String,
        end_hash: String,
    },
    Block(Block),
    GetMempool,
    Transaction(SignedTransaction),
    Ping(u64),
    Pong(u64),
    GetPeers,
    Peers(Vec<SocketAddr>),
    Disconnect(String),
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub node_id: String,
    pub version: u32,
    pub best_height: u64,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_outbound: bool,
}

pub struct NetworkNode {
    listen_addr: SocketAddr,
    blockchain: Blockchain,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
    known_peers: Arc<RwLock<HashSet<SocketAddr>>>,
    node_id: String,
    message_sender: Option<mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>>,
    is_running: Arc<RwLock<bool>>,
}

impl NetworkNode {
    pub fn new(listen_addr: SocketAddr, blockchain: Blockchain) -> Self {
        let node_id = format!("node_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        
        Self {
            listen_addr,
            blockchain,
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_peers: Arc::new(RwLock::new(HashSet::new())),
            node_id,
            message_sender: None,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(anyhow!("Network node is already running"));
        }
        *is_running = true;
        drop(is_running);

        info!("Starting network node {} on {}", self.node_id, self.listen_addr);

        // Create message channel
        let (tx, mut rx) = mpsc::unbounded_channel::<(SocketAddr, NetworkMessage)>();
        self.message_sender = Some(tx);

        // Start listening for connections
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!("Listening for connections on {}", self.listen_addr);

        // Clone data for tasks
        let peers = Arc::clone(&self.peers);
        let known_peers = Arc::clone(&self.known_peers);
        let node_id = self.node_id.clone();
        let blockchain = self.blockchain.clone();

        // Start connection acceptor
        let accept_peers = Arc::clone(&peers);
        let accept_node_id = node_id.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("Accepted connection from {}", addr);
                        let peer_info = PeerInfo {
                            address: addr,
                            node_id: String::new(),
                            version: 0,
                            best_height: 0,
                            connected_at: Utc::now(),
                            last_seen: Utc::now(),
                            is_outbound: false,
                        };
                        
                        {
                            let mut peers_write = accept_peers.write().await;
                            peers_write.insert(addr, peer_info);
                        }
                        
                        let handle_peers = Arc::clone(&accept_peers);
                        let handle_node_id = accept_node_id.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_peer_connection(
                                stream, 
                                addr, 
                                handle_peers, 
                                handle_node_id,
                                blockchain.clone()
                            ).await {
                                error!("Peer connection error {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        // Start message processor
        let process_peers = Arc::clone(&peers);
        let process_known_peers = Arc::clone(&known_peers);
        tokio::spawn(async move {
            while let Some((sender_addr, message)) = rx.recv().await {
                Self::process_network_message(
                    sender_addr, 
                    message, 
                    Arc::clone(&process_peers),
                    Arc::clone(&process_known_peers)
                ).await;
            }
        });

        // Start heartbeat task
        let heartbeat_peers = Arc::clone(&peers);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(HEARTBEAT_INTERVAL));
            loop {
                interval.tick().await;
                Self::send_heartbeats(Arc::clone(&heartbeat_peers)).await;
            }
        });

        // Start peer cleanup task
        let cleanup_peers = Arc::clone(&peers);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(PEER_TIMEOUT));
            loop {
                interval.tick().await;
                Self::cleanup_stale_peers(Arc::clone(&cleanup_peers)).await;
            }
        });

        Ok(())
    }

    pub async fn connect_to_peer(&self, peer_addr: SocketAddr) -> Result<()> {
        if peer_addr == self.listen_addr {
            return Err(anyhow!("Cannot connect to self"));
        }

        let peers = self.peers.read().await;
        if peers.contains_key(&peer_addr) {
            return Err(anyhow!("Already connected to peer"));
        }
        drop(peers);

        info!("Connecting to peer: {}", peer_addr);

        let stream = TcpStream::connect(peer_addr).await?;
        
        let peer_info = PeerInfo {
            address: peer_addr,
            node_id: String::new(),
            version: 0,
            best_height: 0,
            connected_at: Utc::now(),
            last_seen: Utc::now(),
            is_outbound: true,
        };

        {
            let mut peers_write = self.peers.write().await;
            peers_write.insert(peer_addr, peer_info);
        }

        // Add to known peers
        {
            let mut known_peers_write = self.known_peers.write().await;
            known_peers_write.insert(peer_addr);
        }

        let handle_peers = Arc::clone(&self.peers);
        let handle_node_id = self.node_id.clone();
        let handle_blockchain = self.blockchain.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::handle_peer_connection(
                stream, 
                peer_addr, 
                handle_peers, 
                handle_node_id,
                handle_blockchain
            ).await {
                error!("Outbound peer connection error {}: {}", peer_addr, e);
            }
        });

        Ok(())
    }

    async fn handle_peer_connection(
        mut stream: TcpStream,
        peer_addr: SocketAddr,
        peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
        node_id: String,
        blockchain: Blockchain,
    ) -> Result<()> {
        // Send version message
        let version_msg = NetworkMessage::Version {
            version: PROTOCOL_VERSION,
            timestamp: Utc::now(),
            best_height: blockchain.chain.len() as u64,
            node_id: node_id.clone(),
        };
        
        Self::send_message(&mut stream, &version_msg).await?;

        // Handle incoming messages
        loop {
            match Self::receive_message(&mut stream).await {
                Ok(message) => {
                    debug!("Received message from {}: {:?}", peer_addr, message);
                    
                    // Update last seen time
                    {
                        let mut peers_write = peers.write().await;
                        if let Some(peer_info) = peers_write.get_mut(&peer_addr) {
                            peer_info.last_seen = Utc::now();
                        }
                    }

                    // Process the message
                    match message {
                        NetworkMessage::Version { version, best_height, node_id: peer_node_id, .. } => {
                            // Send version acknowledgment
                            Self::send_message(&mut stream, &NetworkMessage::VerAck).await?;
                            
                            // Update peer info
                            {
                                let mut peers_write = peers.write().await;
                                if let Some(peer_info) = peers_write.get_mut(&peer_addr) {
                                    peer_info.version = version;
                                    peer_info.best_height = best_height;
                                    peer_info.node_id = peer_node_id;
                                }
                            }
                        }
                        NetworkMessage::VerAck => {
                            info!("Version handshake completed with {}", peer_addr);
                        }
                        NetworkMessage::Ping(nonce) => {
                            Self::send_message(&mut stream, &NetworkMessage::Pong(nonce)).await?;
                        }
                        NetworkMessage::Pong(_) => {
                            debug!("Received pong from {}", peer_addr);
                        }
                        NetworkMessage::GetBlocks { .. } => {
                            // TODO: Send blocks
                            warn!("GetBlocks not implemented");
                        }
                        NetworkMessage::Block(block) => {
                            info!("Received block {} from {}", block.hash, peer_addr);
                            // TODO: Validate and add block
                        }
                        NetworkMessage::Transaction(tx) => {
                            info!("Received transaction {} from {}", tx.id, peer_addr);
                            // TODO: Validate and add to mempool
                        }
                        NetworkMessage::GetMempool => {
                            // TODO: Send mempool transactions
                            warn!("GetMempool not implemented");
                        }
                        NetworkMessage::GetPeers => {
                            let known_peers = {
                                let peers_read = peers.read().await;
                                peers_read.keys().copied().collect::<Vec<_>>()
                            };
                            Self::send_message(&mut stream, &NetworkMessage::Peers(known_peers)).await?;
                        }
                        NetworkMessage::Peers(peer_addrs) => {
                            info!("Received {} peer addresses from {}", peer_addrs.len(), peer_addr);
                            // TODO: Connect to new peers
                        }
                        NetworkMessage::Disconnect(reason) => {
                            info!("Peer {} disconnected: {}", peer_addr, reason);
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive message from {}: {}", peer_addr, e);
                    break;
                }
            }
        }

        // Remove peer on disconnect
        {
            let mut peers_write = peers.write().await;
            peers_write.remove(&peer_addr);
        }

        info!("Disconnected from peer: {}", peer_addr);
        Ok(())
    }

    async fn send_message(stream: &mut TcpStream, message: &NetworkMessage) -> Result<()> {
        let serialized = bincode::serialize(message)?;
        let length = serialized.len() as u32;
        
        if length > MAX_MESSAGE_SIZE as u32 {
            return Err(anyhow!("Message too large: {} bytes", length));
        }

        // Send length header
        stream.write_all(&length.to_le_bytes()).await?;
        // Send message data
        stream.write_all(&serialized).await?;
        stream.flush().await?;
        
        Ok(())
    }

    async fn receive_message(stream: &mut TcpStream) -> Result<NetworkMessage> {
        // Read length header
        let mut length_bytes = [0u8; 4];
        stream.read_exact(&mut length_bytes).await?;
        let length = u32::from_le_bytes(length_bytes) as usize;

        if length > MAX_MESSAGE_SIZE {
            return Err(anyhow!("Message too large: {} bytes", length));
        }

        // Read message data
        let mut buffer = vec![0u8; length];
        stream.read_exact(&mut buffer).await?;

        let message: NetworkMessage = bincode::deserialize(&buffer)?;
        Ok(message)
    }

    async fn send_heartbeats(peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>) {
        let peer_addrs: Vec<SocketAddr> = {
            let peers_read = peers.read().await;
            peers_read.keys().copied().collect()
        };

        for _peer_addr in peer_addrs {
            // TODO: Send ping to each peer
            // This would require maintaining connection handles
        }
    }

    async fn cleanup_stale_peers(peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>) {
        let now = Utc::now();
        let timeout_threshold = now - chrono::Duration::seconds(PEER_TIMEOUT as i64);
        
        let stale_peers: Vec<SocketAddr> = {
            let peers_read = peers.read().await;
            peers_read
                .iter()
                .filter(|(_, info)| info.last_seen < timeout_threshold)
                .map(|(addr, _)| *addr)
                .collect()
        };

        if !stale_peers.is_empty() {
            let mut peers_write = peers.write().await;
            for addr in stale_peers {
                info!("Removing stale peer: {}", addr);
                peers_write.remove(&addr);
            }
        }
    }

    async fn process_network_message(
        _sender_addr: SocketAddr,
        _message: NetworkMessage,
        _peers: Arc<RwLock<HashMap<SocketAddr, PeerInfo>>>,
        _known_peers: Arc<RwLock<HashSet<SocketAddr>>>,
    ) {
        // TODO: Process messages received via channel
    }

    pub async fn broadcast_block(&self, block: &Block) -> Result<()> {
        info!("Broadcasting block {} to peers", block.hash);
        
        let message = NetworkMessage::Block(block.clone());
        let peer_count = {
            let peers_read = self.peers.read().await;
            peers_read.len()
        };

        // TODO: Implement actual broadcasting
        // This would require maintaining connection handles for each peer
        
        info!("Block broadcast initiated to {} peers", peer_count);
        Ok(())
    }

    pub async fn broadcast_transaction(&self, transaction: &SignedTransaction) -> Result<()> {
        info!("Broadcasting transaction {} to peers", transaction.id);
        
        let _message = NetworkMessage::Transaction(transaction.clone());
        let peer_count = {
            let peers_read = self.peers.read().await;
            peers_read.len()
        };

        // TODO: Implement actual broadcasting
        
        info!("Transaction broadcast initiated to {} peers", peer_count);
        Ok(())
    }

    pub async fn get_peer_count(&self) -> usize {
        let peers_read = self.peers.read().await;
        peers_read.len()
    }

    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        let peers_read = self.peers.read().await;
        peers_read.values().cloned().collect()
    }

    pub async fn disconnect_peer(&self, peer_addr: SocketAddr, reason: String) -> Result<()> {
        let mut peers_write = self.peers.write().await;
        if peers_write.remove(&peer_addr).is_some() {
            info!("Disconnected from peer {}: {}", peer_addr, reason);
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        // Disconnect all peers
        let peer_addrs: Vec<SocketAddr> = {
            let peers_read = self.peers.read().await;
            peers_read.keys().copied().collect()
        };

        for peer_addr in peer_addrs {
            let _ = self.disconnect_peer(peer_addr, "Node shutting down".to_string()).await;
        }

        info!("Network node stopped");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub known_peers: usize,
    pub node_id: String,
    pub listen_address: SocketAddr,
    pub is_running: bool,
}

impl NetworkNode {
    pub async fn get_network_stats(&self) -> NetworkStats {
        let peers_read = self.peers.read().await;
        let known_peers_read = self.known_peers.read().await;
        let is_running = self.is_running.read().await;

        NetworkStats {
            connected_peers: peers_read.len(),
            known_peers: known_peers_read.len(),
            node_id: self.node_id.clone(),
            listen_address: self.listen_addr,
            is_running: *is_running,
        }
    }
}
