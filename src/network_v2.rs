// Production-grade P2P networking for QuantumCoin
// Built for cryptocurrency-grade security and reliability

pub mod discovery;
pub mod transport;
pub mod peer_manager;
pub mod protocol;
pub mod security;
pub mod metrics;
pub mod nat;
pub mod config;

use crate::blockchain::Blockchain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use uuid::Uuid;

pub use discovery::*;
pub use transport::*;
pub use peer_manager::*;
pub use protocol::*;
pub use security::*;
pub use metrics::*;
pub use nat::*;

/// Production network manager for QuantumCoin
#[derive(Clone)]
pub struct NetworkManager {
    pub node_id: String,
    pub chain_spec: Arc<ChainSpec>,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub peer_manager: Arc<PeerManager>,
    pub discovery: Arc<DnsDiscovery>,
    pub transport: Arc<SecureTransport>,
    pub security_manager: Arc<SecurityManager>,
    pub metrics: Arc<NetworkMetrics>,
    pub nat_manager: Arc<NatManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSpec {
    pub network_name: String,
    pub magic_bytes: [u8; 4],
    pub protocol_version: u32,
    pub default_port: u16,
    pub max_connections: usize,
    pub connection_timeout: u64,
    pub dns_seeds: Vec<String>,
    pub bootstrap_nodes: Vec<SocketAddr>,
}

impl Default for ChainSpec {
    fn default() -> Self {
        Self {
            network_name: "quantumcoin".to_string(),
            magic_bytes: [0x51, 0x54, 0x43, 0x4D], // "QTCM"
            protocol_version: 70015,
            default_port: 8333,
            max_connections: 125,
            connection_timeout: 5,
            dns_seeds: vec![
                "seed1.quantumcoin.network".to_string(),
                "seed2.quantumcoin.network".to_string(),
                "seed3.quantumcoin.network".to_string(),
                "seed4.quantumcoin.network".to_string(),
            ],
            bootstrap_nodes: vec![],
        }
    }
}

impl NetworkManager {
    pub async fn new(
        listen_addr: SocketAddr,
        blockchain: Arc<RwLock<Blockchain>>,
        chain_spec: Option<ChainSpec>,
    ) -> Result<Self> {
        let chain_spec = Arc::new(chain_spec.unwrap_or_default());
        let node_id = Uuid::new_v4().to_string();
        
        let metrics = Arc::new(NetworkMetrics::new());
        let security_manager = Arc::new(SecurityManager::new(chain_spec.clone(), metrics.clone()));
        let transport = Arc::new(SecureTransport::new(chain_spec.clone(), metrics.clone()).await?);
        let nat_manager = Arc::new(NatManager::new(listen_addr, chain_spec.clone()).await?);
        let peer_manager = Arc::new(PeerManager::new(
            chain_spec.clone(),
            security_manager.clone(),
            transport.clone(),
            metrics.clone(),
        ));
        let discovery = Arc::new(DnsDiscovery::new(
            chain_spec.clone(),
            peer_manager.clone(),
            metrics.clone(),
        ));

        Ok(Self {
            node_id,
            chain_spec,
            blockchain,
            peer_manager,
            discovery,
            transport,
            security_manager,
            metrics,
            nat_manager,
        })
    }

    /// Start the complete P2P network stack
    pub async fn start(&self) -> Result<()> {
        log::info!("Starting QuantumCoin P2P network node {}", self.node_id);
        
        // Start NAT traversal
        self.nat_manager.start().await?;
        let external_addr = self.nat_manager.get_external_address().await?;
        log::info!("External address: {:?}", external_addr);

        // Start transport layer
        self.transport.start().await?;
        
        // Start peer manager
        self.peer_manager.start().await?;
        
        // Start DNS seed discovery
        self.discovery.start().await?;
        
        // Start metrics collection
        self.metrics.start().await?;
        
        // Initial peer discovery from DNS seeds
        self.bootstrap_from_seeds().await?;
        
        log::info!("QuantumCoin P2P network fully started");
        Ok(())
    }

    /// Bootstrap initial connections from DNS seeds
    async fn bootstrap_from_seeds(&self) -> Result<()> {
        log::info!("Bootstrapping from DNS seeds...");
        
        let seed_addrs = self.discovery.resolve_all_seeds().await?;
        if seed_addrs.is_empty() {
            log::warn!("No seed addresses resolved - using bootstrap nodes");
            for addr in &self.chain_spec.bootstrap_nodes {
                let _ = self.peer_manager.connect_to_peer(*addr).await;
            }
        } else {
            log::info!("Resolved {} addresses from DNS seeds", seed_addrs.len());
            
            // Connect to multiple seed nodes for resilience
            let mut connected = 0;
            for addr in seed_addrs.into_iter().take(8) {
                if let Ok(_) = self.peer_manager.connect_to_peer(addr).await {
                    connected += 1;
                    if connected >= 4 {
                        break; // Connect to at least 4 seeds
                    }
                }
            }
            
            if connected == 0 {
                return Err(anyhow::anyhow!("Failed to connect to any seed nodes"));
            }
            
            log::info!("Connected to {} seed nodes", connected);
        }

        Ok(())
    }

    /// Sync blockchain from network (fresh node sync)
    pub async fn sync_from_zero(&self) -> Result<()> {
        log::info!("Starting fresh blockchain sync from zero...");
        
        // Ensure we have peers
        if self.peer_manager.get_peer_count().await == 0 {
            log::warn!("No peers available - attempting bootstrap");
            self.bootstrap_from_seeds().await?;
        }

        // Request blockchain sync from best peers
        self.peer_manager.request_full_sync().await?;
        
        Ok(())
    }

    /// Get network status
    pub async fn get_status(&self) -> NetworkStatus {
        NetworkStatus {
            node_id: self.node_id.clone(),
            peer_count: self.peer_manager.get_peer_count().await,
            connected_peers: self.peer_manager.get_connected_peers().await,
            best_height: self.blockchain.read().await.get_height(),
            network_hashrate: self.metrics.get_network_hashrate().await,
            sync_progress: self.peer_manager.get_sync_progress().await,
            uptime: self.metrics.get_uptime().await,
        }
    }

    /// Shutdown network gracefully
    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down P2P network...");
        
        self.peer_manager.shutdown().await?;
        self.transport.shutdown().await?;
        self.nat_manager.shutdown().await?;
        self.metrics.shutdown().await?;
        
        log::info!("P2P network shutdown complete");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStatus {
    pub node_id: String,
    pub peer_count: usize,
    pub connected_peers: Vec<String>,
    pub best_height: u64,
    pub network_hashrate: f64,
    pub sync_progress: f32,
    pub uptime: u64,
}
