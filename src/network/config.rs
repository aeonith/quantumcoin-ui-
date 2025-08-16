// Configuration loading from chain_spec.toml
use crate::network::ChainSpec;
use anyhow::Result;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ChainSpecFile {
    network_protocol: NetworkProtocol,
}

#[derive(Debug, Deserialize)]
struct NetworkProtocol {
    magic_bytes: [u8; 4],
    protocol_version: u32,
    default_port: u16,
    max_connections: usize,
    connection_timeout: u64,
}

impl ChainSpec {
    /// Load chain specification from TOML file
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let spec: ChainSpecFile = toml::from_str(&content)?;
        
        Ok(Self {
            network_name: "quantumcoin".to_string(),
            magic_bytes: spec.network_protocol.magic_bytes,
            protocol_version: spec.network_protocol.protocol_version,
            default_port: spec.network_protocol.default_port,
            max_connections: spec.network_protocol.max_connections,
            connection_timeout: spec.network_protocol.connection_timeout,
            dns_seeds: vec![
                "seed1.quantumcoin.network".to_string(),
                "seed2.quantumcoin.network".to_string(),
                "seed3.quantumcoin.network".to_string(),
                "seed4.quantumcoin.network".to_string(),
            ],
            bootstrap_nodes: vec![
                // Add hardcoded bootstrap nodes for initial fallback
                "67.205.139.101:8333".parse().unwrap_or_else(|_| "127.0.0.1:8333".parse().unwrap()),
                "134.209.116.207:8333".parse().unwrap_or_else(|_| "127.0.0.1:8334".parse().unwrap()),
            ],
        })
    }
    
    /// Load with fallback to default if file doesn't exist
    pub async fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path).await {
            Ok(spec) => {
                log::info!("Loaded chain specification from file");
                spec
            }
            Err(e) => {
                log::warn!("Failed to load chain spec, using defaults: {}", e);
                Self::default()
            }
        }
    }
}
