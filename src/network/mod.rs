pub mod peer;
pub mod protocol;
pub mod discovery;
pub mod sync;
pub mod node;

pub use node::NetworkNode;
pub use peer::{Peer, PeerInfo, PeerManager};
pub use protocol::{Message, MessageType, ProtocolHandler};
pub use discovery::PeerDiscovery;
pub use sync::BlockchainSync;

use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid message format")]
    InvalidMessage,
    #[error("Peer rejected connection")]
    PeerRejected,
    #[error("Network timeout")]
    Timeout,
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Peer blacklisted")]
    Blacklisted,
    #[error("Protocol version mismatch")]
    VersionMismatch,
    #[error("Invalid peer signature")]
    InvalidSignature,
    #[error("Sync failed: {0}")]
    SyncFailed(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkConfig {
    pub protocol_version: u32,
    pub network_id: u32, // 0=mainnet, 1=testnet, 2=regtest
    pub max_peers: usize,
    pub max_inbound_peers: usize,
    pub max_outbound_peers: usize,
    pub listen_port: u16,
    pub bootstrap_peers: Vec<SocketAddr>,
    pub dns_seeds: Vec<String>,
    pub connection_timeout_secs: u64,
    pub ping_interval_secs: u64,
    pub sync_timeout_secs: u64,
    pub rate_limit_requests_per_minute: u32,
    pub blacklist_duration_hours: u64,
    pub enable_encryption: bool,
    pub require_authentication: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            protocol_version: 1,
            network_id: 0, // mainnet
            max_peers: 125,
            max_inbound_peers: 75,
            max_outbound_peers: 50,
            listen_port: 8333,
            bootstrap_peers: vec![],
            dns_seeds: vec![
                "seed1.quantumcoin.org".to_string(),
                "seed2.quantumcoin.org".to_string(),
            ],
            connection_timeout_secs: 30,
            ping_interval_secs: 60,
            sync_timeout_secs: 300,
            rate_limit_requests_per_minute: 100,
            blacklist_duration_hours: 24,
            enable_encryption: true,
            require_authentication: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub inbound_peers: usize,
    pub outbound_peers: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub blacklisted_peers: usize,
    pub sync_height: u64,
    pub is_syncing: bool,
}
