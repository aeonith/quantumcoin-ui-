//! Basic network layer for QuantumCoin node

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Network errors
#[derive(thiserror::Error, Debug)]
pub enum NetworkError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    /// Invalid message
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Ping message
    Ping { nonce: u64 },
    
    /// Pong response
    Pong { nonce: u64 },
    
    /// Get peer list
    GetPeers,
    
    /// Peer list response
    Peers { addresses: Vec<SocketAddr> },
    
    /// New block announcement
    NewBlock { block_hash: [u8; 32] },
    
    /// New transaction announcement
    NewTransaction { tx_hash: [u8; 32] },
}

/// Network peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer address
    pub address: SocketAddr,
    
    /// Last seen timestamp
    pub last_seen: u64,
    
    /// Connection status
    pub connected: bool,
}

/// Basic network manager
pub struct NetworkManager {
    /// Known peers
    peers: Vec<PeerInfo>,
}

impl NetworkManager {
    /// Create new network manager
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }
    
    /// Add a peer
    pub fn add_peer(&mut self, address: SocketAddr) {
        let peer = PeerInfo {
            address,
            last_seen: chrono::Utc::now().timestamp() as u64,
            connected: false,
        };
        self.peers.push(peer);
    }
    
    /// Get connected peers
    pub fn connected_peers(&self) -> Vec<&PeerInfo> {
        self.peers.iter().filter(|p| p.connected).collect()
    }
    
    /// Broadcast message to all connected peers
    pub async fn broadcast(&self, _message: NetworkMessage) -> Result<(), NetworkError> {
        // Placeholder implementation
        // In a real implementation, this would send the message to all connected peers
        Ok(())
    }
    
    /// Handle incoming message
    pub async fn handle_message(
        &mut self,
        _peer: SocketAddr,
        _message: NetworkMessage,
    ) -> Result<Option<NetworkMessage>, NetworkError> {
        // Placeholder implementation
        // Would handle different message types and return appropriate responses
        Ok(None)
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[test]
    fn test_add_peer() {
        let mut network = NetworkManager::new();
        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
        
        network.add_peer(addr);
        assert_eq!(network.peers.len(), 1);
        assert_eq!(network.peers[0].address, addr);
    }
    
    #[test]
    fn test_connected_peers() {
        let mut network = NetworkManager::new();
        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
        
        network.add_peer(addr);
        
        // Initially no connected peers
        assert_eq!(network.connected_peers().len(), 0);
        
        // Mark as connected
        network.peers[0].connected = true;
        assert_eq!(network.connected_peers().len(), 1);
    }
}
