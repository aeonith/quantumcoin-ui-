//! Chain specification configuration for genesis block creation

use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

/// Complete chain specification loaded from chain_spec.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSpec {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub supply: SupplyConfig,
    pub transaction: TransactionConfig,
    pub block: BlockConfig,
    pub cryptography: CryptographyConfig,
    pub network_protocol: NetworkProtocolConfig,
    pub genesis: GenesisConfig,
    pub post_quantum: PostQuantumConfig,
    pub metadata: MetadataConfig,
    pub verification: VerificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub algorithm: String,
    pub hash_function: String,
    pub target_block_time: u64,
    pub difficulty_adjustment_period: u64,
    pub max_difficulty_change: f64,
    pub genesis_difficulty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyConfig {
    pub max_supply: u64,
    pub initial_reward: u64,
    pub halving_interval: u64,
    pub premine: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    pub max_tx_size: u64,
    pub min_tx_fee: u64,
    pub dust_threshold: u64,
    pub max_inputs_per_tx: u32,
    pub max_outputs_per_tx: u32,
    pub signature_hash_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockConfig {
    pub max_block_size: u64,
    pub max_block_weight: u64,
    pub coinbase_maturity: u64,
    pub max_reorg_depth: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptographyConfig {
    pub address_version: u8,
    pub private_key_version: u8,
    pub checksum_algorithm: String,
    pub signature_scheme: String,
    pub hash_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProtocolConfig {
    pub magic_bytes: [u8; 4],
    pub protocol_version: u32,
    pub services: u64,
    pub default_port: u16,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostQuantumConfig {
    pub signature_algorithm: String,
    pub public_key_size: usize,
    pub private_key_size: usize,
    pub signature_size: usize,
    pub security_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub specification_version: String,
    pub created_at: DateTime<Utc>,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: String,
    pub documentation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub spec_hash: String,
    pub genesis_hash: String,
    pub checksum: String,
}

/// Genesis-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Genesis block timestamp (must be deterministic)
    pub timestamp: DateTime<Utc>,
    /// Genesis message embedded in coinbase
    pub message: String,
    /// Initial allocations for development/foundation
    pub allocations: Vec<GenesisAllocation>,
    /// Genesis coinbase transaction parameters
    pub coinbase: GenesisCoinbaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAllocation {
    /// Address to receive allocation
    pub address: String,
    /// Amount in satoshis (8 decimal places)
    pub amount: u64,
    /// Description of allocation purpose
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisCoinbaseConfig {
    /// Coinbase transaction message
    pub message: String,
    /// Extra nonce space (bytes)
    pub extra_nonce_size: usize,
    /// Coinbase flags
    pub flags: String,
}

impl ChainSpec {
    /// Load chain specification from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read chain specification file")?;
        
        let mut spec: ChainSpec = toml::from_str(&content)
            .context("Failed to parse chain specification")?;
        
        // Use default genesis config if not present in TOML
        // This handles the case where genesis section is missing or incomplete
        
        Ok(spec)
    }
    
    /// Load mainnet configuration
    pub fn load_mainnet() -> Result<Self> {
        let mut spec = Self::load_from_file("chain_spec.toml")?;
        spec.genesis = Self::mainnet_genesis_config();
        Ok(spec)
    }
    
    /// Load testnet configuration
    pub fn load_testnet() -> Result<Self> {
        let mut spec = Self::load_from_file("chain_spec.toml")?;
        spec.genesis = Self::testnet_genesis_config();
        // Adjust for testnet
        spec.consensus.genesis_difficulty = 0x207fffff; // Lower difficulty
        spec.network_protocol.default_port = 18333;
        spec.network_protocol.magic_bytes = [0x51, 0x54, 0x43, 0x54]; // "QTCT"
        Ok(spec)
    }
    
    /// Default genesis configuration (no premine)
    fn default_genesis_config() -> GenesisConfig {
        GenesisConfig {
            timestamp: DateTime::parse_from_rfc3339("2025-01-15T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            message: "QuantumCoin Genesis - Quantum-Safe Future of Digital Money".to_string(),
            allocations: vec![],
            coinbase: GenesisCoinbaseConfig {
                message: "QuantumCoin/2.0 - The Chancellor on brink of second bailout for banks".to_string(),
                extra_nonce_size: 8,
                flags: "QuantumCoin/2.0".to_string(),
            },
        }
    }
    
    /// Mainnet genesis configuration
    fn mainnet_genesis_config() -> GenesisConfig {
        GenesisConfig {
            // Fixed mainnet timestamp for reproducibility
            timestamp: DateTime::parse_from_rfc3339("2025-01-15T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            message: "QuantumCoin Mainnet Genesis - Post-Quantum Cryptographic Future".to_string(),
            allocations: vec![
                // No premine for fairness - all coins from mining
            ],
            coinbase: GenesisCoinbaseConfig {
                message: "The Times 15/Jan/2025 Chancellor on brink of post-quantum cryptography era".to_string(),
                extra_nonce_size: 8,
                flags: "QuantumCoin/2.0".to_string(),
            },
        }
    }
    
    /// Testnet genesis configuration
    fn testnet_genesis_config() -> GenesisConfig {
        GenesisConfig {
            timestamp: DateTime::parse_from_rfc3339("2025-01-15T00:00:01Z")
                .unwrap()
                .with_timezone(&Utc),
            message: "QuantumCoin Testnet Genesis - Testing Post-Quantum Future".to_string(),
            allocations: vec![
                GenesisAllocation {
                    address: "qtc1qtestnet00000000000000000000000000000000".to_string(),
                    amount: 1000_00000000, // 1000 QTC for testing
                    purpose: "Testnet faucet allocation".to_string(),
                },
            ],
            coinbase: GenesisCoinbaseConfig {
                message: "QuantumCoin Testnet - Post-Quantum Testing Environment".to_string(),
                extra_nonce_size: 8,
                flags: "QuantumCoin/2.0-testnet".to_string(),
            },
        }
    }
    
    /// Calculate the total genesis allocation amount
    pub fn total_genesis_allocation(&self) -> u64 {
        self.genesis.allocations.iter().map(|a| a.amount).sum()
    }
    
    /// Validate the chain specification
    pub fn validate(&self) -> Result<()> {
        // Validate supply constraints
        if self.total_genesis_allocation() > self.supply.max_supply {
            anyhow::bail!("Genesis allocations exceed maximum supply");
        }
        
        // Validate network parameters
        if self.network.decimals > 18 {
            anyhow::bail!("Too many decimal places");
        }
        
        // Validate consensus parameters
        if self.consensus.target_block_time == 0 {
            anyhow::bail!("Block time cannot be zero");
        }
        
        // Validate cryptography parameters
        if self.post_quantum.signature_algorithm != "dilithium2" {
            anyhow::bail!("Only Dilithium2 is currently supported");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_chain_spec_validation() {
        let spec = ChainSpec::load_mainnet().unwrap();
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_genesis_allocations() {
        let spec = ChainSpec::load_testnet().unwrap();
        assert!(!spec.genesis.allocations.is_empty());
        assert!(spec.total_genesis_allocation() > 0);
    }
    
    #[test]
    fn test_mainnet_no_premine() {
        let spec = ChainSpec::load_mainnet().unwrap();
        assert_eq!(spec.supply.premine, 0);
        assert_eq!(spec.total_genesis_allocation(), 0);
    }
}
