use crate::{Blockchain, Transaction, Block};
use crate::network::{NetworkMessage, Peer, PeerInfo};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, warn, error, debug};

pub struct NetworkNode {
    pub node_id: String,
    pub version: u32,
    pub listen_addr: SocketAddr,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub peers: Arc<RwLock<HashMap<SocketAddr, Peer>>>,
    pub mempool: Arc<RwLock<Vec<Transaction>>>,
    pub known_peers: Arc<RwLock<Vec<SocketAddr>>>,
    pub message_tx: mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>,
    pub message_rx: Option<mpsc::UnboundedReceiver<(SocketAddr, NetworkMessage)>>,
}

impl NetworkNode {
    pub fn new(listen_addr: SocketAddr, blockchain: Blockchain) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        Self {
            node_id: Uuid::new_v4().to_string(),
            version: 1,
            listen_addr,
            blockchain: Arc::new(RwLock::new(blockchain)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            mempool: Arc::new(RwLock::new(Vec::new())),
            known_peers: Arc::new(RwLock::new(Vec::new())),
            message_tx,
            message_rx: Some(message_rx),
        }
    }
    
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting network node on {}", self.listen_addr);
        
        // Start listening for incoming connections
        let listener = TcpListener::bind(self.listen_addr).await?;
        let peers = Arc::clone(&self.peers);
        let blockchain = Arc::clone(&self.blockchain);
        let mempool = Arc::clone(&self.mempool);
        let message_tx = self.message_tx.clone();
        let node_id = self.node_id.clone();
        let version = self.version;
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New connection from {}", addr);
                        let peers = Arc::clone(&peers);
                        let blockchain = Arc::clone(&blockchain);
                        let mempool = Arc::clone(&mempool);
                        let message_tx = message_tx.clone();
                        let node_id = node_id.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream, addr, peers, blockchain, mempool, message_tx, &node_id, version
                            ).await {
                                error!("Error handling connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });
        
        // Start message processing loop
        let mut message_rx = self.message_rx.take().unwrap();
        let peers = Arc::clone(&self.peers);
        let blockchain = Arc::clone(&self.blockchain);
        let mempool = Arc::clone(&self.mempool);
        
        tokio::spawn(async move {
            while let Some((addr, message)) = message_rx.recv().await {
                if let Err(e) = Self::process_message(
                    addr, message, &peers, &blockchain, &mempool
                ).await {
                    error!("Error processing message from {}: {}", addr, e);
                }
            }
        });
        
        // Start peer maintenance loop
        self.start_peer_maintenance().await;
        
        Ok(())
    }
    
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        peers: Arc<RwLock<HashMap<SocketAddr, Peer>>>,
        blockchain: Arc<RwLock<Blockchain>>,
        mempool: Arc<RwLock<Vec<Transaction>>>,
        message_tx: mpsc::UnboundedSender<(SocketAddr, NetworkMessage)>,
        node_id: &str,
        version: u32,
    ) -> Result<()> {
        let mut peer = Peer::new(addr);
        peer.stream = Some(stream);
        peer.info.connected = true;
        
        // Perform handshake
        let blockchain_read = blockchain.read().await;
        let chain_height = blockchain_read.chain.len() as u64;
        drop(blockchain_read);
        
        if peer.handshake(version, node_id, chain_height).await? {
            info!("Handshake successful with {}", addr);
            
            // Add to peers
            {
                let mut peers_write = peers.write().await;
                peers_write.insert(addr, peer);
            }
            
            // Listen for messages from this peer
            loop {
                let message = {
                    let mut peers_write = peers.write().await;
                    if let Some(peer) = peers_write.get_mut(&addr) {
                        match peer.receive_message().await {
                            Ok(msg) => msg,
                            Err(e) => {
                                warn!("Error receiving message from {}: {}", addr, e);
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                };
                
                // Forward message to processing loop
                if message_tx.send((addr, message)).is_err() {
                    break;
                }
            }
        }
        
        // Remove peer on disconnect
        {
            let mut peers_write = peers.write().await;
            peers_write.remove(&addr);
        }
        
        info!("Peer {} disconnected", addr);
        Ok(())
    }
    
    async fn process_message(
        addr: SocketAddr,
        message: NetworkMessage,
        peers: &Arc<RwLock<HashMap<SocketAddr, Peer>>>,
        blockchain: &Arc<RwLock<Blockchain>>,
        mempool: &Arc<RwLock<Vec<Transaction>>>,
    ) -> Result<()> {
        debug!("Processing message from {}: {:?}", addr, message);
        
        match message {
            NetworkMessage::NewBlock(block) => {
                let mut blockchain_write = blockchain.write().await;
                match blockchain_write.add_block(block.clone()) {
                    Ok(_) => {
                        info!("Added new block from network: {}", block.hash);
                        // Broadcast to other peers
                        Self::broadcast_message(peers, &NetworkMessage::NewBlock(block), Some(addr)).await;
                    }
                    Err(e) => {
                        warn!("Failed to add block from {}: {}", addr, e);
                    }
                }
            }
            
            NetworkMessage::NewTransaction(tx) => {
                let blockchain_read = blockchain.read().await;
                if blockchain_read.validate_transaction(&tx).is_ok() {
                    let mut mempool_write = mempool.write().await;
                    mempool_write.push(tx.clone());
                    drop(mempool_write);
                    drop(blockchain_read);
                    
                    info!("Added new transaction to mempool: {}", tx.id);
                    // Broadcast to other peers
                    Self::broadcast_message(peers, &NetworkMessage::NewTransaction(tx), Some(addr)).await;
                } else {
                    warn!("Invalid transaction from {}: {}", addr, tx.id);
                }
            }
            
            NetworkMessage::GetBlocks { start_hash, end_hash, limit } => {
                let blockchain_read = blockchain.read().await;
                let blocks = blockchain_read.get_blocks_range(&start_hash, end_hash.as_deref(), limit);
                drop(blockchain_read);
                
                let response = NetworkMessage::Blocks(blocks);
                Self::send_to_peer(peers, addr, &response).await;
            }
            
            NetworkMessage::GetChainInfo => {
                let blockchain_read = blockchain.read().await;
                let response = NetworkMessage::ChainInfo {
                    height: blockchain_read.chain.len() as u64,
                    best_hash: blockchain_read.get_latest_block_hash(),
                    difficulty: blockchain_read.difficulty,
                    total_work: blockchain_read.calculate_total_work(),
                };
                drop(blockchain_read);
                
                Self::send_to_peer(peers, addr, &response).await;
            }
            
            NetworkMessage::GetMempool => {
                let mempool_read = mempool.read().await;
                let response = NetworkMessage::Mempool(mempool_read.clone());
                drop(mempool_read);
                
                Self::send_to_peer(peers, addr, &response).await;
            }
            
            NetworkMessage::Ping(nonce) => {
                let response = NetworkMessage::Pong(nonce);
                Self::send_to_peer(peers, addr, &response).await;
            }
            
            _ => {
                debug!("Unhandled message type from {}", addr);
            }
        }
        
        Ok(())
    }
    
    async fn broadcast_message(
        peers: &Arc<RwLock<HashMap<SocketAddr, Peer>>>,
        message: &NetworkMessage,
        exclude: Option<SocketAddr>,
    ) {
        let mut peers_write = peers.write().await;
        for (addr, peer) in peers_write.iter_mut() {
            if exclude.map_or(true, |ex| *addr != ex) && peer.info.connected {
                if let Err(e) = peer.send_message(message).await {
                    error!("Failed to send message to {}: {}", addr, e);
                }
            }
        }
    }
    
    async fn send_to_peer(
        peers: &Arc<RwLock<HashMap<SocketAddr, Peer>>>,
        addr: SocketAddr,
        message: &NetworkMessage,
    ) {
        let mut peers_write = peers.write().await;
        if let Some(peer) = peers_write.get_mut(&addr) {
            if let Err(e) = peer.send_message(message).await {
                error!("Failed to send message to {}: {}", addr, e);
            }
        }
    }
    
    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        let mut peer = Peer::new(addr);
        peer.connect().await?;
        
        let blockchain_read = self.blockchain.read().await;
        let chain_height = blockchain_read.chain.len() as u64;
        drop(blockchain_read);
        
        if peer.handshake(self.version, &self.node_id, chain_height).await? {
            info!("Connected to peer {}", addr);
            
            let mut peers_write = self.peers.write().await;
            peers_write.insert(addr, peer);
            
            // Add to known peers
            let mut known_peers_write = self.known_peers.write().await;
            if !known_peers_write.contains(&addr) {
                known_peers_write.push(addr);
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Handshake failed with {}", addr))
        }
    }
    
    async fn start_peer_maintenance(&self) {
        let peers = Arc::clone(&self.peers);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Clean up dead peers
                let mut peers_write = peers.write().await;
                let dead_peers: Vec<SocketAddr> = peers_write
                    .iter()
                    .filter(|(_, peer)| !peer.is_alive())
                    .map(|(addr, _)| *addr)
                    .collect();
                
                for addr in dead_peers {
                    info!("Removing dead peer {}", addr);
                    peers_write.remove(&addr);
                }
                
                // Send ping to all connected peers
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                for (_, peer) in peers_write.iter_mut() {
                    if peer.info.connected {
                        let _ = peer.send_message(&NetworkMessage::Ping(now)).await;
                    }
                }
            }
        });
    }
    
    pub async fn broadcast_transaction(&self, tx: Transaction) {
        Self::broadcast_message(&self.peers, &NetworkMessage::NewTransaction(tx), None).await;
    }
    
    pub async fn broadcast_block(&self, block: Block) {
        Self::broadcast_message(&self.peers, &NetworkMessage::NewBlock(block), None).await;
    }
    
    pub async fn get_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
}
