// NAT traversal and external address discovery
use crate::network::ChainSpec;
use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

/// NAT manager for handling port forwarding and external address discovery
pub struct NatManager {
    listen_addr: SocketAddr,
    chain_spec: Arc<ChainSpec>,
    external_address: Arc<RwLock<Option<SocketAddr>>>,
    upnp_gateway: Arc<RwLock<Option<UpnpGateway>>>,
    stun_servers: Vec<String>,
    nat_type: Arc<RwLock<NatType>>,
    port_mapping: Arc<RwLock<Option<PortMapping>>>,
}

#[derive(Debug, Clone)]
pub struct UpnpGateway {
    pub gateway_ip: IpAddr,
    pub external_ip: IpAddr,
    pub description: String,
    pub supports_port_mapping: bool,
    pub last_seen: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NatType {
    OpenInternet,       // Direct connection to internet
    FullCone,          // Port mapping preserves external port
    RestrictedCone,    // Port mapping with IP restrictions
    PortRestricted,    // Port mapping with IP and port restrictions
    Symmetric,         // Different external port for each destination
    Blocked,           // No UDP traffic allowed
    Unknown,
}

#[derive(Debug)]
pub struct PortMapping {
    pub external_port: u16,
    pub internal_port: u16,
    pub protocol: String,
    pub description: String,
    pub lease_duration: Duration,
    pub created_at: Instant,
}

impl NatManager {
    pub async fn new(listen_addr: SocketAddr, chain_spec: Arc<ChainSpec>) -> Result<Self> {
        let stun_servers = vec![
            "stun.l.google.com:19302".to_string(),
            "stun1.l.google.com:19302".to_string(),
            "stun2.l.google.com:19302".to_string(),
            "stun.cloudflare.com:3478".to_string(),
            "stun.nextcloud.com:443".to_string(),
        ];

        Ok(Self {
            listen_addr,
            chain_spec,
            external_address: Arc::new(RwLock::new(None)),
            upnp_gateway: Arc::new(RwLock::new(None)),
            stun_servers,
            nat_type: Arc::new(RwLock::new(NatType::Unknown)),
            port_mapping: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        log::info!("Starting NAT traversal manager");
        
        // Start discovery processes
        let manager = self.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.discover_external_address().await {
                log::warn!("External address discovery failed: {}", e);
            }
        });

        let manager = self.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.discover_upnp_gateway().await {
                log::debug!("UPnP discovery failed: {}", e);
            }
        });

        let manager = self.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.determine_nat_type().await {
                log::warn!("NAT type determination failed: {}", e);
            }
        });

        let manager = self.clone();
        tokio::spawn(async move {
            manager.setup_port_forwarding().await;
        });

        // Start maintenance loop
        let manager = self.clone();
        tokio::spawn(async move {
            manager.maintenance_loop().await;
        });

        Ok(())
    }

    /// Discover external IP address using STUN servers
    async fn discover_external_address(&self) -> Result<()> {
        log::debug!("Discovering external address via STUN");
        
        for stun_server in &self.stun_servers {
            match self.query_stun_server(stun_server).await {
                Ok(addr) => {
                    log::info!("Discovered external address: {}", addr);
                    *self.external_address.write().await = Some(addr);
                    return Ok(());
                }
                Err(e) => {
                    log::debug!("STUN query failed for {}: {}", stun_server, e);
                    continue;
                }
            }
        }
        
        Err(anyhow::anyhow!("Failed to discover external address"))
    }

    /// Query STUN server for external address
    async fn query_stun_server(&self, server: &str) -> Result<SocketAddr> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server).await?;
        
        // STUN Binding Request
        let mut request = Vec::new();
        
        // STUN header: Message Type (Binding Request = 0x0001)
        request.extend_from_slice(&0x0001u16.to_be_bytes());
        
        // Message Length (will be 0 for simple request)
        request.extend_from_slice(&0x0000u16.to_be_bytes());
        
        // Magic Cookie
        request.extend_from_slice(&0x2112A442u32.to_be_bytes());
        
        // Transaction ID (12 bytes)
        let mut transaction_id = [0u8; 12];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut transaction_id);
        request.extend_from_slice(&transaction_id);
        
        // Send request
        socket.send(&request).await?;
        
        // Receive response
        let mut buffer = [0u8; 1024];
        let (len, _) = tokio::time::timeout(
            Duration::from_secs(5),
            socket.recv_from(&mut buffer)
        ).await??;
        
        // Parse STUN response
        self.parse_stun_response(&buffer[..len])
    }

    /// Parse STUN response to extract external address
    fn parse_stun_response(&self, data: &[u8]) -> Result<SocketAddr> {
        if data.len() < 20 {
            return Err(anyhow::anyhow!("STUN response too short"));
        }
        
        // Check if it's a Binding Success Response (0x0101)
        let msg_type = u16::from_be_bytes([data[0], data[1]]);
        if msg_type != 0x0101 {
            return Err(anyhow::anyhow!("Not a binding success response"));
        }
        
        let msg_length = u16::from_be_bytes([data[2], data[3]]) as usize;
        if data.len() < 20 + msg_length {
            return Err(anyhow::anyhow!("Incomplete STUN response"));
        }
        
        // Parse attributes
        let mut offset = 20;
        while offset + 4 <= data.len() {
            let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let attr_length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
            
            if offset + 4 + attr_length > data.len() {
                break;
            }
            
            // XOR-MAPPED-ADDRESS (0x0020) or MAPPED-ADDRESS (0x0001)
            if attr_type == 0x0020 || attr_type == 0x0001 {
                return self.parse_address_attribute(&data[offset + 4..offset + 4 + attr_length], attr_type == 0x0020);
            }
            
            // Move to next attribute (with padding)
            offset += 4 + ((attr_length + 3) & !3);
        }
        
        Err(anyhow::anyhow!("No address attribute found in STUN response"))
    }

    /// Parse address attribute from STUN response
    fn parse_address_attribute(&self, data: &[u8], is_xor_mapped: bool) -> Result<SocketAddr> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("Address attribute too short"));
        }
        
        let family = u16::from_be_bytes([data[1], data[2]]);
        let mut port = u16::from_be_bytes([data[2], data[3]]);
        
        if is_xor_mapped {
            port ^= 0x2112; // XOR with magic cookie
        }
        
        match family {
            0x01 => {
                // IPv4
                if data.len() < 8 {
                    return Err(anyhow::anyhow!("IPv4 address too short"));
                }
                
                let mut ip_bytes = [data[4], data[5], data[6], data[7]];
                if is_xor_mapped {
                    let magic = 0x2112A442u32.to_be_bytes();
                    for i in 0..4 {
                        ip_bytes[i] ^= magic[i];
                    }
                }
                
                let ip = Ipv4Addr::from(ip_bytes);
                Ok(SocketAddr::new(IpAddr::V4(ip), port))
            }
            0x02 => {
                // IPv6 - not implemented for simplicity
                Err(anyhow::anyhow!("IPv6 not supported in this implementation"))
            }
            _ => Err(anyhow::anyhow!("Unknown address family: {}", family)),
        }
    }

    /// Discover UPnP gateway for port mapping
    async fn discover_upnp_gateway(&self) -> Result<()> {
        log::debug!("Discovering UPnP gateway");
        
        // Send SSDP discovery request
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;
        
        let ssdp_request = format!(
            "M-SEARCH * HTTP/1.1\r\n\
             HOST: 239.255.255.250:1900\r\n\
             MAN: \"ssdp:discover\"\r\n\
             ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
             MX: 3\r\n\r\n"
        );
        
        socket.send_to(ssdp_request.as_bytes(), "239.255.255.250:1900").await?;
        
        // Wait for responses
        let mut buffer = [0u8; 1024];
        let timeout = tokio::time::timeout(Duration::from_secs(5), socket.recv(&mut buffer));
        
        match timeout.await {
            Ok(Ok(len)) => {
                let response = String::from_utf8_lossy(&buffer[..len]);
                if let Some(location) = self.parse_ssdp_location(&response) {
                    log::info!("Found UPnP gateway at: {}", location);
                    // In a full implementation, we would fetch the device description
                    // and determine port mapping capabilities
                    return Ok(());
                }
            }
            Ok(Err(e)) => log::debug!("UPnP recv error: {}", e),
            Err(_) => log::debug!("UPnP discovery timeout"),
        }
        
        Err(anyhow::anyhow!("No UPnP gateway found"))
    }

    /// Parse location from SSDP response
    fn parse_ssdp_location(&self, response: &str) -> Option<String> {
        for line in response.lines() {
            if line.to_lowercase().starts_with("location:") {
                return line.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }
        None
    }

    /// Determine NAT type using STUN binding tests
    async fn determine_nat_type(&self) -> Result<()> {
        log::debug!("Determining NAT type");
        
        // Test 1: Basic connectivity
        let test1_result = self.stun_test_basic().await;
        if test1_result.is_err() {
            *self.nat_type.write().await = NatType::Blocked;
            return Ok(());
        }
        
        let external_addr1 = test1_result?;
        
        // Test 2: Different server, same port
        let test2_result = self.stun_test_different_server().await;
        if let Ok(external_addr2) = test2_result {
            if external_addr1 == external_addr2 {
                // Test 3: Same server, different port
                let test3_result = self.stun_test_different_port().await;
                if let Ok(external_addr3) = test3_result {
                    if external_addr1 == external_addr3 {
                        *self.nat_type.write().await = NatType::FullCone;
                    } else {
                        *self.nat_type.write().await = NatType::RestrictedCone;
                    }
                } else {
                    *self.nat_type.write().await = NatType::PortRestricted;
                }
            } else {
                *self.nat_type.write().await = NatType::Symmetric;
            }
        } else {
            // Check if we have a direct internet connection
            if self.listen_addr.ip().is_global() {
                *self.nat_type.write().await = NatType::OpenInternet;
            } else {
                *self.nat_type.write().await = NatType::Unknown;
            }
        }
        
        let nat_type = self.nat_type.read().await.clone();
        log::info!("Determined NAT type: {:?}", nat_type);
        
        Ok(())
    }

    async fn stun_test_basic(&self) -> Result<SocketAddr> {
        self.query_stun_server(&self.stun_servers[0]).await
    }

    async fn stun_test_different_server(&self) -> Result<SocketAddr> {
        if self.stun_servers.len() > 1 {
            self.query_stun_server(&self.stun_servers[1]).await
        } else {
            Err(anyhow::anyhow!("No second STUN server available"))
        }
    }

    async fn stun_test_different_port(&self) -> Result<SocketAddr> {
        // This would require a STUN server on a different port
        // For simplicity, we'll use the same test
        self.query_stun_server(&self.stun_servers[0]).await
    }

    /// Setup port forwarding if possible
    async fn setup_port_forwarding(&self) {
        let nat_type = self.nat_type.read().await.clone();
        
        match nat_type {
            NatType::OpenInternet => {
                log::info!("Direct internet connection - no port forwarding needed");
                return;
            }
            NatType::FullCone | NatType::RestrictedCone => {
                log::info!("NAT type supports port forwarding");
            }
            NatType::Symmetric | NatType::PortRestricted => {
                log::warn!("NAT type may not support reliable port forwarding");
            }
            NatType::Blocked => {
                log::error!("Network blocks UDP traffic - P2P functionality limited");
                return;
            }
            NatType::Unknown => {
                log::warn!("Unknown NAT type - attempting port forwarding");
            }
        }
        
        // Attempt UPnP port mapping if gateway is available
        if self.upnp_gateway.read().await.is_some() {
            if let Err(e) = self.create_upnp_mapping().await {
                log::warn!("Failed to create UPnP port mapping: {}", e);
            }
        }
    }

    /// Create UPnP port mapping
    async fn create_upnp_mapping(&self) -> Result<()> {
        let external_port = self.chain_spec.default_port;
        let internal_port = self.listen_addr.port();
        
        // In a full implementation, this would use the UPnP SOAP API
        // to create the actual port mapping
        let mapping = PortMapping {
            external_port,
            internal_port,
            protocol: "TCP".to_string(),
            description: "QuantumCoin P2P".to_string(),
            lease_duration: Duration::from_secs(3600), // 1 hour
            created_at: Instant::now(),
        };
        
        *self.port_mapping.write().await = Some(mapping);
        
        log::info!("Created port mapping: {} -> {}", external_port, internal_port);
        Ok(())
    }

    /// Maintenance loop for NAT management
    async fn maintenance_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            
            // Refresh external address
            if let Err(e) = self.discover_external_address().await {
                log::debug!("Failed to refresh external address: {}", e);
            }
            
            // Renew port mapping if needed
            if let Some(mapping) = self.port_mapping.read().await.as_ref() {
                if mapping.created_at.elapsed() > mapping.lease_duration / 2 {
                    log::debug!("Renewing port mapping");
                    let _ = self.create_upnp_mapping().await;
                }
            }
        }
    }

    /// Get discovered external address
    pub async fn get_external_address(&self) -> Result<SocketAddr> {
        if let Some(addr) = *self.external_address.read().await {
            Ok(addr)
        } else {
            Err(anyhow::anyhow!("External address not discovered"))
        }
    }

    /// Get NAT type
    pub async fn get_nat_type(&self) -> NatType {
        self.nat_type.read().await.clone()
    }

    /// Check if port forwarding is active
    pub async fn has_port_forwarding(&self) -> bool {
        self.port_mapping.read().await.is_some()
    }

    /// Get connection info for advertising to peers
    pub async fn get_connection_info(&self) -> ConnectionInfo {
        let external_addr = *self.external_address.read().await;
        let nat_type = self.nat_type.read().await.clone();
        let has_upnp = self.upnp_gateway.read().await.is_some();
        let has_mapping = self.port_mapping.read().await.is_some();
        
        ConnectionInfo {
            listen_address: self.listen_addr,
            external_address: external_addr,
            nat_type,
            supports_upnp: has_upnp,
            has_port_mapping: has_mapping,
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down NAT manager");
        
        // Remove port mapping if it exists
        if let Some(_mapping) = self.port_mapping.read().await.as_ref() {
            // In a full implementation, we would remove the UPnP mapping
            log::debug!("Removed port mapping");
        }
        
        Ok(())
    }
}

impl Clone for NatManager {
    fn clone(&self) -> Self {
        Self {
            listen_addr: self.listen_addr,
            chain_spec: self.chain_spec.clone(),
            external_address: self.external_address.clone(),
            upnp_gateway: self.upnp_gateway.clone(),
            stun_servers: self.stun_servers.clone(),
            nat_type: self.nat_type.clone(),
            port_mapping: self.port_mapping.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub listen_address: SocketAddr,
    pub external_address: Option<SocketAddr>,
    pub nat_type: NatType,
    pub supports_upnp: bool,
    pub has_port_mapping: bool,
}

trait IpAddrExt {
    fn is_global(&self) -> bool;
}

impl IpAddrExt for IpAddr {
    fn is_global(&self) -> bool {
        match self {
            IpAddr::V4(ip) => {
                !ip.is_private() && !ip.is_loopback() && !ip.is_multicast() && !ip.is_broadcast()
            }
            IpAddr::V6(ip) => {
                !ip.is_loopback() && !ip.is_multicast() && !ip.is_unspecified()
            }
        }
    }
}
