//! Genesis block builder with deterministic generation

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use crate::{
    config::{ChainSpec, GenesisAllocation},
    block::{GenesisBlock, BlockHeader, GenesisTransaction, TransactionType, GenesisMetadata, CreationParams},
    crypto::{GenesisCrypto, generate_genesis_seed, blake3_hash},
    merkle::MerkleTree,
};

/// Builder for creating reproducible genesis blocks
pub struct GenesisBuilder {
    chain_spec: ChainSpec,
    deterministic: bool,
    custom_seed: Option<[u8; 32]>,
}

impl GenesisBuilder {
    /// Create new genesis builder
    pub fn new(chain_spec: ChainSpec) -> Self {
        Self {
            chain_spec,
            deterministic: true,
            custom_seed: None,
        }
    }
    
    /// Enable/disable deterministic mode
    pub fn deterministic(mut self, deterministic: bool) -> Self {
        self.deterministic = deterministic;
        self
    }
    
    /// Set custom seed for deterministic generation
    pub fn with_seed(mut self, seed: [u8; 32]) -> Self {
        self.custom_seed = Some(seed);
        self
    }
    
    /// Build the genesis block
    pub fn build(self) -> Result<GenesisBlock> {
        // Validate chain specification first
        self.chain_spec.validate()
            .context("Invalid chain specification")?;
        
        // Generate deterministic seed if needed
        let seed = if let Some(custom_seed) = self.custom_seed {
            custom_seed
        } else if self.deterministic {
            generate_genesis_seed(
                &self.chain_spec.network.name,
                &self.chain_spec.network_protocol.magic_bytes,
                self.chain_spec.genesis.timestamp.timestamp() as u64,
            )
        } else {
            // Use current time for non-deterministic generation
            let mut seed = [0u8; 32];
            let current_time = Utc::now().timestamp() as u64;
            seed[..8].copy_from_slice(&current_time.to_le_bytes());
            seed
        };
        
        // Create cryptographic context
        let crypto = GenesisCrypto::new_deterministic(seed)
            .context("Failed to create cryptographic context")?;
        
        // Create genesis transactions
        let mut transactions = Vec::new();
        let mut tx_index = 0u32;
        
        // Add coinbase transaction (always first)
        let coinbase = self.create_coinbase_transaction(tx_index)?;
        transactions.push(coinbase);
        tx_index += 1;
        
        // Add allocation transactions
        for allocation in &self.chain_spec.genesis.allocations {
            let tx = self.create_allocation_transaction(allocation, tx_index)?;
            transactions.push(tx);
            tx_index += 1;
        }
        
        // Create merkle tree
        let tx_hashes: Vec<[u8; 32]> = transactions.iter()
            .map(|tx| tx.hash)
            .collect();
        
        let merkle_tree = MerkleTree::new(tx_hashes)
            .context("Failed to create merkle tree")?;
        
        // Create block header
        let header = self.create_block_header(merkle_tree.root())?;
        
        // Calculate chain spec hash
        let chain_spec_hash = self.calculate_chain_spec_hash()?;
        
        // Create metadata
        let metadata = self.create_metadata();
        
        // Create genesis block
        let mut block = GenesisBlock::new(
            header,
            transactions,
            chain_spec_hash,
            metadata,
        ).context("Failed to create genesis block")?;
        
        // Sign the block
        if self.deterministic {
            let header_bytes = bincode::serialize(&block.header)
                .context("Failed to serialize header for signing")?;
            
            let signature = crypto.sign(&header_bytes)
                .context("Failed to sign genesis block")?;
            
            block.sign(signature);
        }
        
        // Final validation
        block.validate()
            .context("Genesis block validation failed")?;
        
        Ok(block)
    }
    
    /// Create coinbase transaction
    fn create_coinbase_transaction(&self, index: u32) -> Result<GenesisTransaction> {
        // Coinbase gets initial block reward if no premine
        let amount = if self.chain_spec.supply.premine == 0 && self.chain_spec.genesis.allocations.is_empty() {
            self.chain_spec.supply.initial_reward
        } else {
            0 // No reward if there are allocations
        };
        
        let address = format!("qtc1q{:0>58}", "genesis"); // Genesis address
        
        Ok(GenesisTransaction::new_coinbase(
            address,
            amount,
            self.chain_spec.genesis.coinbase.message.clone(),
            index,
            self.chain_spec.genesis.timestamp,
            &self.create_extra_nonce(),
        ))
    }
    
    /// Create allocation transaction
    fn create_allocation_transaction(
        &self,
        allocation: &GenesisAllocation,
        index: u32,
    ) -> Result<GenesisTransaction> {
        Ok(GenesisTransaction::new_allocation(
            allocation.address.clone(),
            allocation.amount,
            allocation.purpose.clone(),
            index,
        ))
    }
    
    /// Create block header
    fn create_block_header(&self, merkle_root: [u8; 32]) -> Result<BlockHeader> {
        Ok(BlockHeader {
            version: 1,
            previous_hash: [0; 32], // Genesis has no previous block
            merkle_root,
            timestamp: self.chain_spec.genesis.timestamp,
            difficulty: self.chain_spec.consensus.genesis_difficulty,
            nonce: 0, // Genesis blocks don't need mining
            extra_nonce: self.create_extra_nonce(),
        })
    }
    
    /// Create extra nonce for coinbase
    fn create_extra_nonce(&self) -> Vec<u8> {
        if self.deterministic {
            // Deterministic extra nonce
            let mut data = Vec::new();
            data.extend_from_slice(self.chain_spec.network.name.as_bytes());
            data.extend_from_slice(&self.chain_spec.genesis.timestamp.timestamp().to_le_bytes());
            let hash = blake3_hash(&data);
            hash[..self.chain_spec.genesis.coinbase.extra_nonce_size].to_vec()
        } else {
            vec![0u8; self.chain_spec.genesis.coinbase.extra_nonce_size]
        }
    }
    
    /// Calculate hash of chain specification
    fn calculate_chain_spec_hash(&self) -> Result<[u8; 32]> {
        let serialized = bincode::serialize(&self.chain_spec)
            .context("Failed to serialize chain specification")?;
        Ok(blake3_hash(&serialized))
    }
    
    /// Create genesis metadata
    fn create_metadata(&self) -> GenesisMetadata {
        GenesisMetadata {
            creator: "QuantumCoin Genesis Builder v2.0".to_string(),
            created_at: if self.deterministic {
                self.chain_spec.genesis.timestamp
            } else {
                Utc::now()
            },
            chain_spec_version: self.chain_spec.metadata.specification_version.clone(),
            creation_params: CreationParams {
                network_magic: self.chain_spec.network_protocol.magic_bytes,
                genesis_message: self.chain_spec.genesis.message.clone(),
                max_supply: self.chain_spec.supply.max_supply,
                initial_reward: self.chain_spec.supply.initial_reward,
                deterministic: self.deterministic,
            },
        }
    }
}

/// Helper for mining genesis blocks if needed
pub struct GenesisMiner;

impl GenesisMiner {
    /// Mine genesis block to meet difficulty target
    pub fn mine_genesis(block: &mut GenesisBlock) -> Result<()> {
        let target = Self::difficulty_to_target(block.header.difficulty);
        let mut nonce = 0u64;
        let mut extra_nonce_counter = 0u64;
        
        loop {
            block.header.nonce = nonce;
            let hash = GenesisBlock::calculate_block_hash(&block.header)?;
            
            if Self::hash_meets_target(&hash, &target) {
                block.hash = hash;
                break;
            }
            
            nonce = nonce.wrapping_add(1);
            
            // Increment extra nonce every 4 billion attempts
            if nonce == 0 {
                extra_nonce_counter += 1;
                let extra_nonce_bytes = extra_nonce_counter.to_le_bytes();
                let extra_nonce_len = block.header.extra_nonce.len();
                
                for (i, &byte) in extra_nonce_bytes.iter().enumerate() {
                    if i < extra_nonce_len {
                        block.header.extra_nonce[i] = byte;
                    }
                }
                
                // Recalculate merkle root with new extra nonce
                if let Some(coinbase) = block.transactions.get_mut(0) {
                    coinbase.hash = crate::merkle::create_coinbase_transaction_hash(
                        &coinbase.message,
                        block.header.timestamp.timestamp() as u64,
                        &block.header.extra_nonce,
                    );
                    
                    let tx_hashes: Vec<[u8; 32]> = block.transactions.iter()
                        .map(|tx| tx.hash)
                        .collect();
                    
                    block.merkle_tree = MerkleTree::new(tx_hashes)?;
                    block.header.merkle_root = block.merkle_tree.root();
                }
            }
            
            if nonce % 100_000 == 0 {
                tracing::info!(
                    "Mining genesis block: {} attempts, extra_nonce: {}",
                    nonce,
                    extra_nonce_counter
                );
            }
        }
        
        tracing::info!(
            "Genesis block mined! Hash: {}, Nonce: {}, Extra nonce: {}",
            hex::encode(block.hash),
            block.header.nonce,
            extra_nonce_counter
        );
        
        Ok(())
    }
    
    /// Convert difficulty to target
    fn difficulty_to_target(difficulty: u32) -> [u8; 32] {
        let mut target = [0u8; 32];
        
        // Extract exponent and mantissa from compact format
        let exponent = (difficulty >> 24) as usize;
        let mantissa = difficulty & 0x00ffffff;
        
        if exponent <= 3 {
            // Special case for very small targets
            let mantissa_bytes = mantissa.to_be_bytes();
            for i in 0..(4 - exponent).min(32) {
                target[31 - i] = mantissa_bytes[3 - i];
            }
        } else if exponent < 32 {
            // Normal case
            let mantissa_bytes = mantissa.to_be_bytes();
            let start_pos = 32 - exponent;
            for i in 1..4 {
                if start_pos + i < 32 {
                    target[start_pos + i] = mantissa_bytes[i];
                }
            }
        }
        // If exponent >= 32, target remains all zeros (impossible target)
        
        target
    }
    
    /// Check if hash meets difficulty target
    fn hash_meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
        // Compare hash with target (both in big-endian format)
        for i in 0..32 {
            if hash[i] < target[i] {
                return true;
            } else if hash[i] > target[i] {
                return false;
            }
        }
        true // Equal is also acceptable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ChainSpec;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    fn create_test_chain_spec() -> ChainSpec {
        ChainSpec::load_testnet().unwrap()
    }
    
    #[test]
    fn test_deterministic_genesis_generation() {
        let chain_spec = create_test_chain_spec();
        
        let seed = [0x42; 32];
        let builder1 = GenesisBuilder::new(chain_spec.clone()).with_seed(seed);
        let builder2 = GenesisBuilder::new(chain_spec).with_seed(seed);
        
        let genesis1 = builder1.build().unwrap();
        let genesis2 = builder2.build().unwrap();
        
        assert_eq!(genesis1.hash, genesis2.hash);
        assert_eq!(genesis1.header.merkle_root, genesis2.header.merkle_root);
        assert_eq!(genesis1.signature, genesis2.signature);
    }
    
    #[test]
    fn test_mainnet_genesis_creation() {
        let chain_spec = ChainSpec::load_mainnet().unwrap();
        let builder = GenesisBuilder::new(chain_spec);
        
        let genesis = builder.build().unwrap();
        
        assert_eq!(genesis.header.previous_hash, [0; 32]);
        assert_eq!(genesis.total_allocation(), genesis.chain_spec.supply.initial_reward);
        assert!(genesis.signature.is_some());
        assert!(genesis.validate().is_ok());
    }
    
    #[test]
    fn test_testnet_genesis_with_allocations() {
        let chain_spec = ChainSpec::load_testnet().unwrap();
        let builder = GenesisBuilder::new(chain_spec);
        
        let genesis = builder.build().unwrap();
        
        assert!(!genesis.allocation_transactions().is_empty());
        assert!(genesis.total_allocation() > 0);
        assert!(genesis.validate().is_ok());
    }
    
    #[test]
    fn test_difficulty_target_conversion() {
        // Test various difficulty values
        let target1 = GenesisMiner::difficulty_to_target(0x1d00ffff);
        let target2 = GenesisMiner::difficulty_to_target(0x207fffff);
        
        assert_ne!(target1, target2);
        assert_ne!(target1, [0; 32]);
        assert_ne!(target2, [0; 32]);
    }
    
    #[test]
    fn test_hash_target_comparison() {
        let easy_target = GenesisMiner::difficulty_to_target(0x207fffff);
        let hard_target = GenesisMiner::difficulty_to_target(0x1d00ffff);
        
        // Easy target should be larger than hard target
        let easy_larger = {
            for i in 0..32 {
                if easy_target[i] > hard_target[i] {
                    break true;
                } else if easy_target[i] < hard_target[i] {
                    break false;
                }
            }
            false // Equal
        };
        
        assert!(easy_larger);
    }
}
