use std::net::SocketAddr;
use serde::{Serialize, Deserialize};

// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

// Network magic bytes for message identification
pub const NETWORK_MAGIC: u32 = 0xD9B4BEF9;

// Message types
pub const MSG_HANDSHAKE: u8 = 1;
pub const MSG_HANDSHAKE_ACK: u8 = 2;
pub const MSG_GET_BLOCKS: u8 = 3;
pub const MSG_BLOCKS: u8 = 4;
pub const MSG_NEW_BLOCK: u8 = 5;
pub const MSG_GET_BLOCK: u8 = 6;
pub const MSG_BLOCK: u8 = 7;
pub const MSG_NEW_TRANSACTION: u8 = 8;
pub const MSG_GET_MEMPOOL: u8 = 9;
pub const MSG_MEMPOOL: u8 = 10;
pub const MSG_GET_CHAIN_INFO: u8 = 11;
pub const MSG_CHAIN_INFO: u8 = 12;
pub const MSG_GET_PEERS: u8 = 13;
pub const MSG_PEERS: u8 = 14;
pub const MSG_PING: u8 = 15;
pub const MSG_PONG: u8 = 16;
pub const MSG_ERROR: u8 = 255;

// Protocol limits
pub const MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024; // 32MB
pub const MAX_BLOCKS_PER_MESSAGE: usize = 500;
pub const MAX_TRANSACTIONS_PER_MESSAGE: usize = 10000;
pub const MAX_PEERS_PER_MESSAGE: usize = 1000;

// Connection timeouts
pub const HANDSHAKE_TIMEOUT_SECS: u64 = 30;
pub const MESSAGE_TIMEOUT_SECS: u64 = 60;
pub const KEEPALIVE_INTERVAL_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSettings {
    pub version: u32,
    pub max_connections: usize,
    pub connection_timeout: u64,
    pub message_timeout: u64,
    pub keepalive_interval: u64,
    pub max_message_size: usize,
}

impl Default for ProtocolSettings {
    fn default() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            max_connections: 100,
            connection_timeout: 30,
            message_timeout: 60,
            keepalive_interval: 30,
            max_message_size: MAX_MESSAGE_SIZE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub supports_mining: bool,
    pub supports_wallet: bool,
    pub supports_relay: bool,
    pub supports_bloom_filter: bool,
    pub supports_compact_blocks: bool,
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self {
            supports_mining: true,
            supports_wallet: true,
            supports_relay: true,
            supports_bloom_filter: false,
            supports_compact_blocks: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connections_accepted: u64,
    pub connections_initiated: u64,
    pub handshakes_completed: u64,
    pub handshakes_failed: u64,
}

impl NetworkStats {
    pub fn new() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            connections_accepted: 0,
            connections_initiated: 0,
            handshakes_completed: 0,
            handshakes_failed: 0,
        }
    }
    
    pub fn record_message_sent(&mut self, size: usize) {
        self.messages_sent += 1;
        self.bytes_sent += size as u64;
    }
    
    pub fn record_message_received(&mut self, size: usize) {
        self.messages_received += 1;
        self.bytes_received += size as u64;
    }
    
    pub fn record_connection_accepted(&mut self) {
        self.connections_accepted += 1;
    }
    
    pub fn record_connection_initiated(&mut self) {
        self.connections_initiated += 1;
    }
    
    pub fn record_handshake_completed(&mut self) {
        self.handshakes_completed += 1;
    }
    
    pub fn record_handshake_failed(&mut self) {
        self.handshakes_failed += 1;
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate_message_size(size: usize) -> bool {
    size <= MAX_MESSAGE_SIZE
}

pub fn validate_blocks_count(count: usize) -> bool {
    count <= MAX_BLOCKS_PER_MESSAGE
}

pub fn validate_transactions_count(count: usize) -> bool {
    count <= MAX_TRANSACTIONS_PER_MESSAGE
}

pub fn validate_peers_count(count: usize) -> bool {
    count <= MAX_PEERS_PER_MESSAGE
}

pub fn is_valid_protocol_version(version: u32) -> bool {
    version == PROTOCOL_VERSION
}
