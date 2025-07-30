use crate::network::{NetworkError, NetworkConfig};
use crate::network::protocol::{Message, MessageType, MessagePayload, ProtocolHandler};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub node_id: String,
    pub user_agent: String,
    pub protocol_version: u32,
    pub services: u64,
    pub best_height: u64,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub ping_time: Option<Duration>,
    pub is_inbound: bool,
    pub reputation_score: i32,
}

impl PeerInfo {
    pub fn new(address: SocketAddr, is_inbound: bool) -> Self {
        Self {
            address,
            node_id: String::new(),
            user_agent: String::new(),
            protocol_version: 0,
            services: 0,
            best_height: 0,
            connected_at: Utc::now(),
            last_seen: Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
            ping_time: None,
            is_inbound,
            reputation_score: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RateLimiter {
    requests: Arc<Mutex<Vec<Instant>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_minutes: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            max_requests,
            window_duration: Duration::from_secs(window_minutes * 60),
        }
    }

    pub fn is_allowed(&self) -> bool {
        let now = Instant::now();
        let mut requests = self.requests.lock();
        
        // Remove old requests outside the window
        requests.retain(|&time| now.duration_since(time) < self.window_duration);
        
        if requests.len() >= self.max_requests as usize {
            false
        } else {
            requests.push(now);
            true
        }
    }
}

pub struct Peer {
    pub info: RwLock<PeerInfo>,
    pub stream: Arc<Mutex<Option<TcpStream>>>,
    pub message_sender: mpsc::UnboundedSender<Message>,
    pub message_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<Message>>>>,
    pub rate_limiter: RateLimiter,
    pub is_connected: Arc<parking_lot::Mutex<bool>>,
    pub last_ping: Arc<Mutex<Option<Instant>>>,
}

impl Peer {
    pub fn new(address: SocketAddr, is_inbound: bool, config: &NetworkConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        Self {
            info: RwLock::new(PeerInfo::new(address, is_inbound)),
            stream: Arc::new(Mutex::new(None)),
            message_sender: tx,
            message_receiver: Arc::new(Mutex::new(Some(rx))),
            rate_limiter: RateLimiter::new(
                config.rate_limit_requests_per_minute,
                config.blacklist_duration_hours,
            ),
            is_connected: Arc::new(parking_lot::Mutex::new(false)),
            last_ping: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self, stream: TcpStream) -> Result<(), NetworkError> {
        {
            let mut connection = self.stream.lock();
            *connection = Some(stream);
        }
        
        {
            let mut connected = self.is_connected.lock();
            *connected = true;
        }

        Ok(())
    }

    pub async fn disconnect(&self) {
        {
            let mut connection = self.stream.lock();
            if let Some(mut stream) = connection.take() {
                let _ = stream.shutdown().await;
            }
        }
        
        {
            let mut connected = self.is_connected.lock();
            *connected = false;
        }
    }

    pub fn is_connected(&self) -> bool {
        *self.is_connected.lock()
    }

    pub async fn send_message(&self, message: Message) -> Result<(), NetworkError> {
        if !self.rate_limiter.is_allowed() {
            return Err(NetworkError::RateLimited);
        }

        self.message_sender
            .send(message)
            .map_err(|_| NetworkError::ConnectionFailed("Channel closed".to_string()))?;

        // Update bytes sent
        {
            let mut info = self.info.write().await;
            info.bytes_sent += message.to_bytes().len() as u64;
            info.last_seen = Utc::now();
        }

        Ok(())
    }

    pub async fn handle_message(&self, message: Message) -> Result<Option<Message>, NetworkError> {
        // Update bytes received
        {
            let mut info = self.info.write().await;
            info.bytes_received += message.to_bytes().len() as u64;
            info.last_seen = Utc::now();
        }

        match message.message_type {
            MessageType::Ping => {
                if let MessagePayload::Ping(ping_data) = message.payload {
                    let pong = Message::new(
                        MessageType::Pong,
                        message.sender,
                        MessagePayload::Pong(crate::network::protocol::PongData {
                            nonce: ping_data.nonce,
                            timestamp: Utc::now(),
                        }),
                    );
                    return Ok(Some(pong));
                }
            }
            MessageType::Pong => {
                if let Some(ping_time) = self.last_ping.lock().take() {
                    let duration = ping_time.elapsed();
                    let mut info = self.info.write().await;
                    info.ping_time = Some(duration);
                }
            }
            MessageType::Handshake => {
                if let MessagePayload::Handshake(handshake) = message.payload {
                    let mut info = self.info.write().await;
                    info.node_id = handshake.node_id;
                    info.user_agent = handshake.user_agent;
                    info.protocol_version = handshake.protocol_version;
                    info.services = handshake.services;
                    info.best_height = handshake.best_block_height;
                }
            }
            _ => {}
        }

        Ok(None)
    }

    pub async fn ping(&self) -> Result<(), NetworkError> {
        {
            let mut last_ping = self.last_ping.lock();
            *last_ping = Some(Instant::now());
        }

        let ping = Message::new(
            MessageType::Ping,
            self.info.read().await.address,
            MessagePayload::Ping(crate::network::protocol::PingData {
                nonce: rand::random(),
                timestamp: Utc::now(),
            }),
        );

        self.send_message(ping).await
    }

    pub fn update_reputation(&self, delta: i32) {
        tokio::spawn({
            let info = self.info.clone();
            async move {
                let mut peer_info = info.write().await;
                peer_info.reputation_score += delta;
                // Cap reputation between -100 and 100
                peer_info.reputation_score = peer_info.reputation_score.clamp(-100, 100);
            }
        });
    }

    pub async fn is_misbehaving(&self) -> bool {
        self.info.read().await.reputation_score < -50
    }
}

pub struct PeerManager {
    peers: Arc<DashMap<SocketAddr, Arc<Peer>>>,
    blacklist: Arc<DashMap<SocketAddr, DateTime<Utc>>>,
    config: NetworkConfig,
    protocol_handler: ProtocolHandler,
}

impl PeerManager {
    pub fn new(config: NetworkConfig) -> Self {
        let protocol_handler = ProtocolHandler::new(config.clone());
        
        Self {
            peers: Arc::new(DashMap::new()),
            blacklist: Arc::new(DashMap::new()),
            config,
            protocol_handler,
        }
    }

    pub async fn add_peer(&self, address: SocketAddr, is_inbound: bool) -> Result<Arc<Peer>, NetworkError> {
        // Check blacklist
        if self.is_blacklisted(&address).await {
            return Err(NetworkError::Blacklisted);
        }

        // Check peer limits
        let current_peers = self.peers.len();
        let inbound_count = self.get_inbound_peer_count().await;
        let outbound_count = current_peers - inbound_count;

        if current_peers >= self.config.max_peers {
            return Err(NetworkError::ConnectionFailed("Max peers reached".to_string()));
        }

        if is_inbound && inbound_count >= self.config.max_inbound_peers {
            return Err(NetworkError::ConnectionFailed("Max inbound peers reached".to_string()));
        }

        if !is_inbound && outbound_count >= self.config.max_outbound_peers {
            return Err(NetworkError::ConnectionFailed("Max outbound peers reached".to_string()));
        }

        let peer = Arc::new(Peer::new(address, is_inbound, &self.config));
        self.peers.insert(address, peer.clone());

        Ok(peer)
    }

    pub async fn remove_peer(&self, address: &SocketAddr) {
        if let Some((_, peer)) = self.peers.remove(address) {
            peer.disconnect().await;
        }
    }

    pub async fn blacklist_peer(&self, address: SocketAddr, reason: &str) {
        println!("Blacklisting peer {} for reason: {}", address, reason);
        
        let expiry = Utc::now() + chrono::Duration::hours(self.config.blacklist_duration_hours as i64);
        self.blacklist.insert(address, expiry);
        
        // Remove from active peers
        self.remove_peer(&address).await;
    }

    pub async fn is_blacklisted(&self, address: &SocketAddr) -> bool {
        if let Some(expiry) = self.blacklist.get(address) {
            if Utc::now() < *expiry {
                true
            } else {
                self.blacklist.remove(address);
                false
            }
        } else {
            false
        }
    }

    pub async fn get_peer(&self, address: &SocketAddr) -> Option<Arc<Peer>> {
        self.peers.get(address).map(|peer| peer.clone())
    }

    pub async fn get_all_peers(&self) -> Vec<Arc<Peer>> {
        self.peers.iter().map(|entry| entry.value().clone()).collect()
    }

    pub async fn get_connected_peers(&self) -> Vec<Arc<Peer>> {
        let mut connected = Vec::new();
        for peer in self.peers.iter() {
            if peer.is_connected() {
                connected.push(peer.value().clone());
            }
        }
        connected
    }

    pub async fn get_inbound_peer_count(&self) -> usize {
        let mut count = 0;
        for peer in self.peers.iter() {
            let info = peer.info.read().await;
            if info.is_inbound {
                count += 1;
            }
        }
        count
    }

    pub async fn broadcast_message(&self, message: Message) -> Result<usize, NetworkError> {
        let peers = self.get_connected_peers().await;
        let mut success_count = 0;

        for peer in peers {
            if peer.send_message(message.clone()).await.is_ok() {
                success_count += 1;
            }
        }

        Ok(success_count)
    }

    pub async fn send_to_peer(&self, address: &SocketAddr, message: Message) -> Result<(), NetworkError> {
        if let Some(peer) = self.get_peer(address).await {
            peer.send_message(message).await
        } else {
            Err(NetworkError::ConnectionFailed("Peer not found".to_string()))
        }
    }

    pub async fn cleanup_expired_blacklist(&self) {
        let now = Utc::now();
        self.blacklist.retain(|_, expiry| now < *expiry);
    }

    pub async fn cleanup_stale_peers(&self) {
        let stale_timeout = Duration::from_secs(self.config.connection_timeout_secs * 2);
        let mut to_remove = Vec::new();

        for peer in self.peers.iter() {
            let info = peer.info.read().await;
            let elapsed = Utc::now().signed_duration_since(info.last_seen);
            
            if elapsed.to_std().unwrap_or(Duration::MAX) > stale_timeout {
                to_remove.push(info.address);
            }
        }

        for address in to_remove {
            self.remove_peer(&address).await;
        }
    }

    pub async fn ping_all_peers(&self) {
        for peer in self.peers.iter() {
            if peer.is_connected() {
                let _ = peer.ping().await;
            }
        }
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_peers".to_string(), self.peers.len());
        stats.insert("blacklisted_peers".to_string(), self.blacklist.len());
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_peer_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let config = NetworkConfig::default();
        let peer = Peer::new(addr, false, &config);
        
        assert!(!peer.is_connected());
        assert_eq!(peer.info.read().await.address, addr);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, 1);
        
        assert!(limiter.is_allowed());
        assert!(limiter.is_allowed());
        assert!(!limiter.is_allowed()); // Should be rate limited
    }

    #[tokio::test]
    async fn test_peer_manager() {
        let config = NetworkConfig::default();
        let manager = PeerManager::new(config);
        
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let peer = manager.add_peer(addr, false).await.unwrap();
        
        assert_eq!(manager.peers.len(), 1);
        assert!(manager.get_peer(&addr).await.is_some());
    }
}
