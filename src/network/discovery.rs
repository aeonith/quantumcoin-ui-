use crate::network::{NetworkError, NetworkConfig};
use crate::network::peer::{PeerManager, PeerInfo};
use crate::network::protocol::{Message, MessageType, MessagePayload};
use serde::{Serialize, Deserialize};
use std::net::{SocketAddr, ToSocketAddrs, IpAddr};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use tokio::time::{sleep, interval};
use tokio::net::UdpSocket;
use std::sync::Arc;
use parking_lot::RwLock;
use rand::seq::SliceRandom;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerRecord {
    pub address: SocketAddr,
    pub last_seen: DateTime<Utc>,
    pub success_count: u32,
    pub failure_count: u32,
    pub services: u64,
    pub user_agent: String,
    pub protocol_version: u32,
    pub source: PeerSource,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PeerSource {
    Bootstrap,
    DnsSeeds,
    PeerExchange,
    Manual,
    Incoming,
}

impl PeerRecord {
    pub fn new(address: SocketAddr, source: PeerSource) -> Self {
        Self {
            address,
            last_seen: Utc::now(),
            success_count: 0,
            failure_count: 0,
            services: 0,
            user_agent: String::new(),
            protocol_version: 0,
            source,
        }
    }

    pub fn reliability_score(&self) -> f64 {
        if self.success_count + self.failure_count == 0 {
            0.5 // Neutral score for unknown peers
        } else {
            self.success_count as f64 / (self.success_count + self.failure_count) as f64
        }
    }

    pub fn is_stale(&self, max_age_hours: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.last_seen);
        age.num_hours() > max_age_hours as i64
    }
}

pub struct PeerDiscovery {
    config: NetworkConfig,
    peer_manager: Arc<PeerManager>,
    peer_database: Arc<RwLock<HashMap<SocketAddr, PeerRecord>>>,
    connection_attempts: Arc<RwLock<HashMap<SocketAddr, DateTime<Utc>>>>,
    bootstrap_completed: Arc<parking_lot::Mutex<bool>>,
}

impl PeerDiscovery {
    pub fn new(config: NetworkConfig, peer_manager: Arc<PeerManager>) -> Self {
        Self {
            config,
            peer_manager,
            peer_database: Arc::new(RwLock::new(HashMap::new())),
            connection_attempts: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_completed: Arc::new(parking_lot::Mutex::new(false)),
        }
    }

    pub async fn start_discovery(&self) -> Result<(), NetworkError> {
        println!("Starting peer discovery...");

        // Initial bootstrap
        if !*self.bootstrap_completed.lock() {
            self.bootstrap_peers().await?;
            *self.bootstrap_completed.lock() = true;
        }

        // Start periodic discovery tasks
        let discovery_clone = self.clone_discovery_data();
        tokio::spawn(async move {
            Self::run_periodic_discovery(discovery_clone).await;
        });

        Ok(())
    }

    async fn bootstrap_peers(&self) -> Result<(), NetworkError> {
        println!("Bootstrapping with {} seed nodes...", self.config.bootstrap_peers.len());

        // Add bootstrap peers
        for &address in &self.config.bootstrap_peers {
            self.add_peer_candidate(address, PeerSource::Bootstrap).await;
        }

        // DNS seed discovery
        if !self.config.dns_seeds.is_empty() {
            self.discover_dns_seeds().await?;
        }

        // Try to connect to initial peers
        self.connect_to_best_peers(8).await;

        Ok(())
    }

    async fn discover_dns_seeds(&self) -> Result<(), NetworkError> {
        println!("Querying DNS seeds...");

        for seed in &self.config.dns_seeds {
            match self.query_dns_seed(seed).await {
                Ok(addresses) => {
                    println!("Found {} peers from DNS seed: {}", addresses.len(), seed);
                    for address in addresses {
                        self.add_peer_candidate(address, PeerSource::DnsSeeds).await;
                    }
                }
                Err(e) => {
                    println!("Failed to query DNS seed {}: {}", seed, e);
                }
            }
        }

        Ok(())
    }

    async fn query_dns_seed(&self, seed: &str) -> Result<Vec<SocketAddr>, NetworkError> {
        let seed_with_port = if seed.contains(':') {
            seed.to_string()
        } else {
            format!("{}:{}", seed, self.config.listen_port)
        };

        let addresses: Result<Vec<SocketAddr>, _> = seed_with_port.to_socket_addrs()
            .map_err(|e| NetworkError::ConnectionFailed(format!("DNS resolution failed: {}", e)))?
            .collect::<Vec<_>>()
            .into();

        Ok(addresses.unwrap_or_default())
    }

    pub async fn add_peer_candidate(&self, address: SocketAddr, source: PeerSource) {
        // Don't add localhost or private IPs in mainnet
        if self.config.network_id == 0 && self.is_private_ip(&address.ip()) {
            return;
        }

        let mut database = self.peer_database.write();
        if let Some(record) = database.get_mut(&address) {
            record.last_seen = Utc::now();
        } else {
            database.insert(address, PeerRecord::new(address, source));
        }
    }

    fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_loopback() || 
                ipv4.is_private() || 
                ipv4.is_link_local() ||
                ipv4.is_broadcast()
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() || 
                ipv6.is_unspecified()
            }
        }
    }

    pub async fn connect_to_best_peers(&self, count: usize) {
        let candidates = self.get_best_peer_candidates(count * 2).await;
        let mut connected = 0;

        for candidate in candidates {
            if connected >= count {
                break;
            }

            // Check if we recently attempted to connect
            if self.recently_attempted(&candidate.address).await {
                continue;
            }

            // Check if already connected
            if self.peer_manager.get_peer(&candidate.address).await.is_some() {
                continue;
            }

            // Attempt connection
            match self.attempt_connection(candidate.address).await {
                Ok(_) => {
                    connected += 1;
                    self.mark_peer_success(candidate.address).await;
                }
                Err(e) => {
                    println!("Failed to connect to {}: {}", candidate.address, e);
                    self.mark_peer_failure(candidate.address).await;
                }
            }

            // Record connection attempt
            self.record_connection_attempt(candidate.address).await;
        }

        println!("Connected to {} new peers", connected);
    }

    async fn get_best_peer_candidates(&self, count: usize) -> Vec<PeerRecord> {
        let database = self.peer_database.read();
        let mut candidates: Vec<PeerRecord> = database
            .values()
            .filter(|record| !record.is_stale(24)) // Not older than 24 hours
            .cloned()
            .collect();

        // Sort by reliability score (descending)
        candidates.sort_by(|a, b| {
            b.reliability_score().partial_cmp(&a.reliability_score()).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Add some randomization to avoid always connecting to the same peers
        let mut rng = rand::thread_rng();
        if candidates.len() > count {
            let split_point = (count as f64 * 0.7) as usize; // Take 70% best, 30% random
            let (best, rest) = candidates.split_at_mut(split_point);
            rest.shuffle(&mut rng);
            
            let mut result = best.to_vec();
            result.extend_from_slice(&rest[..((count - split_point).min(rest.len()))]);
            result
        } else {
            candidates
        }
    }

    async fn recently_attempted(&self, address: &SocketAddr) -> bool {
        let attempts = self.connection_attempts.read();
        if let Some(last_attempt) = attempts.get(address) {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(*last_attempt);
            elapsed.num_minutes() < 5 // Don't retry within 5 minutes
        } else {
            false
        }
    }

    async fn attempt_connection(&self, address: SocketAddr) -> Result<(), NetworkError> {
        use tokio::time::timeout;
        use tokio::net::TcpStream;

        let connect_timeout = Duration::from_secs(self.config.connection_timeout_secs);
        
        match timeout(connect_timeout, TcpStream::connect(address)).await {
            Ok(Ok(stream)) => {
                // Connection successful, add to peer manager
                let peer = self.peer_manager.add_peer(address, false).await?;
                peer.connect(stream).await?;
                
                // Send handshake
                let handshake = self.create_handshake_message(address).await;
                peer.send_message(handshake).await?;
                
                Ok(())
            }
            Ok(Err(e)) => Err(NetworkError::ConnectionFailed(e.to_string())),
            Err(_) => Err(NetworkError::Timeout),
        }
    }

    async fn create_handshake_message(&self, _peer_address: SocketAddr) -> Message {
        // This would typically require access to blockchain state
        Message::new(
            MessageType::Handshake,
            SocketAddr::from(([127, 0, 0, 1], self.config.listen_port)),
            MessagePayload::Handshake(crate::network::protocol::HandshakeData {
                protocol_version: self.config.protocol_version,
                network_id: self.config.network_id,
                node_id: uuid::Uuid::new_v4().to_string(),
                user_agent: "QuantumCoin/1.0.0".to_string(),
                services: 1, // Full node
                timestamp: Utc::now(),
                best_block_height: 0, // Would get from blockchain
                best_block_hash: "genesis".to_string(), // Would get from blockchain
                public_key: "placeholder".to_string(), // Would use actual public key
            }),
        )
    }

    async fn record_connection_attempt(&self, address: SocketAddr) {
        let mut attempts = self.connection_attempts.write();
        attempts.insert(address, Utc::now());
    }

    async fn mark_peer_success(&self, address: SocketAddr) {
        let mut database = self.peer_database.write();
        if let Some(record) = database.get_mut(&address) {
            record.success_count += 1;
            record.last_seen = Utc::now();
        }
    }

    async fn mark_peer_failure(&self, address: SocketAddr) {
        let mut database = self.peer_database.write();
        if let Some(record) = database.get_mut(&address) {
            record.failure_count += 1;
        }
    }

    pub async fn handle_peer_exchange(&self, peers: Vec<PeerInfo>) {
        println!("Received {} peers from peer exchange", peers.len());
        
        for peer_info in peers {
            self.add_peer_candidate(peer_info.address, PeerSource::PeerExchange).await;
        }
    }

    pub async fn get_peers_for_sharing(&self, count: usize) -> Vec<PeerInfo> {
        let database = self.peer_database.read();
        let mut peers: Vec<PeerInfo> = database
            .values()
            .filter(|record| {
                !record.is_stale(1) && // Active within last hour
                record.reliability_score() > 0.5 // Good reliability
            })
            .map(|record| PeerInfo {
                address: record.address,
                last_seen: record.last_seen,
                services: record.services,
                user_agent: record.user_agent.clone(),
                version: record.protocol_version,
            })
            .collect();

        // Randomize and limit
        let mut rng = rand::thread_rng();
        peers.shuffle(&mut rng);
        peers.truncate(count);
        
        peers
    }

    pub async fn cleanup_stale_peers(&self) {
        let mut database = self.peer_database.write();
        let initial_count = database.len();
        
        database.retain(|_, record| !record.is_stale(72)); // Keep peers seen within 72 hours
        
        let removed = initial_count - database.len();
        if removed > 0 {
            println!("Cleaned up {} stale peer records", removed);
        }

        // Also cleanup old connection attempts
        let mut attempts = self.connection_attempts.write();
        let now = Utc::now();
        attempts.retain(|_, last_attempt| {
            now.signed_duration_since(*last_attempt).num_hours() < 1
        });
    }

    fn clone_discovery_data(&self) -> DiscoveryData {
        DiscoveryData {
            config: self.config.clone(),
            peer_manager: self.peer_manager.clone(),
            peer_database: self.peer_database.clone(),
            connection_attempts: self.connection_attempts.clone(),
        }
    }

    async fn run_periodic_discovery(data: DiscoveryData) {
        let mut discovery_interval = interval(Duration::from_secs(300)); // Every 5 minutes
        let mut cleanup_interval = interval(Duration::from_secs(3600)); // Every hour

        loop {
            tokio::select! {
                _ = discovery_interval.tick() => {
                    Self::periodic_peer_discovery(&data).await;
                }
                _ = cleanup_interval.tick() => {
                    Self::periodic_cleanup(&data).await;
                }
            }
        }
    }

    async fn periodic_peer_discovery(data: &DiscoveryData) {
        println!("Running periodic peer discovery...");
        
        let connected_count = data.peer_manager.get_connected_peers().await.len();
        let target_connections = data.config.max_outbound_peers / 2; // Maintain at least half capacity
        
        if connected_count < target_connections {
            let needed = target_connections - connected_count;
            
            // Try to connect to more peers
            let discovery = PeerDiscovery {
                config: data.config.clone(),
                peer_manager: data.peer_manager.clone(),
                peer_database: data.peer_database.clone(),
                connection_attempts: data.connection_attempts.clone(),
                bootstrap_completed: Arc::new(parking_lot::Mutex::new(true)),
            };
            
            discovery.connect_to_best_peers(needed).await;
        }
    }

    async fn periodic_cleanup(data: &DiscoveryData) {
        println!("Running periodic cleanup...");
        
        let discovery = PeerDiscovery {
            config: data.config.clone(),
            peer_manager: data.peer_manager.clone(),
            peer_database: data.peer_database.clone(),
            connection_attempts: data.connection_attempts.clone(),
            bootstrap_completed: Arc::new(parking_lot::Mutex::new(true)),
        };
        
        discovery.cleanup_stale_peers().await;
        data.peer_manager.cleanup_expired_blacklist().await;
        data.peer_manager.cleanup_stale_peers().await;
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("known_peers".to_string(), self.peer_database.read().len());
        stats.insert("connection_attempts".to_string(), self.connection_attempts.read().len());
        stats
    }
}

#[derive(Clone)]
struct DiscoveryData {
    config: NetworkConfig,
    peer_manager: Arc<PeerManager>,
    peer_database: Arc<RwLock<HashMap<SocketAddr, PeerRecord>>>,
    connection_attempts: Arc<RwLock<HashMap<SocketAddr, DateTime<Utc>>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_peer_record() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut record = PeerRecord::new(addr, PeerSource::Bootstrap);
        
        assert_eq!(record.reliability_score(), 0.5);
        
        record.success_count = 3;
        record.failure_count = 1;
        assert_eq!(record.reliability_score(), 0.75);
    }

    #[tokio::test]
    async fn test_peer_discovery_creation() {
        let config = NetworkConfig::default();
        let peer_manager = Arc::new(PeerManager::new(config.clone()));
        let discovery = PeerDiscovery::new(config, peer_manager);
        
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        discovery.add_peer_candidate(addr, PeerSource::Manual).await;
        
        assert_eq!(discovery.peer_database.read().len(), 1);
    }
}
