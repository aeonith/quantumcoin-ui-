//! Block structure and validation for QuantumCoin

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Block validation errors
#[derive(thiserror::Error, Debug)]
pub enum BlockError {
    /// Invalid block hash
    #[error("Invalid block hash")]
    InvalidHash,
    
    /// Invalid timestamp
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    /// Invalid proof of work
    #[error("Invalid proof of work")]
    InvalidPoW,
}

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block height/number
    pub height: u64,
    
    /// Previous block hash
    pub previous_hash: [u8; 32],
    
    /// Merkle root of transactions
    pub merkle_root: [u8; 32],
    
    /// Block timestamp
    pub timestamp: u64,
    
    /// Difficulty target
    pub difficulty: u32,
    
    /// Nonce for proof of work
    pub nonce: u64,
}

/// Complete block with header and transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    
    /// List of transaction IDs
    pub transactions: Vec<[u8; 32]>,
}

impl Block {
    /// Calculate the hash of this block
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        let serialized = bincode::serialize(&self.header).unwrap();
        hasher.update(&serialized);
        hasher.finalize().into()
    }
    
    /// Verify the block's proof of work
    pub fn verify_pow(&self) -> bool {
        let hash = self.hash();
        let hash_value = u256_from_bytes(&hash);
        let target = difficulty_to_target(self.header.difficulty);
        hash_value < target
    }
    
    /// Create genesis block
    pub fn genesis() -> Self {
        Block {
            header: BlockHeader {
                height: 0,
                previous_hash: [0; 32],
                merkle_root: [0; 32],
                timestamp: 1640995200, // Jan 1, 2022 00:00:00 UTC
                difficulty: 0x1d00ffff, // Initial difficulty
                nonce: 0,
            },
            transactions: vec![],
        }
    }
}

// Utility functions for difficulty calculations
fn difficulty_to_target(difficulty: u32) -> [u8; 32] {
    // Simplified difficulty to target conversion
    let mut target = [0xff; 32];
    target[0] = (difficulty >> 24) as u8;
    target[1] = (difficulty >> 16) as u8;
    target[2] = (difficulty >> 8) as u8;
    target[3] = difficulty as u8;
    target
}

fn u256_from_bytes(bytes: &[u8; 32]) -> [u8; 32] {
    *bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.header.previous_hash, [0; 32]);
    }
    
    #[test]
    fn test_block_hash() {
        let genesis = Block::genesis();
        let hash1 = genesis.hash();
        let hash2 = genesis.hash();
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }
}
