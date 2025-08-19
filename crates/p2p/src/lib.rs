//! QuantumCoin P2P Networking and Gossip Protocol
//! 
//! Production-grade P2P networking with:
//! - Secure gossip protocol for blocks/transactions
//! - DoS protection and peer scoring
//! - Backpressure control and congestion management
//! - Network partition detection and recovery
//! - Flood attack resistance

pub mod gossip;
pub mod dos_protection;
pub mod message_propagation;
pub mod peer_scoring;
pub mod network_health;
pub mod priority_queue;

pub use gossip::{GossipProtocol};
pub use dos_protection::{DosProtection, PeerScore, SecurityLevel};
pub use message_propagation::{PropagationManager, PropagationStats};
pub use peer_scoring::{PeerScorer, ScoreReason, PeerBehavior};
pub use network_health::{NetworkHealth, PartitionDetector, HealthMetrics};
pub use priority_queue::{PriorityMessageQueue, MessageItem};

use std::net::SocketAddr;
use std::time::SystemTime;
use blake3::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub id: MessageId,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: i64,
    pub sender: Option<SocketAddr>,
    pub ttl: u8,
    pub priority: MessagePriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId([u8; 32]);

impl MessageId {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Self(*hash.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum P2PError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("DoS protection triggered: {0}")]
    DosProtection(String),
    
    #[error("Message validation failed: {0}")]
    MessageValidation(String),
    
    #[error("Peer banned: {peer}")]
    PeerBanned { peer: SocketAddr },
    
    #[error("Network partition detected")]
    NetworkPartition,
    
    #[error("Backpressure limit exceeded")]
    BackpressureLimit,
    
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
}

pub type Result<T> = std::result::Result<T, P2PError>;

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Gossip message with metadata
#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub network_message: NetworkMessage,
    pub first_seen: SystemTime,
    pub propagation_count: u32,
    pub source_peer: Option<SocketAddr>,
}

/// Message types for the P2P network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Block,
    Transaction,
    PeerExchange,
    HealthCheck,
    Announcement,
}

/// Gossip message wrapper
#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub network_message: NetworkMessage,
    pub first_seen: SystemTime,
    pub propagation_count: u32,
    pub source_peer: Option<SocketAddr>,
}

impl GossipMessage {
    pub fn new(
        message_type: MessageType,
        payload: Vec<u8>,
        sender: Option<SocketAddr>,
        priority: MessagePriority,
    ) -> Self {
        let id = MessageId::new(&payload);
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            network_message: NetworkMessage {
                id,
                message_type,
                payload,
                timestamp,
                sender,
                ttl: 32,
                priority,
            },
            first_seen: SystemTime::now(),
            propagation_count: 0,
            source_peer: sender,
        }
    }
}
