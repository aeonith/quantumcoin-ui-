//! Chain specification loader and validator
//! 
//! This module handles loading and parsing of the chain_spec.toml file
//! with comprehensive validation and type safety.

use crate::consensus_engine::{ChainSpec, ConsensusError};
use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn, error};

/// Raw chain specification as loaded from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawChainSpec {
    pub network: RawNetworkSpec,
    pub consensus: RawConsensusSpec,
    pub supply: RawSupplySpec,
    pub transaction: RawTransactionSpec,
    pub block: RawBlockSpec,
    pub cryptography: RawCryptographySpec,
    pub network_protocol: RawNetworkProtocolSpec,
    pub fees: RawFeeSpec,
    pub mempool: RawMempoolSpec,
    pub mining: RawMiningSpec,
    pub governance: RawGovernanceSpec,
    pub checkpoints: RawCheckpointSpec,
    pub economic_model: RawEconomicModelSpec,
    pub post_quantum: RawPostQuantumSpec,
    pub ai_integration: RawAiIntegrationSpec,
    pub metadata: RawMetadataSpec,
    pub verification: RawVerificationSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawNetworkSpec {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawConsensusSpec {
    pub algorithm: String,
    pub hash_function: String,
    pub target_block_time: u64,
    pub difficulty_adjustment_period: u64,
    pub max_difficulty_change: f64,
    pub genesis_difficulty: String, // Hex string like "0x1d00ffff"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSupplySpec {
    pub max_supply: u64,
    pub initial_reward: u64,
    pub halving_interval: u64,
    pub premine: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransactionSpec {
    pub max_tx_size: usize,
    pub min_tx_fee: u64,
    pub dust_threshold: u64,
    pub max_inputs_per_tx: usize,
    pub max_outputs_per_tx: usize,
    pub signature_hash_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawBlockSpec {
    pub max_block_size: usize,
    pub max_block_weight: usize,
    pub coinbase_maturity: u64,
    pub max_reorg_depth: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCryptographySpec {
    pub address_version: u8,
    pub private_key_version: u8,
    pub checksum_algorithm: String,
    pub signature_scheme: String,
    pub hash_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawNetworkProtocolSpec {
    pub magic_bytes: Vec<u8>,
    pub protocol_version: u32,
    pub services: u64,
    pub default_port: u16,
    pub max_connections: usize,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFeeSpec {
    pub min_relay_fee: u64,
    pub increment_fee: u64,
    pub dust_relay_fee: u64,
    pub max_fee_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMempoolSpec {
    pub max_size: usize,
    pub max_tx_count: usize,
    pub expiry_time: u64,
    pub replacement_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMiningSpec {
    pub coinbase_flags: String,
    pub extra_nonce_placeholder: usize,
    pub witness_commitment_pos: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawGovernanceSpec {
    pub bip9_activation_threshold: u64,
    pub bip9_min_activation_height: u64,
    pub lock_in_period: u64,
    pub timeout_period: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCheckpointSpec {
    pub genesis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEconomicModelSpec {
    pub inflation_schedule: Vec<RawInflationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawInflationEntry {
    pub height: u64,
    pub reward: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPostQuantumSpec {
    pub signature_algorithm: String,
    pub public_key_size: usize,
    pub private_key_size: usize,
    pub signature_size: usize,
    pub security_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAiIntegrationSpec {
    pub enabled: bool,
    pub telemetry_endpoint: String,
    pub anomaly_detection: bool,
    pub fee_prediction: bool,
    pub network_analysis: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMetadataSpec {
    pub specification_version: String,
    pub created_at: String,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: String,
    pub documentation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawVerificationSpec {
    pub spec_hash: String,
    pub genesis_hash: String,
    pub checksum: String,
}

/// Chain specification loader with comprehensive validation
pub struct ChainSpecLoader;

impl ChainSpecLoader {
    /// Load and validate chain specification from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ChainSpec> {
        let path = path.as_ref();
        info!("Loading chain specification from: {}", path.display());
        
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read chain spec from {}", path.display()))?;
        
        let raw_spec: RawChainSpec = toml::from_str(&content)
            .with_context(|| "Failed to parse chain specification TOML")?;
        
        info!("Loaded chain spec for {} v{}", raw_spec.network.name, raw_spec.network.version);
        
        // Validate and convert to typed spec
        let spec = Self::validate_and_convert(raw_spec)?;
        
        info!("Chain specification loaded and validated successfully");
        Ok(spec)
    }
    
    /// Validate raw specification and convert to typed version
    fn validate_and_convert(raw: RawChainSpec) -> Result<ChainSpec> {
        // Validate network parameters
        Self::validate_network_spec(&raw.network)?;
        
        // Validate and convert consensus parameters
        let consensus = Self::validate_consensus_spec(&raw.consensus)?;
        
        // Validate supply parameters
        Self::validate_supply_spec(&raw.supply)?;
        
        // Validate transaction parameters
        Self::validate_transaction_spec(&raw.transaction)?;
        
        // Validate block parameters
        Self::validate_block_spec(&raw.block)?;
        
        // Validate cryptography parameters
        Self::validate_cryptography_spec(&raw.cryptography)?;
        
        // Validate fee parameters
        Self::validate_fee_spec(&raw.fees)?;
        
        // Validate governance parameters
        Self::validate_governance_spec(&raw.governance)?;
        
        // Validate post-quantum parameters
        Self::validate_post_quantum_spec(&raw.post_quantum)?;
        
        // Convert to typed specification
        Ok(ChainSpec {
            network: crate::consensus_engine::NetworkSpec {
                name: raw.network.name,
                symbol: raw.network.symbol,
                decimals: raw.network.decimals,
                version: raw.network.version,
            },
            consensus,
            supply: crate::consensus_engine::SupplySpec {
                max_supply: raw.supply.max_supply,
                initial_reward: raw.supply.initial_reward,
                halving_interval: raw.supply.halving_interval,
                premine: raw.supply.premine,
                inflation_schedule: raw.economic_model.inflation_schedule
                    .into_iter()
                    .map(|entry| crate::consensus_engine::InflationEntry {
                        height: entry.height,
                        reward: entry.reward,
                    })
                    .collect(),
            },
            transaction: crate::consensus_engine::TransactionSpec {
                max_tx_size: raw.transaction.max_tx_size,
                min_tx_fee: raw.transaction.min_tx_fee,
                dust_threshold: raw.transaction.dust_threshold,
                max_inputs_per_tx: raw.transaction.max_inputs_per_tx,
                max_outputs_per_tx: raw.transaction.max_outputs_per_tx,
                signature_hash_type: raw.transaction.signature_hash_type,
            },
            block: crate::consensus_engine::BlockSpec {
                max_block_size: raw.block.max_block_size,
                max_block_weight: raw.block.max_block_weight,
                coinbase_maturity: raw.block.coinbase_maturity,
                max_reorg_depth: raw.block.max_reorg_depth,
            },
            cryptography: crate::consensus_engine::CryptographySpec {
                address_version: raw.cryptography.address_version,
                private_key_version: raw.cryptography.private_key_version,
                checksum_algorithm: raw.cryptography.checksum_algorithm,
                signature_scheme: raw.cryptography.signature_scheme,
                hash_algorithm: raw.cryptography.hash_algorithm,
            },
            fees: crate::consensus_engine::FeeSpec {
                min_relay_fee: raw.fees.min_relay_fee,
                increment_fee: raw.fees.increment_fee,
                dust_relay_fee: raw.fees.dust_relay_fee,
                max_fee_rate: raw.fees.max_fee_rate,
            },
            mining: crate::consensus_engine::MiningSpec {
                coinbase_flags: raw.mining.coinbase_flags,
                extra_nonce_placeholder: raw.mining.extra_nonce_placeholder,
                witness_commitment_pos: raw.mining.witness_commitment_pos,
            },
            governance: crate::consensus_engine::GovernanceSpec {
                bip9_activation_threshold: raw.governance.bip9_activation_threshold,
                bip9_min_activation_height: raw.governance.bip9_min_activation_height,
                lock_in_period: raw.governance.lock_in_period,
                timeout_period: raw.governance.timeout_period,
            },
            post_quantum: crate::consensus_engine::PostQuantumSpec {
                signature_algorithm: raw.post_quantum.signature_algorithm,
                public_key_size: raw.post_quantum.public_key_size,
                private_key_size: raw.post_quantum.private_key_size,
                signature_size: raw.post_quantum.signature_size,
                security_level: raw.post_quantum.security_level,
            },
        })
    }
    
    fn validate_network_spec(spec: &RawNetworkSpec) -> Result<()> {
        if spec.name.is_empty() {
            return Err(anyhow!("Network name cannot be empty"));
        }
        
        if spec.symbol.is_empty() {
            return Err(anyhow!("Network symbol cannot be empty"));
        }
        
        if spec.decimals > 18 {
            return Err(anyhow!("Decimals cannot exceed 18"));
        }
        
        if spec.version.is_empty() {
            return Err(anyhow!("Version cannot be empty"));
        }
        
        info!("Network: {} ({}) v{} with {} decimals", 
              spec.name, spec.symbol, spec.version, spec.decimals);
        
        Ok(())
    }
    
    fn validate_consensus_spec(spec: &RawConsensusSpec) -> Result<crate::consensus_engine::ConsensusSpec> {
        // Validate algorithm
        if spec.algorithm != "proof_of_work" {
            return Err(anyhow!("Only proof_of_work consensus algorithm is supported"));
        }
        
        // Validate hash function
        if spec.hash_function != "blake3" {
            return Err(anyhow!("Only blake3 hash function is supported"));
        }
        
        // Validate block time
        if spec.target_block_time == 0 {
            return Err(anyhow!("Target block time cannot be zero"));
        }
        
        if spec.target_block_time < 60 || spec.target_block_time > 3600 {
            warn!("Target block time {} is outside recommended range (60-3600 seconds)", 
                  spec.target_block_time);
        }
        
        // Validate difficulty adjustment
        if spec.difficulty_adjustment_period == 0 {
            return Err(anyhow!("Difficulty adjustment period cannot be zero"));
        }
        
        if spec.max_difficulty_change <= 0.0 || spec.max_difficulty_change > 10.0 {
            return Err(anyhow!("Max difficulty change must be between 0 and 10"));
        }
        
        // Parse genesis difficulty
        let genesis_difficulty = if spec.genesis_difficulty.starts_with("0x") {
            u32::from_str_radix(&spec.genesis_difficulty[2..], 16)
                .with_context(|| format!("Invalid hex difficulty: {}", spec.genesis_difficulty))?
        } else {
            spec.genesis_difficulty.parse::<u32>()
                .with_context(|| format!("Invalid difficulty: {}", spec.genesis_difficulty))?
        };
        
        info!("Consensus: {} with {} hash, {}s blocks, difficulty adjustment every {} blocks",
              spec.algorithm, spec.hash_function, spec.target_block_time, spec.difficulty_adjustment_period);
        
        Ok(crate::consensus_engine::ConsensusSpec {
            algorithm: spec.algorithm.clone(),
            hash_function: spec.hash_function.clone(),
            target_block_time: spec.target_block_time,
            difficulty_adjustment_period: spec.difficulty_adjustment_period,
            max_difficulty_change: spec.max_difficulty_change,
            genesis_difficulty,
        })
    }
    
    fn validate_supply_spec(spec: &RawSupplySpec) -> Result<()> {
        if spec.max_supply == 0 {
            return Err(anyhow!("Max supply cannot be zero"));
        }
        
        if spec.initial_reward == 0 {
            return Err(anyhow!("Initial reward cannot be zero"));
        }
        
        if spec.halving_interval == 0 {
            return Err(anyhow!("Halving interval cannot be zero"));
        }
        
        // Validate that premine doesn't exceed max supply
        if spec.premine > spec.max_supply {
            return Err(anyhow!("Premine cannot exceed max supply"));
        }
        
        info!("Supply: max {} with {} initial reward, halving every {} blocks",
              spec.max_supply, spec.initial_reward, spec.halving_interval);
        
        Ok(())
    }
    
    fn validate_transaction_spec(spec: &RawTransactionSpec) -> Result<()> {
        if spec.max_tx_size == 0 {
            return Err(anyhow!("Max transaction size cannot be zero"));
        }
        
        if spec.min_tx_fee == 0 {
            warn!("Minimum transaction fee is zero - this may allow spam");
        }
        
        if spec.max_inputs_per_tx == 0 || spec.max_outputs_per_tx == 0 {
            return Err(anyhow!("Max inputs and outputs per transaction must be positive"));
        }
        
        // Validate signature scheme
        if spec.signature_hash_type != "dilithium2" {
            return Err(anyhow!("Only dilithium2 signature scheme is supported"));
        }
        
        info!("Transactions: max {}KB, min fee {}, up to {} inputs/{} outputs",
              spec.max_tx_size / 1024, spec.min_tx_fee, spec.max_inputs_per_tx, spec.max_outputs_per_tx);
        
        Ok(())
    }
    
    fn validate_block_spec(spec: &RawBlockSpec) -> Result<()> {
        if spec.max_block_size == 0 {
            return Err(anyhow!("Max block size cannot be zero"));
        }
        
        if spec.max_block_weight == 0 {
            return Err(anyhow!("Max block weight cannot be zero"));
        }
        
        if spec.coinbase_maturity == 0 {
            warn!("Coinbase maturity is zero - coinbase outputs can be spent immediately");
        }
        
        if spec.max_reorg_depth == 0 {
            return Err(anyhow!("Max reorg depth cannot be zero"));
        }
        
        info!("Blocks: max {}MB/{}MW, coinbase maturity {} blocks, max reorg depth {}",
              spec.max_block_size / (1024 * 1024), spec.max_block_weight / (1024 * 1024),
              spec.coinbase_maturity, spec.max_reorg_depth);
        
        Ok(())
    }
    
    fn validate_cryptography_spec(spec: &RawCryptographySpec) -> Result<()> {
        // Validate checksum algorithm
        if spec.checksum_algorithm != "blake3" {
            return Err(anyhow!("Only blake3 checksum algorithm is supported"));
        }
        
        // Validate signature scheme  
        if spec.signature_scheme != "dilithium2" {
            return Err(anyhow!("Only dilithium2 signature scheme is supported"));
        }
        
        // Validate hash algorithm
        if spec.hash_algorithm != "blake3_256" {
            return Err(anyhow!("Only blake3_256 hash algorithm is supported"));
        }
        
        info!("Cryptography: {} signatures with {} hashing",
              spec.signature_scheme, spec.hash_algorithm);
        
        Ok(())
    }
    
    fn validate_fee_spec(spec: &RawFeeSpec) -> Result<()> {
        if spec.min_relay_fee == 0 {
            warn!("Minimum relay fee is zero - may allow spam transactions");
        }
        
        if spec.max_fee_rate == 0 {
            return Err(anyhow!("Max fee rate cannot be zero"));
        }
        
        if spec.min_relay_fee > spec.max_fee_rate {
            return Err(anyhow!("Min relay fee cannot exceed max fee rate"));
        }
        
        info!("Fees: min relay {}, max rate {}, dust relay {}",
              spec.min_relay_fee, spec.max_fee_rate, spec.dust_relay_fee);
        
        Ok(())
    }
    
    fn validate_governance_spec(spec: &RawGovernanceSpec) -> Result<()> {
        if spec.bip9_activation_threshold == 0 {
            return Err(anyhow!("BIP9 activation threshold cannot be zero"));
        }
        
        if spec.lock_in_period == 0 {
            return Err(anyhow!("Lock-in period cannot be zero"));
        }
        
        if spec.timeout_period == 0 {
            return Err(anyhow!("Timeout period cannot be zero"));
        }
        
        info!("Governance: BIP9 threshold {}, lock-in {} blocks, timeout {} blocks",
              spec.bip9_activation_threshold, spec.lock_in_period, spec.timeout_period);
        
        Ok(())
    }
    
    fn validate_post_quantum_spec(spec: &RawPostQuantumSpec) -> Result<()> {
        if spec.signature_algorithm != "dilithium2" {
            return Err(anyhow!("Only dilithium2 post-quantum signatures are supported"));
        }
        
        if spec.security_level != 2 {
            return Err(anyhow!("Only NIST security level 2 is supported"));
        }
        
        // Validate key and signature sizes for dilithium2
        if spec.public_key_size != 1312 {
            return Err(anyhow!("Invalid public key size for dilithium2: expected 1312, got {}", spec.public_key_size));
        }
        
        if spec.private_key_size != 2528 {
            return Err(anyhow!("Invalid private key size for dilithium2: expected 2528, got {}", spec.private_key_size));
        }
        
        if spec.signature_size != 2420 {
            return Err(anyhow!("Invalid signature size for dilithium2: expected 2420, got {}", spec.signature_size));
        }
        
        info!("Post-quantum: {} signatures (NIST level {}), key sizes {}/{}B, signature {}B",
              spec.signature_algorithm, spec.security_level, 
              spec.public_key_size, spec.private_key_size, spec.signature_size);
        
        Ok(())
    }
    
    /// Create a default chain specification for testing
    pub fn create_test_spec() -> ChainSpec {
        ChainSpec {
            network: crate::consensus_engine::NetworkSpec {
                name: "quantumcoin-test".to_string(),
                symbol: "QTC-TEST".to_string(),
                decimals: 8,
                version: "2.0.0-test".to_string(),
            },
            consensus: crate::consensus_engine::ConsensusSpec {
                algorithm: "proof_of_work".to_string(),
                hash_function: "blake3".to_string(),
                target_block_time: 60, // 1 minute for testing
                difficulty_adjustment_period: 10, // Every 10 blocks for testing
                max_difficulty_change: 4.0,
                genesis_difficulty: 0x207fffff, // Lower difficulty for testing
            },
            supply: crate::consensus_engine::SupplySpec {
                max_supply: 1_000_000_00000000, // 1M coins for testing
                initial_reward: 50_00000000,
                halving_interval: 100, // Every 100 blocks for testing
                premine: 0,
                inflation_schedule: vec![
                    crate::consensus_engine::InflationEntry { height: 0, reward: 50_00000000 },
                    crate::consensus_engine::InflationEntry { height: 100, reward: 25_00000000 },
                    crate::consensus_engine::InflationEntry { height: 200, reward: 12_50000000 },
                ],
            },
            transaction: crate::consensus_engine::TransactionSpec {
                max_tx_size: 50000, // 50KB for testing
                min_tx_fee: 100,
                dust_threshold: 546,
                max_inputs_per_tx: 100,
                max_outputs_per_tx: 100,
                signature_hash_type: "dilithium2".to_string(),
            },
            block: crate::consensus_engine::BlockSpec {
                max_block_size: 1_000_000, // 1MB for testing
                max_block_weight: 1_000_000,
                coinbase_maturity: 10, // 10 blocks for testing
                max_reorg_depth: 3, // 3 blocks for testing
            },
            cryptography: crate::consensus_engine::CryptographySpec {
                address_version: 0x6f, // Testnet version
                private_key_version: 0xef,
                checksum_algorithm: "blake3".to_string(),
                signature_scheme: "dilithium2".to_string(),
                hash_algorithm: "blake3_256".to_string(),
            },
            fees: crate::consensus_engine::FeeSpec {
                min_relay_fee: 100,
                increment_fee: 100,
                dust_relay_fee: 300,
                max_fee_rate: 1_000_000,
            },
            mining: crate::consensus_engine::MiningSpec {
                coinbase_flags: "QuantumCoin-Test/2.0".to_string(),
                extra_nonce_placeholder: 8,
                witness_commitment_pos: 0,
            },
            governance: crate::consensus_engine::GovernanceSpec {
                bip9_activation_threshold: 8, // 80% of 10 blocks for testing
                bip9_min_activation_height: 0,
                lock_in_period: 10,
                timeout_period: 50,
            },
            post_quantum: crate::consensus_engine::PostQuantumSpec {
                signature_algorithm: "dilithium2".to_string(),
                public_key_size: 1312,
                private_key_size: 2528,
                signature_size: 2420,
                security_level: 2,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_chain_spec_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = r#"
[network]
name = "quantumcoin-test"
symbol = "QTC-TEST"
decimals = 8
version = "1.0.0-test"

[consensus]
algorithm = "proof_of_work"
hash_function = "blake3"
target_block_time = 600
difficulty_adjustment_period = 2016
max_difficulty_change = 4.0
genesis_difficulty = "0x1d00ffff"

[supply]
max_supply = 22000000000000000
initial_reward = 5000000000
halving_interval = 210000
premine = 0

[transaction]
max_tx_size = 100000
min_tx_fee = 1000
dust_threshold = 546
max_inputs_per_tx = 1000
max_outputs_per_tx = 1000
signature_hash_type = "dilithium2"

[block]
max_block_size = 4000000
max_block_weight = 4000000
coinbase_maturity = 100
max_reorg_depth = 6

[cryptography]
address_version = 81
private_key_version = 128
checksum_algorithm = "blake3"
signature_scheme = "dilithium2"
hash_algorithm = "blake3_256"

[network_protocol]
magic_bytes = [81, 84, 67, 77]
protocol_version = 70015
services = 1
default_port = 8333
max_connections = 125
connection_timeout = 5

[fees]
min_relay_fee = 1000
increment_fee = 1000
dust_relay_fee = 3000
max_fee_rate = 10000000

[mempool]
max_size = 300000000
max_tx_count = 100000
expiry_time = 86400
replacement_enabled = true

[mining]
coinbase_flags = "QuantumCoin/2.0"
extra_nonce_placeholder = 8
witness_commitment_pos = 0

[governance]
bip9_activation_threshold = 1916
bip9_min_activation_height = 0
lock_in_period = 2016
timeout_period = 10080

[checkpoints]
genesis = "0000000000000000000000000000000000000000000000000000000000000000"

[economic_model]
inflation_schedule = [
    { height = 0, reward = 5000000000 },
    { height = 210000, reward = 2500000000 },
    { height = 420000, reward = 1250000000 },
]

[post_quantum]
signature_algorithm = "dilithium2"
public_key_size = 1312
private_key_size = 2528
signature_size = 2420
security_level = 2

[ai_integration]
enabled = true
telemetry_endpoint = "https://ai.quantumcoincrypto.com/telemetry"
anomaly_detection = true
fee_prediction = true
network_analysis = true

[metadata]
specification_version = "2.0.0"
created_at = "2025-01-15T00:00:00Z"
authors = ["test"]
license = "MIT"
repository = "https://github.com/test/test"
documentation = "https://docs.test.com"

[verification]
spec_hash = "blake3:test"
genesis_hash = "blake3:test"
checksum = "blake3:test"
"#;
        
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        
        let spec = ChainSpecLoader::load(temp_file.path()).unwrap();
        assert_eq!(spec.network.name, "quantumcoin-test");
        assert_eq!(spec.consensus.target_block_time, 600);
        assert_eq!(spec.consensus.genesis_difficulty, 0x1d00ffff);
    }
    
    #[test]
    fn test_invalid_chain_spec() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_toml = r#"
[network]
name = ""
symbol = "QTC"
decimals = 8
version = "1.0.0"

[consensus]
algorithm = "invalid_algorithm"
hash_function = "blake3"
target_block_time = 0
difficulty_adjustment_period = 2016
max_difficulty_change = 4.0
genesis_difficulty = "0x1d00ffff"
"#;
        
        temp_file.write_all(invalid_toml.as_bytes()).unwrap();
        
        let result = ChainSpecLoader::load(temp_file.path());
        assert!(result.is_err(), "Should reject invalid chain spec");
    }
    
    #[test] 
    fn test_test_spec_creation() {
        let spec = ChainSpecLoader::create_test_spec();
        assert_eq!(spec.network.name, "quantumcoin-test");
        assert_eq!(spec.consensus.target_block_time, 60);
        assert!(spec.supply.max_supply > 0);
    }
}
