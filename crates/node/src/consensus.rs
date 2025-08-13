//! Consensus rules and validation for QuantumCoin

use crate::{block::Block, transaction::Transaction, economics::Economics};

/// Consensus validation errors
#[derive(thiserror::Error, Debug)]
pub enum ConsensusError {
    /// Invalid block reward
    #[error("Invalid block reward: expected {expected}, got {actual}")]
    InvalidBlockReward { expected: u64, actual: u64 },
    
    /// Block height mismatch
    #[error("Invalid block height: expected {expected}, got {actual}")]
    InvalidHeight { expected: u64, actual: u64 },
}

/// Consensus engine that validates blocks against network rules
pub struct ConsensusEngine {
    economics: Economics,
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new(economics: Economics) -> Self {
        Self { economics }
    }
    
    /// Validate a block against consensus rules
    pub fn validate_block(&self, block: &Block, prev_block: Option<&Block>) -> Result<(), ConsensusError> {
        // Validate block height sequence
        if let Some(prev) = prev_block {
            let expected_height = prev.header.height + 1;
            if block.header.height != expected_height {
                return Err(ConsensusError::InvalidHeight {
                    expected: expected_height,
                    actual: block.header.height,
                });
            }
        }
        
        // Validate proof of work
        if !block.verify_pow() {
            return Err(ConsensusError::InvalidBlockReward { expected: 0, actual: 1 });
        }
        
        // Validate block reward matches economics
        let expected_reward = self.economics.block_reward(block.header.height);
        // This would need actual coinbase transaction validation
        // For now, assume valid
        
        Ok(())
    }
    
    /// Validate a transaction
    pub fn validate_transaction(&self, _tx: &Transaction) -> Result<(), ConsensusError> {
        // Transaction validation logic would go here
        // - Signature verification
        // - UTXO validation  
        // - RevStop checks (if applicable)
        // - Fee validation
        
        Ok(())
    }
}
