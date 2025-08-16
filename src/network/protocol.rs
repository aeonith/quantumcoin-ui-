// Network protocol messages and versioning
use crate::block::Block;
use crate::transaction::Transaction;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 70015;
pub const MIN_PROTOCOL_VERSION: u32 = 70010;

/// Network protocol messages for QuantumCoin P2P
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    // Connection management
    Version {
        version: u32,
        services: u64,
        timestamp: u64,
        user_agent: String,
        start_height: u64,
        relay: bool,
    },
    VerAck,
    Ping {
        nonce: u64,
    },
    Pong {
        nonce: u64,
    },
    
    // Peer discovery
    GetAddr,
    Addr {
        addresses: Vec<NetworkAddress>,
    },
    
    // Block chain synchronization
    GetHeaders {
        start_hash: String,
        stop_hash: String,
    },
    Headers {
        headers: Vec<BlockHeader>,
    },
    GetBlocks {
        start_hash: String,
        stop_hash: String,
    },
    Inv {
        inventory: Vec<InventoryItem>,
    },
    GetData {
        inventory: Vec<InventoryItem>,
    },
    Block {
        block: Block,
    },
    
    // Transaction handling
    Tx {
        transaction: Transaction,
    },
    MemPool,
    GetMemPool,
    
    // Fee estimation
    FeeFilter {
        fee_rate: u64,
    },
    
    // Compact blocks (BIP152)
    SendCmpct {
        announce: bool,
        version: u32,
    },
    CmpctBlock {
        block: CompactBlock,
    },
    GetBlockTxn {
        request: BlockTransactionsRequest,
    },
    BlockTxn {
        response: BlockTransactionsResponse,
    },
    
    // Error handling
    Reject {
        message: String,
        code: RejectCode,
        reason: String,
        data: Option<Vec<u8>>,
    },
    
    // Custom QuantumCoin messages
    QuantumProof {
        proof_data: Vec<u8>,
    },
    
    // Service messages
    Alert {
        alert: AlertMessage,
    },
    
    // Unknown message type for forward compatibility
    Unknown {
        command: String,
        payload: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkAddress {
    pub timestamp: u32,
    pub services: u64,
    pub ip: std::net::IpAddr,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: String,
    pub merkle_root: String,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryItem {
    pub inv_type: InventoryType,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InventoryType {
    Error = 0,
    MsgTx = 1,
    MsgBlock = 2,
    MsgFilteredBlock = 3,
    MsgCmpctBlock = 4,
    MsgWitnessBlock = 0x40000002,
    MsgWitnessTx = 0x40000001,
    MsgFilteredWitnessBlock = 0x40000003,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompactBlock {
    pub header: BlockHeader,
    pub nonce: u64,
    pub short_txids: Vec<u64>,
    pub prefilled_txns: Vec<PrefilledTransaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefilledTransaction {
    pub index: u32,
    pub tx: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockTransactionsRequest {
    pub block_hash: String,
    pub indexes: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockTransactionsResponse {
    pub block_hash: String,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RejectCode {
    Malformed = 0x01,
    Invalid = 0x10,
    Obsolete = 0x11,
    Duplicate = 0x12,
    Nonstandard = 0x40,
    Dust = 0x41,
    InsufficientFee = 0x42,
    Checkpoint = 0x43,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlertMessage {
    pub version: u32,
    pub relay_until: u64,
    pub expiration: u64,
    pub id: u32,
    pub cancel: u32,
    pub min_ver: u32,
    pub max_ver: u32,
    pub priority: u32,
    pub comment: String,
    pub status_bar: String,
}

/// Protocol version management
pub struct ProtocolVersion {
    pub version: u32,
    pub supported_features: Vec<String>,
}

impl ProtocolVersion {
    pub fn new(version: u32) -> Self {
        let mut supported_features = Vec::new();
        
        // Features by protocol version
        if version >= 70001 {
            supported_features.push("bloom_filter".to_string());
        }
        if version >= 70002 {
            supported_features.push("reject_message".to_string());
        }
        if version >= 70012 {
            supported_features.push("fee_filter".to_string());
        }
        if version >= 70014 {
            supported_features.push("compact_blocks".to_string());
        }
        if version >= 70015 {
            supported_features.push("quantum_proof".to_string());
        }
        
        Self {
            version,
            supported_features,
        }
    }
    
    pub fn is_compatible(&self, other_version: u32) -> bool {
        other_version >= MIN_PROTOCOL_VERSION && self.version >= MIN_PROTOCOL_VERSION
    }
    
    pub fn supports_feature(&self, feature: &str) -> bool {
        self.supported_features.contains(&feature.to_string())
    }
}

impl NetworkMessage {
    /// Serialize message to bytes with proper framing
    pub fn serialize(&self) -> Result<Vec<u8>> {
        // Message format: [magic][command][length][checksum][payload]
        let magic = [0x51, 0x54, 0x43, 0x4D]; // "QTCM"
        let command = self.get_command();
        let payload = bincode::serialize(self)?;
        let length = payload.len() as u32;
        let checksum = calculate_checksum(&payload);
        
        let mut data = Vec::new();
        data.extend_from_slice(&magic);
        data.extend_from_slice(&command);
        data.extend_from_slice(&length.to_le_bytes());
        data.extend_from_slice(&checksum);
        data.extend_from_slice(&payload);
        
        Ok(data)
    }
    
    /// Deserialize message from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(anyhow::anyhow!("Message too short"));
        }
        
        // Verify magic bytes
        let magic = [data[0], data[1], data[2], data[3]];
        if magic != [0x51, 0x54, 0x43, 0x4D] {
            return Err(anyhow::anyhow!("Invalid magic bytes"));
        }
        
        // Extract command
        let command = &data[4..16];
        
        // Extract length
        let length = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        
        // Extract checksum
        let checksum = [data[20], data[21], data[22], data[23]];
        
        // Extract payload
        if data.len() < 24 + length as usize {
            return Err(anyhow::anyhow!("Incomplete message"));
        }
        let payload = &data[24..24 + length as usize];
        
        // Verify checksum
        if calculate_checksum(payload) != checksum {
            return Err(anyhow::anyhow!("Invalid checksum"));
        }
        
        // Deserialize payload
        let message: NetworkMessage = bincode::deserialize(payload)?;
        Ok(message)
    }
    
    /// Get command name for message
    pub fn get_command(&self) -> [u8; 12] {
        let command_str = match self {
            NetworkMessage::Version { .. } => "version",
            NetworkMessage::VerAck => "verack",
            NetworkMessage::Ping { .. } => "ping",
            NetworkMessage::Pong { .. } => "pong",
            NetworkMessage::GetAddr => "getaddr",
            NetworkMessage::Addr { .. } => "addr",
            NetworkMessage::GetHeaders { .. } => "getheaders",
            NetworkMessage::Headers { .. } => "headers",
            NetworkMessage::GetBlocks { .. } => "getblocks",
            NetworkMessage::Inv { .. } => "inv",
            NetworkMessage::GetData { .. } => "getdata",
            NetworkMessage::Block { .. } => "block",
            NetworkMessage::Tx { .. } => "tx",
            NetworkMessage::MemPool => "mempool",
            NetworkMessage::GetMemPool => "getmempool",
            NetworkMessage::FeeFilter { .. } => "feefilter",
            NetworkMessage::SendCmpct { .. } => "sendcmpct",
            NetworkMessage::CmpctBlock { .. } => "cmpctblock",
            NetworkMessage::GetBlockTxn { .. } => "getblocktxn",
            NetworkMessage::BlockTxn { .. } => "blocktxn",
            NetworkMessage::Reject { .. } => "reject",
            NetworkMessage::QuantumProof { .. } => "qproof",
            NetworkMessage::Alert { .. } => "alert",
            NetworkMessage::Unknown { command, .. } => command,
        };
        
        let mut cmd = [0u8; 12];
        let bytes = command_str.as_bytes();
        let len = bytes.len().min(12);
        cmd[..len].copy_from_slice(&bytes[..len]);
        cmd
    }
    
    /// Check if message is critical for network operation
    pub fn is_critical(&self) -> bool {
        matches!(self,
            NetworkMessage::Version { .. } |
            NetworkMessage::VerAck |
            NetworkMessage::Ping { .. } |
            NetworkMessage::Pong { .. } |
            NetworkMessage::Headers { .. } |
            NetworkMessage::Block { .. }
        )
    }
    
    /// Get message priority for queue processing
    pub fn get_priority(&self) -> u8 {
        match self {
            NetworkMessage::Version { .. } | NetworkMessage::VerAck => 0, // Highest
            NetworkMessage::Ping { .. } | NetworkMessage::Pong { .. } => 1,
            NetworkMessage::Block { .. } | NetworkMessage::Headers { .. } => 2,
            NetworkMessage::Tx { .. } => 3,
            NetworkMessage::Inv { .. } | NetworkMessage::GetData { .. } => 4,
            _ => 5, // Lowest
        }
    }
}

/// Calculate Blake3 checksum for message integrity
fn calculate_checksum(data: &[u8]) -> [u8; 4] {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    [hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]]
}

/// Message handler trait for processing network messages
pub trait MessageHandler {
    fn handle_version(&self, version: u32, services: u64, user_agent: String, start_height: u64) -> Result<()>;
    fn handle_ver_ack(&self) -> Result<()>;
    fn handle_ping(&self, nonce: u64) -> Result<NetworkMessage>;
    fn handle_pong(&self, nonce: u64) -> Result<()>;
    fn handle_addr(&self, addresses: Vec<NetworkAddress>) -> Result<()>;
    fn handle_inv(&self, inventory: Vec<InventoryItem>) -> Result<Option<NetworkMessage>>;
    fn handle_block(&self, block: Block) -> Result<()>;
    fn handle_transaction(&self, transaction: Transaction) -> Result<()>;
    fn handle_headers(&self, headers: Vec<BlockHeader>) -> Result<()>;
    fn handle_reject(&self, message: String, code: RejectCode, reason: String) -> Result<()>;
}

/// Protocol state machine for connection management
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolState {
    Disconnected,
    Connected,
    VersionSent,
    VersionReceived,
    Handshaked,
    Ready,
}

pub struct ProtocolStateMachine {
    state: ProtocolState,
    version_sent: bool,
    version_received: bool,
    verack_sent: bool,
    verack_received: bool,
}

impl ProtocolStateMachine {
    pub fn new() -> Self {
        Self {
            state: ProtocolState::Disconnected,
            version_sent: false,
            version_received: false,
            verack_sent: false,
            verack_received: false,
        }
    }
    
    pub fn on_connect(&mut self) {
        self.state = ProtocolState::Connected;
    }
    
    pub fn on_version_sent(&mut self) {
        self.version_sent = true;
        self.state = ProtocolState::VersionSent;
        self.update_state();
    }
    
    pub fn on_version_received(&mut self) {
        self.version_received = true;
        self.state = ProtocolState::VersionReceived;
        self.update_state();
    }
    
    pub fn on_verack_sent(&mut self) {
        self.verack_sent = true;
        self.update_state();
    }
    
    pub fn on_verack_received(&mut self) {
        self.verack_received = true;
        self.update_state();
    }
    
    fn update_state(&mut self) {
        if self.version_sent && self.version_received && !self.is_handshaked() {
            self.state = ProtocolState::Handshaked;
        }
        
        if self.version_sent && self.version_received && 
           self.verack_sent && self.verack_received {
            self.state = ProtocolState::Ready;
        }
    }
    
    pub fn is_ready(&self) -> bool {
        self.state == ProtocolState::Ready
    }
    
    pub fn is_handshaked(&self) -> bool {
        self.state == ProtocolState::Handshaked || self.state == ProtocolState::Ready
    }
    
    pub fn get_state(&self) -> ProtocolState {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_serialization() {
        let msg = NetworkMessage::Ping { nonce: 12345 };
        let serialized = msg.serialize().unwrap();
        let deserialized = NetworkMessage::deserialize(&serialized).unwrap();
        
        if let NetworkMessage::Ping { nonce } = deserialized {
            assert_eq!(nonce, 12345);
        } else {
            panic!("Wrong message type");
        }
    }
    
    #[test]
    fn test_protocol_version() {
        let protocol = ProtocolVersion::new(70015);
        assert!(protocol.supports_feature("quantum_proof"));
        assert!(protocol.is_compatible(70010));
        assert!(!protocol.is_compatible(70009));
    }
    
    #[test]
    fn test_protocol_state_machine() {
        let mut protocol = ProtocolStateMachine::new();
        
        protocol.on_connect();
        assert_eq!(protocol.get_state(), ProtocolState::Connected);
        
        protocol.on_version_sent();
        protocol.on_version_received();
        assert_eq!(protocol.get_state(), ProtocolState::Handshaked);
        
        protocol.on_verack_sent();
        protocol.on_verack_received();
        assert_eq!(protocol.get_state(), ProtocolState::Ready);
        assert!(protocol.is_ready());
    }
}
