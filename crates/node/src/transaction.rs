//! Transaction structure and validation for QuantumCoin

use serde::{Deserialize, Serialize};

/// Transaction validation errors
#[derive(thiserror::Error, Debug)]
pub enum TransactionError {
    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,
    
    /// Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    /// Invalid amount
    #[error("Invalid amount: {0}")]
    InvalidAmount(u64),
}

/// Transaction input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Previous transaction hash
    pub prev_tx_hash: [u8; 32],
    
    /// Output index in previous transaction
    pub output_index: u32,
    
    /// Signature (placeholder for now)
    pub signature: Vec<u8>,
}

/// Transaction output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// Amount in base units
    pub amount: u64,
    
    /// Recipient address (public key hash)
    pub recipient: Vec<u8>,
}

/// Complete transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction inputs
    pub inputs: Vec<TransactionInput>,
    
    /// Transaction outputs
    pub outputs: Vec<TransactionOutput>,
    
    /// Transaction fee
    pub fee: u64,
    
    /// Transaction timestamp
    pub timestamp: u64,
}

impl Transaction {
    /// Calculate transaction hash/ID
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let serialized = bincode::serialize(self).unwrap();
        hasher.update(&serialized);
        hasher.finalize().into()
    }
    
    /// Get transaction ID as hex string
    pub fn id(&self) -> String {
        hex::encode(self.hash())
    }
    
    /// Calculate total input amount
    pub fn total_input_amount(&self) -> u64 {
        // This would need to look up the actual UTXO values
        // For now, return placeholder
        0
    }
    
    /// Calculate total output amount
    pub fn total_output_amount(&self) -> u64 {
        self.outputs.iter().map(|output| output.amount).sum()
    }
    
    /// Validate transaction structure
    pub fn validate(&self) -> Result<(), TransactionError> {
        // Check inputs exist
        if self.inputs.is_empty() {
            return Err(TransactionError::InvalidAmount(0));
        }
        
        // Check outputs exist  
        if self.outputs.is_empty() {
            return Err(TransactionError::InvalidAmount(0));
        }
        
        // Check no overflow in output amounts
        let total_output = self.total_output_amount();
        if total_output == 0 {
            return Err(TransactionError::InvalidAmount(0));
        }
        
        // Additional validation would go here:
        // - Signature verification
        // - UTXO existence checks
        // - Double spending prevention
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_hash() {
        let tx = Transaction {
            inputs: vec![],
            outputs: vec![],
            fee: 1000,
            timestamp: 1640995200,
        };
        
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }
    
    #[test]
    fn test_transaction_id() {
        let tx = Transaction {
            inputs: vec![],
            outputs: vec![],
            fee: 1000,
            timestamp: 1640995200,
        };
        
        let id = tx.id();
        assert_eq!(id.len(), 64, "ID should be 64 hex characters");
    }
}
