use serde::{Serialize, Deserialize};
use crate::block::Block;
use crate::transaction::Transaction;
use crate::network::{NetworkError, NetworkConfig};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::net::SocketAddr;
use blake3;
use base64::{encode, decode};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageType {
    // Connection Management
    Handshake,
    HandshakeAck,
    Version,
    Ping,
    Pong,
    Disconnect,
    
    // Blockchain Sync
    GetBlocks,
    SendBlock,
    GetHeaders,
    SendHeaders,
    GetBlockData,
    
    // Transaction Handling
    GetTransaction,
    SendTransaction,
    TransactionInventory,
    GetMempool,
    
    // Peer Discovery
    GetPeers,
    SendPeers,
    
    // Network Status
    GetStatus,
    SendStatus,
    
    // Error Handling
    Error,
    Reject,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub sender: SocketAddr,
    pub payload: MessagePayload,
    pub signature: Option<String>,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessagePayload {
    Handshake(HandshakeData),
    Version(VersionData),
    Ping(PingData),
    Pong(PongData),
    Block(Block),
    Transaction(Transaction),
    Blocks(Vec<Block>),
    Transactions(Vec<Transaction>),
    Headers(Vec<BlockHeader>),
    Peers(Vec<PeerInfo>),
    Status(NetworkStatus),
    Error(ErrorData),
    Empty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HandshakeData {
    pub protocol_version: u32,
    pub network_id: u32,
    pub node_id: String,
    pub user_agent: String,
    pub services: u64, // Bitfield of supported services
    pub timestamp: DateTime<Utc>,
    pub best_block_height: u64,
    pub best_block_hash: String,
    pub public_key: String, // For authentication
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionData {
    pub protocol_version: u32,
    pub services: u64,
    pub timestamp: DateTime<Utc>,
    pub receiver_address: SocketAddr,
    pub sender_address: SocketAddr,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PingData {
    pub nonce: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PongData {
    pub nonce: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    pub hash: String,
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: DateTime<Utc>,
    pub height: u64,
    pub difficulty: usize,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub last_seen: DateTime<Utc>,
    pub services: u64,
    pub user_agent: String,
    pub version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkStatus {
    pub version: u32,
    pub block_height: u64,
    pub best_block_hash: String,
    pub connected_peers: usize,
    pub mempool_size: usize,
    pub is_syncing: bool,
    pub sync_progress: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorData {
    pub code: u32,
    pub message: String,
    pub details: Option<String>,
}

impl Message {
    pub fn new(
        message_type: MessageType,
        sender: SocketAddr,
        payload: MessagePayload,
    ) -> Self {
        let mut message = Message {
            id: Uuid::new_v4().to_string(),
            message_type,
            timestamp: Utc::now(),
            sender,
            payload,
            signature: None,
            checksum: String::new(),
        };
        
        message.checksum = message.calculate_checksum();
        message
    }

    pub fn calculate_checksum(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.id.as_bytes());
        hasher.update(&bincode::serialize(&self.message_type).unwrap_or_default());
        hasher.update(&self.timestamp.timestamp().to_be_bytes());
        hasher.update(&bincode::serialize(&self.payload).unwrap_or_default());
        encode(hasher.finalize().as_bytes())
    }

    pub fn verify_checksum(&self) -> bool {
        let calculated = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(self.id.as_bytes());
            hasher.update(&bincode::serialize(&self.message_type).unwrap_or_default());
            hasher.update(&self.timestamp.timestamp().to_be_bytes());
            hasher.update(&bincode::serialize(&self.payload).unwrap_or_default());
            encode(hasher.finalize().as_bytes())
        };
        calculated == self.checksum
    }

    pub fn sign(&mut self, private_key: &pqcrypto_dilithium::dilithium2::SecretKey) {
        let message_bytes = self.to_bytes();
        let signature = pqcrypto_dilithium::dilithium2::sign_detached(&message_bytes, private_key);
        self.signature = Some(encode(signature.as_bytes()));
    }

    pub fn verify_signature(&self, public_key: &pqcrypto_dilithium::dilithium2::PublicKey) -> bool {
        if let Some(sig_str) = &self.signature {
            if let Ok(sig_bytes) = decode(sig_str) {
                if let Ok(signature) = pqcrypto_dilithium::dilithium2::DetachedSignature::from_bytes(&sig_bytes) {
                    let message_bytes = self.to_bytes_without_signature();
                    return signature.verify_detached(&message_bytes, public_key).is_ok();
                }
            }
        }
        false
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    pub fn to_bytes_without_signature(&self) -> Vec<u8> {
        let mut temp = self.clone();
        temp.signature = None;
        bincode::serialize(&temp).unwrap_or_default()
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, NetworkError> {
        bincode::deserialize(data).map_err(|_| NetworkError::InvalidMessage)
    }

    pub fn is_expired(&self, timeout_secs: u64) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.timestamp);
        elapsed.num_seconds() > timeout_secs as i64
    }
}

pub struct ProtocolHandler {
    config: NetworkConfig,
}

impl ProtocolHandler {
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }

    pub fn validate_message(&self, message: &Message) -> Result<(), NetworkError> {
        // Check message integrity
        if !message.verify_checksum() {
            return Err(NetworkError::InvalidMessage);
        }

        // Check if message is expired
        if message.is_expired(self.config.connection_timeout_secs) {
            return Err(NetworkError::Timeout);
        }

        // Additional validation based on message type
        match &message.message_type {
            MessageType::Handshake => self.validate_handshake(message),
            MessageType::Version => self.validate_version(message),
            MessageType::SendBlock => self.validate_block_message(message),
            MessageType::SendTransaction => self.validate_transaction_message(message),
            _ => Ok(()),
        }
    }

    fn validate_handshake(&self, message: &Message) -> Result<(), NetworkError> {
        if let MessagePayload::Handshake(handshake) = &message.payload {
            if handshake.protocol_version != self.config.protocol_version {
                return Err(NetworkError::VersionMismatch);
            }
            if handshake.network_id != self.config.network_id {
                return Err(NetworkError::VersionMismatch);
            }
        }
        Ok(())
    }

    fn validate_version(&self, message: &Message) -> Result<(), NetworkError> {
        if let MessagePayload::Version(version) = &message.payload {
            if version.protocol_version != self.config.protocol_version {
                return Err(NetworkError::VersionMismatch);
            }
        }
        Ok(())
    }

    fn validate_block_message(&self, _message: &Message) -> Result<(), NetworkError> {
        // Add block-specific validation
        Ok(())
    }

    fn validate_transaction_message(&self, _message: &Message) -> Result<(), NetworkError> {
        // Add transaction-specific validation
        Ok(())
    }

    pub fn create_handshake(
        &self,
        sender: SocketAddr,
        node_id: &str,
        best_height: u64,
        best_hash: &str,
        public_key: &str,
    ) -> Message {
        let handshake = HandshakeData {
            protocol_version: self.config.protocol_version,
            network_id: self.config.network_id,
            node_id: node_id.to_string(),
            user_agent: "QuantumCoin/1.0.0".to_string(),
            services: 1, // Full node
            timestamp: Utc::now(),
            best_block_height: best_height,
            best_block_hash: best_hash.to_string(),
            public_key: public_key.to_string(),
        };

        Message::new(MessageType::Handshake, sender, MessagePayload::Handshake(handshake))
    }

    pub fn create_ping(&self, sender: SocketAddr) -> Message {
        let ping = PingData {
            nonce: rand::random(),
            timestamp: Utc::now(),
        };

        Message::new(MessageType::Ping, sender, MessagePayload::Ping(ping))
    }

    pub fn create_pong(&self, sender: SocketAddr, ping_nonce: u64) -> Message {
        let pong = PongData {
            nonce: ping_nonce,
            timestamp: Utc::now(),
        };

        Message::new(MessageType::Pong, sender, MessagePayload::Pong(pong))
    }
}
