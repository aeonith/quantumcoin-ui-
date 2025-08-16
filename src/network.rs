// Re-export the new production-grade networking system
pub use network_v2::*;

// Legacy compatibility - gradually migrate to new system
mod network_v2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    // Node discovery
    Ping { node_id: String, version: String },
    Pong { node_id: String, version: String },
    GetPeers,
    Peers { peers: Vec<SocketAddr> },
    
    // Blockchain sync
    GetBlockchain,
    Blockchain { blocks: Vec<Block> },
    GetBlocks { start_hash: String, count: u32 },
    Blocks { blocks: Vec<Block> },
    GetBlock { hash: String },
    BlockResponse { block: Option<Block> },
    
    // Transactions
    NewTransaction { transaction: Transaction },
    GetMempool,
    Mempool { transactions: Vec<Transaction> },
    
    // Mining
    NewBlock { block: Block },
    BlockHeader { header: BlockHeader },
    
    // Error handling
    Error { message: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub hash: String,
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: u128,
    pub difficulty: usize,
    pub nonce: u64,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub address: SocketAddr,
    pub node_id: String,
    pub version: String,
    pub last_seen: std::time::Instant,
    pub connected: bool,
}

#[derive(Clone)]
pub struct NetworkManager {
    pub node_id: String,
    pub version: String,
    pub listen_addr: SocketAddr,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub peers: Arc<RwLock<HashMap<String, Peer>>>,
    pub known_addresses: Arc<RwLock<Vec<SocketAddr>>>,
    pub max_peers: usize,
}

impl NetworkManager {
    pub fn new(listen_addr: SocketAddr, blockchain: Arc<RwLock<Blockchain>>) -> Self {
        Self {
            node_id: Uuid::new_v4().to_string(),
            version: "1.0.0".to_string(),
            listen_addr,
            blockchain,
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_addresses: Arc::new(RwLock::new(Vec::new())),
            max_peers: 50,
        }
    }

    pub async fn start(&self) -> Result<()> {
        println!("Starting QuantumCoin network node {} on {}", self.node_id, self.listen_addr);
        
        // Start listening for incoming connections
        let listener = TokioTcpListener::bind(self.listen_addr).await?;
        let self_clone = self.clone();
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let self_clone = self_clone.clone();
                        tokio::spawn(async move {
                            if let Err(e) = self_clone.handle_connection(stream, addr).await {
                                eprintln!("Error handling connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => eprintln!("Failed to accept connection: {}", e),
                }
            }
        });

        // Start peer discovery and maintenance
        self.start_peer_maintenance().await;
        
        Ok(())
    }

    async fn handle_connection(&self, mut stream: TokioTcpStream, addr: SocketAddr) -> Result<()> {
        let mut buffer = vec![0; 8192];
        
        loop {
            match stream.try_read(&mut buffer) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    let message_data = &buffer[..n];
                    if let Ok(message) = serde_json::from_slice::<NetworkMessage>(message_data) {
                        let response = self.handle_message(message, addr).await;
                        if let Some(response) = response {
                            let response_data = serde_json::to_vec(&response)?;
                            let _ = stream.try_write(&response_data);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(e) => {
                    eprintln!("Error reading from stream: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn handle_message(&self, message: NetworkMessage, sender: SocketAddr) -> Option<NetworkMessage> {
        match message {
            NetworkMessage::Ping { node_id, version } => {
                // Add peer to known peers
                let peer = Peer {
                    address: sender,
                    node_id: node_id.clone(),
                    version: version.clone(),
                    last_seen: std::time::Instant::now(),
                    connected: true,
                };
                
                self.peers.write().await.insert(node_id, peer);
                
                Some(NetworkMessage::Pong {
                    node_id: self.node_id.clone(),
                    version: self.version.clone(),
                })
            }
            
            NetworkMessage::GetPeers => {
                let peers: Vec<SocketAddr> = self.peers.read().await
                    .values()
                    .map(|p| p.address)
                    .collect();
                Some(NetworkMessage::Peers { peers })
            }
            
            NetworkMessage::GetBlockchain => {
                let blockchain = self.blockchain.read().await;
                Some(NetworkMessage::Blockchain {
                    blocks: blockchain.chain.clone(),
                })
            }
            
            NetworkMessage::GetBlocks { start_hash, count } => {
                let blockchain = self.blockchain.read().await;
                let blocks = blockchain.get_blocks_range(&start_hash, None, count as usize);
                Some(NetworkMessage::Blocks { blocks })
            }
            
            NetworkMessage::GetBlock { hash } => {
                let blockchain = self.blockchain.read().await;
                let block = blockchain.get_block_by_hash(&hash).cloned();
                Some(NetworkMessage::BlockResponse { block })
            }
            
            NetworkMessage::NewTransaction { transaction } => {
                let mut blockchain = self.blockchain.write().await;
                if let Ok(_) = blockchain.add_transaction(transaction.clone()) {
                    // Broadcast to other peers
                    self.broadcast_to_peers(NetworkMessage::NewTransaction { transaction }).await;
                }
                None
            }
            
            NetworkMessage::NewBlock { block } => {
                let mut blockchain = self.blockchain.write().await;
                if let Ok(_) = blockchain.add_block(block.clone()) {
                    println!("New block added: {}", block.hash);
                    // Broadcast to other peers
                    self.broadcast_to_peers(NetworkMessage::NewBlock { block }).await;
                }
                None
            }
            
            NetworkMessage::GetMempool => {
                let blockchain = self.blockchain.read().await;
                Some(NetworkMessage::Mempool {
                    transactions: blockchain.get_pending_transactions(),
                })
            }
            
            _ => None,
        }
    }

    async fn broadcast_to_peers(&self, message: NetworkMessage) {
        let peers = self.peers.read().await;
        for peer in peers.values() {
            if let Err(e) = self.send_message_to_peer(&peer.address, &message).await {
                eprintln!("Failed to send message to peer {}: {}", peer.address, e);
            }
        }
    }

    async fn send_message_to_peer(&self, addr: &SocketAddr, message: &NetworkMessage) -> Result<()> {
        let mut stream = TokioTcpStream::connect(addr).await?;
        let data = serde_json::to_vec(message)?;
        stream.try_write(&data)?;
        Ok(())
    }

    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        let ping = NetworkMessage::Ping {
            node_id: self.node_id.clone(),
            version: self.version.clone(),
        };
        
        self.send_message_to_peer(&addr, &ping).await?;
        println!("Connected to peer: {}", addr);
        Ok(())
    }

    async fn start_peer_maintenance(&self) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            loop {
                // Clean up disconnected peers
                let mut peers = self_clone.peers.write().await;
                let now = std::time::Instant::now();
                peers.retain(|_, peer| {
                    now.duration_since(peer.last_seen).as_secs() < 300 // 5 minutes timeout
                });
                drop(peers);

                // Try to discover new peers
                self_clone.discover_peers().await;
                
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        });
    }

    async fn discover_peers(&self) {
        let known_addresses = self.known_addresses.read().await.clone();
        for addr in known_addresses {
            if self.peers.read().await.len() >= self.max_peers {
                break;
            }
            
            if let Err(_) = self.connect_to_peer(addr).await {
                // Connection failed, remove from known addresses
                let mut addresses = self.known_addresses.write().await;
                addresses.retain(|a| *a != addr);
            }
        }
    }

    pub async fn add_bootstrap_node(&self, addr: SocketAddr) {
        self.known_addresses.write().await.push(addr);
        let _ = self.connect_to_peer(addr).await;
    }

    pub async fn sync_blockchain(&self) -> Result<()> {
        println!("Syncing blockchain with network...");
        
        let peers = self.peers.read().await;
        if peers.is_empty() {
            return Ok(());
        }

        for peer in peers.values().take(3) { // Sync with up to 3 peers
            match self.send_message_to_peer(&peer.address, &NetworkMessage::GetBlockchain).await {
                Ok(_) => println!("Requested blockchain from peer: {}", peer.address),
                Err(e) => eprintln!("Failed to sync with peer {}: {}", peer.address, e),
            }
        }
        
        Ok(())
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) {
        self.broadcast_to_peers(NetworkMessage::NewTransaction { transaction }).await;
    }

    pub async fn broadcast_block(&self, block: Block) {
        self.broadcast_to_peers(NetworkMessage::NewBlock { block }).await;
    }

    pub async fn get_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub async fn get_peers(&self) -> Vec<Peer> {
        self.peers.read().await.values().cloned().collect()
    }
}

// Mining pool support
#[derive(Clone)]
pub struct MiningPool {
    pub network: NetworkManager,
    pub miners: Arc<RwLock<HashMap<String, MinerInfo>>>,
    pub current_work: Arc<RwLock<Option<Block>>>,
}

#[derive(Debug, Clone)]
pub struct MinerInfo {
    pub address: String,
    pub hash_rate: f64,
    pub last_share: std::time::Instant,
    pub total_shares: u64,
}

impl MiningPool {
    pub fn new(network: NetworkManager) -> Self {
        Self {
            network,
            miners: Arc::new(RwLock::new(HashMap::new())),
            current_work: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn submit_work(&self, miner_id: String, block: Block) -> Result<bool> {
        // Validate the work
        let blockchain = self.network.blockchain.read().await;
        if blockchain.validate_block(&block).is_ok() {
            drop(blockchain);
            
            // Add block to blockchain
            let mut blockchain = self.network.blockchain.write().await;
            blockchain.add_block(block.clone())?;
            
            // Broadcast new block
            self.network.broadcast_block(block).await;
            
            // Update miner stats
            let mut miners = self.miners.write().await;
            if let Some(miner) = miners.get_mut(&miner_id) {
                miner.total_shares += 1;
                miner.last_share = std::time::Instant::now();
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_work(&self, miner_id: String) -> Option<Block> {
        // Register miner if new
        let mut miners = self.miners.write().await;
        miners.entry(miner_id).or_insert(MinerInfo {
            address: "unknown".to_string(),
            hash_rate: 0.0,
            last_share: std::time::Instant::now(),
            total_shares: 0,
        });
        drop(miners);

        // Return current work template
        self.current_work.read().await.clone()
    }
}
