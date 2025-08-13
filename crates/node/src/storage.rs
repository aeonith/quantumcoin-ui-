//! Storage layer for QuantumCoin blockchain data

use crate::{block::Block, transaction::Transaction};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Storage errors
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    /// Block not found
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    
    /// Transaction not found
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// In-memory blockchain storage (for development)
pub struct MemoryStorage {
    /// Blocks indexed by hash
    blocks: RwLock<HashMap<[u8; 32], Block>>,
    
    /// Blocks indexed by height
    blocks_by_height: RwLock<HashMap<u64, [u8; 32]>>,
    
    /// Transactions indexed by hash
    transactions: RwLock<HashMap<[u8; 32], Transaction>>,
    
    /// Current chain tip
    chain_tip: RwLock<Option<[u8; 32]>>,
}

impl MemoryStorage {
    /// Create new memory storage
    pub fn new() -> Self {
        Self {
            blocks: RwLock::new(HashMap::new()),
            blocks_by_height: RwLock::new(HashMap::new()),
            transactions: RwLock::new(HashMap::new()),
            chain_tip: RwLock::new(None),
        }
    }
    
    /// Store a block
    pub fn store_block(&self, block: Block) -> Result<(), StorageError> {
        let block_hash = block.hash();
        let height = block.header.height;
        
        // Store block
        self.blocks.write().insert(block_hash, block);
        self.blocks_by_height.write().insert(height, block_hash);
        
        // Update chain tip if this is the highest block
        let mut tip = self.chain_tip.write();
        if tip.is_none() || height > self.get_block_height(tip.as_ref().unwrap()).unwrap_or(0) {
            *tip = Some(block_hash);
        }
        
        Ok(())
    }
    
    /// Get block by hash
    pub fn get_block(&self, block_hash: &[u8; 32]) -> Result<Block, StorageError> {
        self.blocks
            .read()
            .get(block_hash)
            .cloned()
            .ok_or_else(|| StorageError::BlockNotFound(hex::encode(block_hash)))
    }
    
    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Block, StorageError> {
        let hash = self.blocks_by_height
            .read()
            .get(&height)
            .cloned()
            .ok_or_else(|| StorageError::BlockNotFound(height.to_string()))?;
        
        self.get_block(&hash)
    }
    
    /// Get block height by hash
    pub fn get_block_height(&self, block_hash: &[u8; 32]) -> Result<u64, StorageError> {
        let block = self.get_block(block_hash)?;
        Ok(block.header.height)
    }
    
    /// Get chain tip (highest block)
    pub fn get_chain_tip(&self) -> Option<[u8; 32]> {
        *self.chain_tip.read()
    }
    
    /// Get current chain height
    pub fn get_chain_height(&self) -> u64 {
        if let Some(tip_hash) = self.get_chain_tip() {
            self.get_block_height(&tip_hash).unwrap_or(0)
        } else {
            0
        }
    }
    
    /// Store transaction
    pub fn store_transaction(&self, tx: Transaction) -> Result<(), StorageError> {
        let tx_hash = tx.hash();
        self.transactions.write().insert(tx_hash, tx);
        Ok(())
    }
    
    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &[u8; 32]) -> Result<Transaction, StorageError> {
        self.transactions
            .read()
            .get(tx_hash)
            .cloned()
            .ok_or_else(|| StorageError::TransactionNotFound(hex::encode(tx_hash)))
    }
    
    /// Get blocks in range
    pub fn get_blocks_range(&self, start_height: u64, limit: usize) -> Vec<Block> {
        let mut blocks = Vec::new();
        
        for height in start_height.. {
            if blocks.len() >= limit {
                break;
            }
            
            if let Ok(block) = self.get_block_by_height(height) {
                blocks.push(block);
            } else {
                break;
            }
        }
        
        blocks
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Block, BlockHeader};
    
    fn sample_block(height: u64) -> Block {
        Block {
            header: BlockHeader {
                height,
                previous_hash: [0; 32],
                merkle_root: [0; 32],
                timestamp: 1640995200,
                difficulty: 0x1d00ffff,
                nonce: 0,
            },
            transactions: vec![],
        }
    }
    
    #[test]
    fn test_store_and_retrieve_block() {
        let storage = MemoryStorage::new();
        let block = sample_block(1);
        let block_hash = block.hash();
        
        assert!(storage.store_block(block.clone()).is_ok());
        
        let retrieved = storage.get_block(&block_hash).unwrap();
        assert_eq!(retrieved.header.height, block.header.height);
    }
    
    #[test]
    fn test_get_block_by_height() {
        let storage = MemoryStorage::new();
        let block = sample_block(42);
        
        assert!(storage.store_block(block.clone()).is_ok());
        
        let retrieved = storage.get_block_by_height(42).unwrap();
        assert_eq!(retrieved.header.height, 42);
    }
    
    #[test]
    fn test_chain_tip_updates() {
        let storage = MemoryStorage::new();
        
        assert_eq!(storage.get_chain_height(), 0);
        
        // Store genesis block
        let genesis = sample_block(0);
        assert!(storage.store_block(genesis).is_ok());
        assert_eq!(storage.get_chain_height(), 0);
        
        // Store block 1
        let block1 = sample_block(1);
        assert!(storage.store_block(block1).is_ok());
        assert_eq!(storage.get_chain_height(), 1);
    }
}
