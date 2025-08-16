// DNS seed discovery for fresh node sync
use crate::network::{ChainSpec, PeerManager, NetworkMetrics};
use anyhow::Result;
use hickory_resolver::{AsyncResolver, TokioAsyncResolver};
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Production DNS seed discovery system
pub struct DnsDiscovery {
    chain_spec: Arc<ChainSpec>,
    peer_manager: Arc<PeerManager>,
    metrics: Arc<NetworkMetrics>,
    resolver: AsyncResolver,
    last_discovery: Arc<tokio::sync::RwLock<Instant>>,
    discovered_addresses: Arc<tokio::sync::RwLock<HashSet<SocketAddr>>>,
}

impl DnsDiscovery {
    pub fn new(
        chain_spec: Arc<ChainSpec>,
        peer_manager: Arc<PeerManager>,
        metrics: Arc<NetworkMetrics>,
    ) -> Self {
        // Create high-performance DNS resolver
        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(5);
        opts.attempts = 3;
        opts.rotate = true;
        opts.use_hosts_file = false;

        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare_tls(),
            opts,
        );

        Self {
            chain_spec,
            peer_manager,
            metrics,
            resolver,
            last_discovery: Arc::new(tokio::sync::RwLock::new(Instant::now())),
            discovered_addresses: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
        }
    }

    /// Start continuous DNS discovery
    pub async fn start(&self) -> Result<()> {
        log::info!("Starting DNS seed discovery");
        
        // Initial discovery
        self.discover_seeds().await?;
        
        // Start continuous discovery loop
        let discovery = self.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(300)).await; // Every 5 minutes
                
                if let Err(e) = discovery.discover_seeds().await {
                    log::warn!("DNS discovery error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Resolve all DNS seeds and return unique addresses
    pub async fn resolve_all_seeds(&self) -> Result<Vec<SocketAddr>> {
        let mut all_addresses = HashSet::new();
        
        log::info!("Resolving {} DNS seeds", self.chain_spec.dns_seeds.len());
        
        for seed in &self.chain_spec.dns_seeds {
            match self.resolve_seed(seed).await {
                Ok(addresses) => {
                    log::info!("Resolved {} addresses from seed {}", addresses.len(), seed);
                    all_addresses.extend(addresses);
                }
                Err(e) => {
                    log::warn!("Failed to resolve seed {}: {}", seed, e);
                    self.metrics.increment_dns_failure().await;
                }
            }
        }

        // Add discovered addresses from previous runs
        let discovered = self.discovered_addresses.read().await;
        all_addresses.extend(discovered.iter().copied());
        drop(discovered);

        Ok(all_addresses.into_iter().collect())
    }

    /// Resolve a single DNS seed
    async fn resolve_seed(&self, seed: &str) -> Result<Vec<SocketAddr>> {
        let start_time = Instant::now();
        let mut addresses = Vec::new();
        
        // Try multiple DNS record types for maximum coverage
        
        // 1. Standard A/AAAA records
        match self.resolve_a_records(seed).await {
            Ok(mut addrs) => addresses.append(&mut addrs),
            Err(e) => log::debug!("A record resolution failed for {}: {}", seed, e),
        }
        
        // 2. SRV records for service discovery
        match self.resolve_srv_records(seed).await {
            Ok(mut addrs) => addresses.append(&mut addrs),
            Err(e) => log::debug!("SRV record resolution failed for {}: {}", seed, e),
        }
        
        // 3. TXT records for additional peer information
        match self.resolve_txt_records(seed).await {
            Ok(mut addrs) => addresses.append(&mut addrs),
            Err(e) => log::debug!("TXT record resolution failed for {}: {}", seed, e),
        }

        let resolution_time = start_time.elapsed();
        self.metrics.record_dns_resolution_time(resolution_time).await;
        
        if addresses.is_empty() {
            return Err(anyhow::anyhow!("No addresses resolved for seed {}", seed));
        }

        // Remove duplicates and validate addresses
        let mut unique_addresses = HashSet::new();
        for addr in addresses {
            if self.validate_address(&addr).await {
                unique_addresses.insert(addr);
            }
        }

        Ok(unique_addresses.into_iter().collect())
    }

    /// Resolve A and AAAA records
    async fn resolve_a_records(&self, hostname: &str) -> Result<Vec<SocketAddr>> {
        let mut addresses = Vec::new();
        
        // IPv4 A records
        match self.resolver.ipv4_lookup(hostname).await {
            Ok(lookup) => {
                for ip in lookup.iter() {
                    addresses.push(SocketAddr::new(
                        IpAddr::V4(*ip),
                        self.chain_spec.default_port,
                    ));
                }
            }
            Err(e) => log::debug!("IPv4 lookup failed for {}: {}", hostname, e),
        }
        
        // IPv6 AAAA records
        match self.resolver.ipv6_lookup(hostname).await {
            Ok(lookup) => {
                for ip in lookup.iter() {
                    addresses.push(SocketAddr::new(
                        IpAddr::V6(*ip),
                        self.chain_spec.default_port,
                    ));
                }
            }
            Err(e) => log::debug!("IPv6 lookup failed for {}: {}", hostname, e),
        }

        Ok(addresses)
    }

    /// Resolve SRV records for service discovery
    async fn resolve_srv_records(&self, hostname: &str) -> Result<Vec<SocketAddr>> {
        let srv_name = format!("_quantumcoin._tcp.{}", hostname);
        let mut addresses = Vec::new();
        
        match self.resolver.srv_lookup(&srv_name).await {
            Ok(lookup) => {
                for srv in lookup.iter() {
                    let target = srv.target().to_string();
                    let port = srv.port();
                    
                    // Remove trailing dot from DNS name
                    let target = target.trim_end_matches('.');
                    
                    // Resolve the target hostname
                    if let Ok(target_addrs) = self.resolve_a_records(target).await {
                        for mut addr in target_addrs {
                            addr.set_port(port);
                            addresses.push(addr);
                        }
                    }
                }
            }
            Err(e) => return Err(anyhow::anyhow!("SRV lookup failed: {}", e)),
        }

        Ok(addresses)
    }

    /// Resolve TXT records for additional peer info
    async fn resolve_txt_records(&self, hostname: &str) -> Result<Vec<SocketAddr>> {
        let txt_name = format!("qtc-peers.{}", hostname);
        let mut addresses = Vec::new();
        
        match self.resolver.txt_lookup(&txt_name).await {
            Ok(lookup) => {
                for txt_record in lookup.iter() {
                    for txt_data in txt_record.iter() {
                        if let Ok(text) = std::str::from_utf8(txt_data) {
                            // Parse peer addresses from TXT records
                            // Format: "ip:port,ip:port,..."
                            for addr_str in text.split(',') {
                                if let Ok(addr) = addr_str.trim().parse::<SocketAddr>() {
                                    if self.validate_address(&addr).await {
                                        addresses.push(addr);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(anyhow::anyhow!("TXT lookup failed: {}", e)),
        }

        Ok(addresses)
    }

    /// Validate that an address is suitable for connection
    async fn validate_address(&self, addr: &SocketAddr) -> bool {
        // Check if it's a valid routable address
        match addr.ip() {
            IpAddr::V4(ip) => {
                // Reject local, broadcast, multicast, etc.
                !ip.is_loopback() 
                    && !ip.is_private() 
                    && !ip.is_multicast() 
                    && !ip.is_broadcast()
                    && !ip.is_documentation()
            }
            IpAddr::V6(ip) => {
                !ip.is_loopback() 
                    && !ip.is_multicast() 
                    && !ip.is_unspecified()
            }
        }
    }

    /// Continuous seed discovery
    async fn discover_seeds(&self) -> Result<()> {
        let now = Instant::now();
        let last_discovery = *self.last_discovery.read().await;
        
        // Rate limiting: only discover every 5 minutes
        if now.duration_since(last_discovery) < Duration::from_secs(300) {
            return Ok(());
        }
        
        log::info!("Running DNS seed discovery...");
        let start_time = Instant::now();
        
        let new_addresses = self.resolve_all_seeds().await?;
        let discovery_time = start_time.elapsed();
        
        // Update discovered addresses
        let mut discovered = self.discovered_addresses.write().await;
        for addr in &new_addresses {
            discovered.insert(*addr);
        }
        
        // Prune old addresses (keep last 1000)
        if discovered.len() > 1000 {
            let addresses_to_remove = discovered.len() - 1000;
            let old_addresses: Vec<_> = discovered.iter().take(addresses_to_remove).copied().collect();
            for addr in old_addresses {
                discovered.remove(&addr);
            }
        }
        drop(discovered);
        
        // Update last discovery time
        *self.last_discovery.write().await = now;
        
        self.metrics.record_dns_discovery(new_addresses.len(), discovery_time).await;
        
        log::info!(
            "DNS discovery completed: {} addresses in {:?}",
            new_addresses.len(),
            discovery_time
        );
        
        // Attempt connections to new addresses
        for addr in new_addresses.into_iter().take(10) {
            let _ = self.peer_manager.try_connect_to_peer(addr).await;
        }

        Ok(())
    }
}

impl Clone for DnsDiscovery {
    fn clone(&self) -> Self {
        Self {
            chain_spec: self.chain_spec.clone(),
            peer_manager: self.peer_manager.clone(),
            metrics: self.metrics.clone(),
            resolver: self.resolver.clone(),
            last_discovery: self.last_discovery.clone(),
            discovered_addresses: self.discovered_addresses.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[tokio::test]
    async fn test_address_validation() {
        let discovery = create_test_discovery();
        
        // Valid addresses
        assert!(discovery.validate_address(&"8.8.8.8:8333".parse().unwrap()).await);
        assert!(discovery.validate_address(&"[2001:4860:4860::8888]:8333".parse().unwrap()).await);
        
        // Invalid addresses
        assert!(!discovery.validate_address(&"127.0.0.1:8333".parse().unwrap()).await);
        assert!(!discovery.validate_address(&"192.168.1.1:8333".parse().unwrap()).await);
        assert!(!discovery.validate_address(&"[::1]:8333".parse().unwrap()).await);
    }
    
    fn create_test_discovery() -> DnsDiscovery {
        // Create test instances - simplified for testing
        let chain_spec = Arc::new(ChainSpec::default());
        let peer_manager = Arc::new(PeerManager::new_test());
        let metrics = Arc::new(NetworkMetrics::new());
        
        DnsDiscovery::new(chain_spec, peer_manager, metrics)
    }
}
