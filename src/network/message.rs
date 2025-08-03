use crate::{Block, Transaction};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    // Handshake messages
    Handshake {
        version: u32,
        node_id: String,
        chain_height: u64,
        timestamp: u64,
    },
    HandshakeAck {
        accepted: bool,
        peer_list: Vec<SocketAddr>,
    },
    
    // Block messages
    GetBlocks {
        start_hash: String,
        end_hash: Option<String>,
        limit: usize,
    },
    Blocks(Vec<Block>),
    NewBlock(Block),
    GetBlock(String), // block hash
    Block(Option<Block>),
    
    // Transaction messages
    NewTransaction(Transaction),
    GetMempool,
    Mempool(Vec<Transaction>),
    
    // Chain info
    GetChainInfo,
    ChainInfo {
        height: u64,
        best_hash: String,
        difficulty: usize,
        total_work: u64,
    },
    
    // Peer discovery
    GetPeers,
    Peers(Vec<SocketAddr>),
    
    // Ping/Pong for keepalive
    Ping(u64),
    Pong(u64),
    
    // Error handling
    Error(String),
}

impl NetworkMessage {
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub magic: u32,
    pub command: u8,
    pub length: u32,
    pub checksum: u32,
}

impl MessageHeader {
    pub const MAGIC: u32 = 0xD9B4BEF9; // Bitcoin-style magic bytes
    pub const SIZE: usize = 13;
    
    pub fn new(command: u8, payload_len: u32) -> Self {
        Self {
            magic: Self::MAGIC,
            command,
            length: payload_len,
            checksum: 0, // Will be calculated
        }
    }
    
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.magic.to_le_bytes());
        bytes[4] = self.command;
        bytes[5..9].copy_from_slice(&self.length.to_le_bytes());
        bytes[9..13].copy_from_slice(&self.checksum.to_le_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::SIZE {
            return Err("Insufficient bytes for header");
        }
        
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != Self::MAGIC {
            return Err("Invalid magic bytes");
        }
        
        Ok(Self {
            magic,
            command: bytes[4],
            length: u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
            checksum: u32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]),
        })
    }
}
