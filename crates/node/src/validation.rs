//! Validation rules and utilities for QuantumCoin

use crate::{block::Block, transaction::Transaction, economics::Economics, NodeError};

/// Validation errors
#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    /// Invalid block
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    
    /// Invalid transaction
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    /// Economics violation
    #[error("Economics violation: {0}")]
    EconomicsViolation(String),
}

/// Block and transaction validator
pub struct Validator {
    economics: Economics,
}

impl Validator {
    /// Create new validator
    pub fn new(economics: Economics) -> Self {
        Self { economics }
    }
    
    /// Validate block structure and economics
    pub fn validate_block(&self, block: &Block) -> Result<(), ValidationError> {
        // Validate block height is reasonable
        if block.header.height > u64::MAX - 1000 {
            return Err(ValidationError::InvalidBlock(
                "Block height too high".to_string()
            ));
        }
        
        // Validate timestamp is reasonable (within 2 hours of now)
        let now = chrono::Utc::now().timestamp() as u64;
        if block.header.timestamp > now + 7200 {
            return Err(ValidationError::InvalidBlock(
                "Block timestamp too far in future".to_string()
            ));
        }
        
        // Validate proof of work
        if !block.verify_pow() {
            return Err(ValidationError::InvalidBlock(
                "Invalid proof of work".to_string()
            ));
        }
        
        // Validate block reward matches economics (would need coinbase tx)
        let expected_reward = self.economics.block_reward(block.header.height);
        if expected_reward == 0 && block.header.height > 0 {
            // This means no more rewards should be issued
            // Would validate coinbase transaction here
        }
        
        Ok(())
    }
    
    /// Validate transaction
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), ValidationError> {
        // Basic structure validation
        tx.validate().map_err(|e| {
            ValidationError::InvalidTransaction(e.to_string())
        })?;
        
        // Check for reasonable timestamp (within 1 hour)
        let now = chrono::Utc::now().timestamp() as u64;
        if tx.timestamp > now + 3600 {
            return Err(ValidationError::InvalidTransaction(
                "Transaction timestamp too far in future".to_string()
            ));
        }
        
        // Additional validation would include:
        // - Signature verification with quantum-safe crypto
        // - UTXO existence and ownership
        // - Double spending checks
        // - RevStop status (if applicable to sender)
        // - Fee adequacy
        
        Ok(())
    }
    
    /// Validate chain consistency
    pub fn validate_chain(&self, blocks: &[Block]) -> Result<(), ValidationError> {
        if blocks.is_empty() {
            return Ok(());
        }
        
        // Validate genesis block
        if blocks[0].header.height != 0 {
            return Err(ValidationError::InvalidBlock(
                "First block must be genesis (height 0)".to_string()
            ));
        }
        
        // Validate chain continuity
        for window in blocks.windows(2) {
            let prev_block = &window[0];
            let curr_block = &window[1];
            
            // Check height sequence
            if curr_block.header.height != prev_block.header.height + 1 {
                return Err(ValidationError::InvalidBlock(
                    format!(
                        "Non-sequential heights: {} -> {}",
                        prev_block.header.height,
                        curr_block.header.height
                    )
                ));
            }
            
            // Check previous hash linkage
            let prev_hash = prev_block.hash();
            if curr_block.header.previous_hash != prev_hash {
                return Err(ValidationError::InvalidBlock(
                    "Invalid previous block hash linkage".to_string()
                ));
            }
            
            // Validate individual block
            self.validate_block(curr_block)?;
        }
        
        Ok(())
    }
    
    /// Check if total issuance is within bounds
    pub fn validate_issuance(&self, height: u64) -> Result<(), ValidationError> {
        let issued = self.economics.cumulative_issuance(height);
        let max_supply = self.economics.max_supply();
        
        if issued > max_supply {
            return Err(ValidationError::EconomicsViolation(
                format!(
                    "Total issuance {} exceeds maximum supply {}",
                    issued, max_supply
                )
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::ChainConfig, block::BlockHeader};
    
    fn test_validator() -> Validator {
        let config = ChainConfig::default();
        let economics = Economics::new(config.economics);
        Validator::new(economics)
    }
    
    fn sample_block(height: u64, prev_hash: [u8; 32]) -> Block {
        Block {
            header: BlockHeader {
                height,
                previous_hash: prev_hash,
                merkle_root: [0; 32],
                timestamp: chrono::Utc::now().timestamp() as u64,
                difficulty: 0x1d00ffff,
                nonce: 0,
            },
            transactions: vec![],
        }
    }
    
    #[test]
    fn test_validate_valid_chain() {
        let validator = test_validator();
        
        let genesis = sample_block(0, [0; 32]);
        let block1 = sample_block(1, genesis.hash());
        let block2 = sample_block(2, block1.hash());
        
        let chain = vec![genesis, block1, block2];
        assert!(validator.validate_chain(&chain).is_ok());
    }
    
    #[test]
    fn test_validate_invalid_height_sequence() {
        let validator = test_validator();
        
        let genesis = sample_block(0, [0; 32]);
        let block2 = sample_block(2, genesis.hash()); // Skip block 1
        
        let chain = vec![genesis, block2];
        assert!(validator.validate_chain(&chain).is_err());
    }
    
    #[test]
    fn test_validate_invalid_previous_hash() {
        let validator = test_validator();
        
        let genesis = sample_block(0, [0; 32]);
        let block1 = sample_block(1, [1; 32]); // Wrong previous hash
        
        let chain = vec![genesis, block1];
        assert!(validator.validate_chain(&chain).is_err());
    }
    
    #[test]
    fn test_validate_issuance() {
        let validator = test_validator();
        
        // Should be valid for reasonable heights
        assert!(validator.validate_issuance(1000).is_ok());
        assert!(validator.validate_issuance(100000).is_ok());
    }
}
