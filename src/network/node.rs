use crate::network::{NetworkError, NetworkConfig, NetworkStats};
use crate::network::peer::PeerManager;
use crate::network::discovery::PeerDiscovery;
use crate::network::sync::BlockchainSync;
use crate::network::protocol::{Message, MessageType, ProtocolHandler};
use crate::blockchain::Blockchain;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use parking_lot::RwLock;

pub struct NetworkNode {
    config: NetworkConfig,
    peer_manager: Arc<PeerManager>,
    peer_discovery: Arc<PeerDiscovery>,
    blockchain_sync: Arc<BlockchainSync>,
    protocol_handler: ProtocolHandler,
    listener: Option<TcpListener>,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<NetworkStats>>,
}

impl NetworkNode {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>, config: NetworkConfig) -> Self {
        let peer_manager = Arc::new(PeerManager::new(config.clone()));
        let peer_discovery = Arc::new(PeerDiscovery::new(config.clone(), peer_manager.clone()));
        let blockchain_sync = Arc::new(BlockchainSync::new(blockchain, peer_manager.clone(), config.clone()));
        let protocol_handler = ProtocolHandler::new(config.clone());

        Self {
            config,
            peer_manager,
            peer_discovery,
            blockchain_sync,
            protocol_handler,
            listener: None,
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(NetworkStats {
                connected_peers: 0,
                inbound_peers: 0,
                outbound_peers: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                messages_sent: 0,
                messages_received: 0,
                blacklisted_peers: 0,
                sync_height: 0,
                is_syncing: false,
            })),
        }
    }

    pub async fn start(&mut self) -> Result<(), NetworkError> {
        println!("Starting QuantumCoin network node on port {}", self.config.listen_port);

        {
            let mut running = self.is_running.write();
            *running = true;
        }

        // Start listening for incoming connections
        let listener = TcpListener::bind(("0.0.0.0", self.config.listen_port))
            .await
            .map_err(|e| NetworkError::ConnectionFailed(format!("Failed to bind listener: {}", e)))?;
        
        self.listener = Some(listener);

        // Start peer discovery
        self.peer_discovery.start_discovery().await?;

        // Start blockchain sync
        self.blockchain_sync.start_sync().await?;

        // Start main network loop
        let node_data = self.get_node_data();
        tokio::spawn(async move {
            Self::run_network_loop(node_data).await;
        });

        println!("Network node started successfully");
        Ok(())
    }

    pub async fn stop(&mut self) {
        println!("Stopping network node...");
        
        {
            let mut running = self.is_running.write();
            *running = false;
        }

        // Disconnect all peers
        let peers = self.peer_manager.get_all_peers().await;
        for peer in peers {
            peer.disconnect().await;
        }

        println!("Network node stopped");
    }

    async fn run_network_loop(data: NodeData) {
        while *data.is_running.read() {
            if let Err(e) = Self::handle_network_iteration(&data).await {
                println!("Network iteration error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    async fn handle_network_iteration(data: &NodeData) -> Result<(), NetworkError> {
        // Accept incoming connections
        // Handle peer messages
        // Update statistics
        // Cleanup tasks
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    pub async fn broadcast_transaction(&self, transaction: crate::transaction::Transaction) -> Result<usize, NetworkError> {
        let message = Message::new(
            MessageType::SendTransaction,
            SocketAddr::from(([0, 0, 0, 0], 0)),
            crate::network::protocol::MessagePayload::Transaction(transaction),
        );

        self.peer_manager.broadcast_message(message).await
    }

    pub async fn broadcast_block(&self, block: crate::block::Block) -> Result<usize, NetworkError> {
        let message = Message::new(
            MessageType::SendBlock,
            SocketAddr::from(([0, 0, 0, 0], 0)),
            crate::network::protocol::MessagePayload::Block(block),
        );

        self.peer_manager.broadcast_message(message).await
    }

    pub async fn get_network_stats(&self) -> NetworkStats {
        let connected_peers = self.peer_manager.get_connected_peers().await;
        let inbound_count = self.peer_manager.get_inbound_peer_count().await;
        let sync_progress = self.blockchain_sync.get_sync_progress().await;

        NetworkStats {
            connected_peers: connected_peers.len(),
            inbound_peers: inbound_count,
            outbound_peers: connected_peers.len() - inbound_count,
            total_bytes_sent: 0, // Would be tracked in real implementation
            total_bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            blacklisted_peers: 0,
            sync_height: sync_progress.current_height,
            is_syncing: matches!(self.blockchain_sync.get_sync_state().await, crate::network::sync::SyncState::Syncing),
        }
    }

    pub fn is_running(&self) -> bool {
        *self.is_running.read()
    }

    fn get_node_data(&self) -> NodeData {
        NodeData {
            config: self.config.clone(),
            peer_manager: self.peer_manager.clone(),
            is_running: self.is_running.clone(),
        }
    }
}

#[derive(Clone)]
struct NodeData {
    config: NetworkConfig,
    peer_manager: Arc<PeerManager>,
    is_running: Arc<RwLock<bool>>,
}
