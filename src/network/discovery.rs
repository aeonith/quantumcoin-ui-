use std::net::SocketAddr;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use tracing::{info, warn, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDiscovery {
    known_peers: HashSet<SocketAddr>,
    seed_nodes: Vec<SocketAddr>,
    last_discovery: u64,
    discovery_interval: u64, // seconds
}

impl PeerDiscovery {
    pub fn new(seed_nodes: Vec<SocketAddr>) -> Self {
        Self {
            known_peers: HashSet::new(),
            seed_nodes,
            last_discovery: 0,
            discovery_interval: 3600, // 1 hour
        }
    }
    
    pub fn add_peer(&mut self, addr: SocketAddr) {
        self.known_peers.insert(addr);
        debug!("Added peer to discovery: {}", addr);
    }
    
    pub fn remove_peer(&mut self, addr: &SocketAddr) {
        self.known_peers.remove(addr);
        debug!("Removed peer from discovery: {}", addr);
    }
    
    pub fn get_random_peers(&self, count: usize) -> Vec<SocketAddr> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        let mut peers: Vec<SocketAddr> = self.known_peers.iter().copied().collect();
        peers.shuffle(&mut rng);
        peers.truncate(count);
        peers
    }
    
    pub fn should_discover(&self) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now - self.last_discovery > self.discovery_interval
    }
    
    pub fn mark_discovery_complete(&mut self) {
        self.last_discovery = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    }
    
    pub fn get_seed_nodes(&self) -> &Vec<SocketAddr> {
        &self.seed_nodes
    }
    
    pub fn peer_count(&self) -> usize {
        self.known_peers.len()
    }
    
    pub async fn discover_peers(&mut self) -> Result<Vec<SocketAddr>> {
        info!("Starting peer discovery");
        let mut discovered = Vec::new();
        
        // Try connecting to seed nodes first
        for seed in &self.seed_nodes.clone() {
            if !self.known_peers.contains(seed) {
                // Try to connect and get peer list
                if let Ok(peers) = self.request_peers_from(*seed).await {
                    for peer in peers {
                        if !self.known_peers.contains(&peer) {
                            self.add_peer(peer);
                            discovered.push(peer);
                        }
                    }
                }
            }
        }
        
        self.mark_discovery_complete();
        info!("Discovered {} new peers", discovered.len());
        Ok(discovered)
    }
    
    async fn request_peers_from(&self, addr: SocketAddr) -> Result<Vec<SocketAddr>> {
        // This would implement the actual network request to get peers
        // For now, return empty list
        warn!("Peer request from {} not implemented", addr);
        Ok(Vec::new())
    }
    
    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, data)?;
        Ok(())
    }
    
    pub fn load_from_file(filename: &str) -> Result<Self> {
        let data = std::fs::read_to_string(filename)?;
        let discovery = serde_json::from_str(&data)?;
        Ok(discovery)
    }
}

impl Default for PeerDiscovery {
    fn default() -> Self {
        // Default seed nodes (would be real addresses in production)
        let seed_nodes = vec![
            "127.0.0.1:8333".parse().unwrap(),
            "127.0.0.1:8334".parse().unwrap(),
        ];
        Self::new(seed_nodes)
    }
}
