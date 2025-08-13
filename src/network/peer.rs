use crate::network::{NetworkMessage, MessageHeader};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub addr: SocketAddr,
    pub node_id: String,
    pub version: u32,
    pub chain_height: u64,
    pub last_seen: u64,
    pub connected: bool,
}

pub struct Peer {
    pub info: PeerInfo,
    stream: Option<TcpStream>,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            info: PeerInfo {
                addr,
                node_id: String::new(),
                version: 0,
                chain_height: 0,
                last_seen: 0,
                connected: false,
            },
            stream: None,
        }
    }
    
    pub async fn connect(&mut self) -> Result<()> {
        let stream = TcpStream::connect(self.info.addr).await?;
        self.stream = Some(stream);
        self.info.connected = true;
        self.info.last_seen = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(())
    }
    
    pub async fn disconnect(&mut self) {
        if let Some(stream) = &mut self.stream {
            let _ = stream.shutdown().await;
        }
        self.stream = None;
        self.info.connected = false;
    }
    
    pub async fn send_message(&mut self, message: &NetworkMessage) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            let payload = message.serialize()?;
            let checksum = self.calculate_checksum(&payload);
            
            let header = MessageHeader {
                magic: MessageHeader::MAGIC,
                command: self.message_type_to_command(message),
                length: payload.len() as u32,
                checksum,
            };
            
            // Send header
            stream.write_all(&header.to_bytes()).await?;
            // Send payload
            stream.write_all(&payload).await?;
            stream.flush().await?;
            
            self.info.last_seen = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            Ok(())
        } else {
            Err(anyhow!("Not connected"))
        }
    }
    
    pub async fn receive_message(&mut self) -> Result<NetworkMessage> {
        if let Some(stream) = &mut self.stream {
            // Read header
            let mut header_bytes = [0u8; MessageHeader::SIZE];
            stream.read_exact(&mut header_bytes).await?;
            
            let header = MessageHeader::from_bytes(&header_bytes)
                .map_err(|e| anyhow!("Invalid header: {}", e))?;
            
            // Read payload
            let mut payload = vec![0u8; header.length as usize];
            stream.read_exact(&mut payload).await?;
            
            // Verify checksum
            let calculated_checksum = self.calculate_checksum(&payload);
            if calculated_checksum != header.checksum {
                return Err(anyhow!("Checksum mismatch"));
            }
            
            let message = NetworkMessage::deserialize(&payload)?;
            self.info.last_seen = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            
            Ok(message)
        } else {
            Err(anyhow!("Not connected"))
        }
    }
    
    pub async fn handshake(&mut self, our_version: u32, our_node_id: &str, our_height: u64) -> Result<bool> {
        // Send handshake
        let handshake = NetworkMessage::Handshake {
            version: our_version,
            node_id: our_node_id.to_string(),
            chain_height: our_height,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        
        self.send_message(&handshake).await?;
        
        // Wait for response
        match self.receive_message().await? {
            NetworkMessage::HandshakeAck { accepted, .. } => {
                if accepted {
                    // Get peer info from their handshake
                    match self.receive_message().await? {
                        NetworkMessage::Handshake { version, node_id, chain_height, .. } => {
                            self.info.version = version;
                            self.info.node_id = node_id;
                            self.info.chain_height = chain_height;
                            
                            // Send our ack
                            let ack = NetworkMessage::HandshakeAck {
                                accepted: true,
                                peer_list: vec![], // TODO: Add known peers
                            };
                            self.send_message(&ack).await?;
                            
                            Ok(true)
                        }
                        _ => Err(anyhow!("Expected handshake from peer")),
                    }
                } else {
                    Err(anyhow!("Handshake rejected by peer"))
                }
            }
            _ => Err(anyhow!("Expected handshake ack")),
        }
    }
    
    fn calculate_checksum(&self, data: &[u8]) -> u32 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }
    
    fn message_type_to_command(&self, message: &NetworkMessage) -> u8 {
        match message {
            NetworkMessage::Handshake { .. } => 1,
            NetworkMessage::HandshakeAck { .. } => 2,
            NetworkMessage::GetBlocks { .. } => 3,
            NetworkMessage::Blocks(_) => 4,
            NetworkMessage::NewBlock(_) => 5,
            NetworkMessage::GetBlock(_) => 6,
            NetworkMessage::Block(_) => 7,
            NetworkMessage::NewTransaction(_) => 8,
            NetworkMessage::GetMempool => 9,
            NetworkMessage::Mempool(_) => 10,
            NetworkMessage::GetChainInfo => 11,
            NetworkMessage::ChainInfo { .. } => 12,
            NetworkMessage::GetPeers => 13,
            NetworkMessage::Peers(_) => 14,
            NetworkMessage::Ping(_) => 15,
            NetworkMessage::Pong(_) => 16,
            NetworkMessage::Error(_) => 255,
        }
    }
    
    pub fn is_alive(&self) -> bool {
        if !self.info.connected {
            return false;
        }
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now - self.info.last_seen < 300 // 5 minutes timeout
    }
}
