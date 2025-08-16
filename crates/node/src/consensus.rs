//! Consensus rules and validation for QuantumCoin
//! 
//! This module provides the primary interface to the production-grade consensus engine
//! and maintains backward compatibility with existing code while adding comprehensive
//! validation capabilities.

use crate::{
    block::{Block, BlockError},
    transaction::{Transaction, TransactionError},
    economics::Economics,
    config::SharedConfig,
};
use crate::consensus_engine::{ConsensusEngine as ProductionConsensusEngine, ChainSpec};
use crate::chain_spec_loader::ChainSpecLoader;
use anyhow::{Result, Context};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{info, error, instrument};

// Re-export consensus errors for backward compatibility
pub use crate::consensus_engine::ConsensusError;

/// Production consensus validation system
pub struct ConsensusSystem {
    /// Production-grade consensus engine
    engine: Arc<ProductionConsensusEngine>,
    
    /// Chain specification
    spec: ChainSpec,
    
    /// Configuration
    config: SharedConfig,
    
    /// Economics engine
    economics: Economics,
}

impl ConsensusSystem {
    /// Create new consensus system with chain specification
    #[instrument(skip(config))]
    pub fn new(config: SharedConfig, chain_spec_path: Option<&str>) -> Result<Self> {
        info!("Initializing QuantumCoin consensus system");
        
        // Load chain specification
        let spec = if let Some(path) = chain_spec_path {
            ChainSpecLoader::load(path)
                .with_context(|| format!("Failed to load chain spec from {}", path))?
        } else {
            ChainSpecLoader::create_test_spec()
        };
        
        info!(
            "Loaded chain specification for {} v{} ({})",
            spec.network.name,
            spec.network.version,
            spec.consensus.algorithm
        );
        
        // Create economics engine
        let economics = Economics::from_shared_config(&config);
        
        // Create production consensus engine
        let engine = Arc::new(
            ProductionConsensusEngine::new(spec.clone(), config.clone())
                .context("Failed to create consensus engine")?
        );
        
        info!("Consensus system initialized successfully");
        
        Ok(ConsensusSystem {
            engine,
            spec,
            config,
            economics,
        })
    }
    
    /// Validate a block with comprehensive checks
    #[instrument(skip(self, block, prev_block))]
    pub fn validate_block(&self, block: &Block, prev_block: Option<&Block>) -> Result<(), ConsensusError> {
        self.engine.validate_block(block, prev_block)
    }
    
    /// Validate a transaction
    #[instrument(skip(self, tx))]
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), ConsensusError> {
        // Basic transaction structure validation
        tx.validate()
            .map_err(|e| ConsensusError::InvalidTransaction(e))?;
        
        // Additional consensus-level validation would go here
        // - UTXO validation
        // - Signature verification
        // - Fee validation
        // - RevStop checks
        
        Ok(())
    }
    
    /// Adjust difficulty based on block timing
    #[instrument(skip(self))]
    pub fn adjust_difficulty(&self, height: u64, time_taken: u64) -> Result<u32, ConsensusError> {
        self.engine.adjust_difficulty(height, time_taken)
    }
    
    /// Resolve chain forks
    #[instrument(skip(self))]
    pub fn resolve_forks(&self) -> Result<String, ConsensusError> {
        self.engine.resolve_forks()
    }
    
    /// Calculate block reward for given height
    pub fn calculate_block_reward(&self, height: u64) -> u64 {
        self.economics.block_reward(height)
    }
    
    /// Get current chain state
    pub fn get_chain_state(&self) -> crate::consensus_engine::ChainState {
        self.engine.get_chain_state()
    }
    
    /// Get current difficulty
    pub fn get_current_difficulty(&self) -> u32 {
        self.engine.get_current_difficulty()
    }
    
    /// Detect network partitions
    pub fn detect_network_partition(&self, peer_heights: &[u64]) -> bool {
        self.engine.detect_network_partition(peer_heights)
    }
    
    /// Update network time from peer samples
    pub fn update_network_time(&self, peer_times: &[u64]) {
        self.engine.update_network_time(peer_times)
    }
    
    /// Get chain specification
    pub fn get_chain_spec(&self) -> &ChainSpec {
        &self.spec
    }
    
    /// Get economics engine
    pub fn get_economics(&self) -> &Economics {
        &self.economics
    }
    
    /// Validate entire blockchain from genesis
    #[instrument(skip(self, blocks))]
    pub fn validate_chain(&self, blocks: &[Block]) -> Result<(), ConsensusError> {
        if blocks.is_empty() {
            return Ok(());
        }
        
        info!("Validating blockchain with {} blocks", blocks.len());
        
        // Validate genesis block
        self.validate_block(&blocks[0], None)?;
        
        // Validate subsequent blocks
        for i in 1..blocks.len() {
            self.validate_block(&blocks[i], Some(&blocks[i - 1]))
                .with_context(|| format!("Block {} failed validation", i))
                .map_err(|e| ConsensusError::ConfigError(e))?;
        }
        
        info!("Blockchain validation completed successfully");
        Ok(())
    }
    
    /// Perform comprehensive system health check
    pub fn health_check(&self) -> Result<ConsensusHealthReport> {
        let chain_state = self.get_chain_state();
        let difficulty = self.get_current_difficulty();
        
        let report = ConsensusHealthReport {
            chain_height: chain_state.best_block_height,
            best_block_hash: chain_state.best_block_hash,
            total_work: chain_state.total_work,
            current_difficulty: difficulty,
            network_name: self.spec.network.name.clone(),
            version: self.spec.network.version.clone(),
            consensus_algorithm: self.spec.consensus.algorithm.clone(),
            target_block_time: self.spec.consensus.target_block_time,
            max_supply: self.spec.supply.max_supply,
        };
        
        Ok(report)
    }
}

/// Consensus system health report
#[derive(Debug, Clone)]
pub struct ConsensusHealthReport {
    pub chain_height: u64,
    pub best_block_hash: String,
    pub total_work: u128,
    pub current_difficulty: u32,
    pub network_name: String,
    pub version: String,
    pub consensus_algorithm: String,
    pub target_block_time: u64,
    pub max_supply: u64,
}

// Legacy ConsensusEngine for backward compatibility
pub struct ConsensusEngine {
    system: Arc<ConsensusSystem>,
}

impl ConsensusEngine {
    /// Create new legacy consensus engine (backward compatibility)
    pub fn new(economics: Economics) -> Self {
        // Create with default configuration for backward compatibility
        let config = crate::config::ChainConfig::default().shared();
        let system = Arc::new(
            ConsensusSystem::new(config, None)
                .expect("Failed to create consensus system")
        );
        
        Self { system }
    }
    
    /// Create from full configuration
    pub fn with_config(config: SharedConfig, chain_spec_path: Option<&str>) -> Result<Self> {
        let system = Arc::new(ConsensusSystem::new(config, chain_spec_path)?);
        Ok(Self { system })
    }
    
    /// Validate a block against consensus rules
    pub fn validate_block(&self, block: &Block, prev_block: Option<&Block>) -> Result<(), ConsensusError> {
        self.system.validate_block(block, prev_block)
    }
    
    /// Validate a transaction
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), ConsensusError> {
        self.system.validate_transaction(tx)
    }
    
    /// Get the underlying consensus system
    pub fn system(&self) -> &ConsensusSystem {
        &self.system
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ChainConfig;
    
    #[test]
    fn test_consensus_system_creation() {
        let config = ChainConfig::default().shared();
        let system = ConsensusSystem::new(config, None).unwrap();
        
        assert_eq!(system.spec.network.name, "quantumcoin-test");
        assert!(system.get_current_difficulty() > 0);
    }
    
    #[test]
    fn test_legacy_compatibility() {
        let config = ChainConfig::default().shared();
        let economics = Economics::from_shared_config(&config);
        let engine = ConsensusEngine::new(economics);
        
        // Legacy interface should work
        let chain_state = engine.system().get_chain_state();
        assert_eq!(chain_state.best_block_height, 0); // Initial state
    }
    
    #[test]
    fn test_health_check() {
        let config = ChainConfig::default().shared();
        let system = ConsensusSystem::new(config, None).unwrap();
        
        let report = system.health_check().unwrap();
        assert_eq!(report.network_name, "quantumcoin-test");
        assert!(report.max_supply > 0);
        assert!(report.current_difficulty > 0);
    }
}
