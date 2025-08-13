//! Memory pool for pending transactions

use crate::transaction::{Transaction, TransactionError};
use parking_lot::RwLock;
use std::collections::HashMap;

/// Mempool errors
#[derive(thiserror::Error, Debug)]
pub enum MempoolError {
    /// Transaction already exists
    #[error("Transaction already exists: {0}")]
    DuplicateTransaction(String),
    
    /// Transaction validation failed
    #[error("Transaction validation failed: {0}")]
    ValidationFailed(#[from] TransactionError),
}

/// In-memory transaction pool
pub struct Mempool {
    /// Pending transactions indexed by hash
    transactions: RwLock<HashMap<[u8; 32], Transaction>>,
}

impl Mempool {
    /// Create new mempool
    pub fn new() -> Self {
        Self {
            transactions: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add transaction to mempool
    pub fn add_transaction(&self, tx: Transaction) -> Result<(), MempoolError> {
        // Validate transaction
        tx.validate()?;
        
        let tx_hash = tx.hash();
        let mut txs = self.transactions.write();
        
        // Check for duplicates
        if txs.contains_key(&tx_hash) {
            return Err(MempoolError::DuplicateTransaction(hex::encode(tx_hash)));
        }
        
        txs.insert(tx_hash, tx);
        Ok(())
    }
    
    /// Remove transaction from mempool
    pub fn remove_transaction(&self, tx_hash: &[u8; 32]) -> Option<Transaction> {
        self.transactions.write().remove(tx_hash)
    }
    
    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &[u8; 32]) -> Option<Transaction> {
        self.transactions.read().get(tx_hash).cloned()
    }
    
    /// Get all transactions (for block building)
    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        self.transactions.read().values().cloned().collect()
    }
    
    /// Get mempool size
    pub fn size(&self) -> usize {
        self.transactions.read().len()
    }
    
    /// Clear mempool
    pub fn clear(&self) {
        self.transactions.write().clear();
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn sample_transaction() -> Transaction {
        Transaction {
            inputs: vec![],
            outputs: vec![],
            fee: 1000,
            timestamp: 1640995200,
        }
    }
    
    #[test]
    fn test_add_transaction() {
        let mempool = Mempool::new();
        let tx = sample_transaction();
        let tx_hash = tx.hash();
        
        assert!(mempool.add_transaction(tx.clone()).is_ok());
        assert_eq!(mempool.size(), 1);
        
        let retrieved = mempool.get_transaction(&tx_hash);
        assert!(retrieved.is_some());
    }
    
    #[test]
    fn test_duplicate_transaction() {
        let mempool = Mempool::new();
        let tx = sample_transaction();
        
        assert!(mempool.add_transaction(tx.clone()).is_ok());
        assert!(mempool.add_transaction(tx).is_err());
    }
    
    #[test]
    fn test_remove_transaction() {
        let mempool = Mempool::new();
        let tx = sample_transaction();
        let tx_hash = tx.hash();
        
        assert!(mempool.add_transaction(tx.clone()).is_ok());
        assert_eq!(mempool.size(), 1);
        
        let removed = mempool.remove_transaction(&tx_hash);
        assert!(removed.is_some());
        assert_eq!(mempool.size(), 0);
    }
}
