//! Genesis block structure and serialization

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::crypto::{QuantumSignature, double_blake3_hash};
use crate::merkle::MerkleTree;
use anyhow::{Result, Context};

/// Genesis block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block version
    pub version: u32,
    /// Previous block hash (all zeros for genesis)
    pub previous_hash: [u8; 32],
    /// Merkle root of all transactions
    pub merkle_root: [u8; 32],
    /// Block timestamp
    pub timestamp: DateTime<Utc>,
    /// Difficulty target
    pub difficulty: u32,
    /// Nonce for proof-of-work
    pub nonce: u64,
    /// Extra nonce for extended proof-of-work space
    pub extra_nonce: Vec<u8>,
}

/// Genesis transaction representing initial coin allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisTransaction {
    /// Transaction type (coinbase or allocation)
    pub tx_type: TransactionType,
    /// Output address
    pub address: String,
    /// Amount in satoshis
    pub amount: u64,
    /// Transaction message/purpose
    pub message: String,
    /// Transaction index within genesis block
    pub index: u32,
    /// Transaction hash
    pub hash: [u8; 32],
}

/// Types of genesis transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Coinbase transaction with mining reward
    Coinbase,
    /// Initial allocation transaction
    Allocation { purpose: String },
}

/// Complete genesis block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    /// Block header
    pub header: BlockHeader,
    /// All genesis transactions
    pub transactions: Vec<GenesisTransaction>,
    /// Merkle tree of transactions
    pub merkle_tree: MerkleTree,
    /// Block hash
    pub hash: [u8; 32],
    /// Post-quantum signature of the block
    pub signature: Option<QuantumSignature>,
    /// Chain specification hash this block was created from
    pub chain_spec_hash: [u8; 32],
    /// Creation metadata
    pub metadata: GenesisMetadata,
}

/// Genesis block creation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisMetadata {
    /// Creator information
    pub creator: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Chain specification version used
    pub chain_spec_version: String,
    /// Genesis creation parameters
    pub creation_params: CreationParams,
}

/// Parameters used for genesis creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationParams {
    /// Network magic bytes used
    pub network_magic: [u8; 4],
    /// Genesis message
    pub genesis_message: String,
    /// Total supply
    pub max_supply: u64,
    /// Initial block reward
    pub initial_reward: u64,
    /// Whether deterministic mode was used
    pub deterministic: bool,
}

impl GenesisBlock {
    /// Create new genesis block
    pub fn new(
        header: BlockHeader,
        transactions: Vec<GenesisTransaction>,
        chain_spec_hash: [u8; 32],
        metadata: GenesisMetadata,
    ) -> Result<Self> {
        // Create merkle tree from transaction hashes
        let tx_hashes: Vec<[u8; 32]> = transactions.iter()
            .map(|tx| tx.hash)
            .collect();
        
        let merkle_tree = MerkleTree::new(tx_hashes)
            .context("Failed to create merkle tree")?;
        
        // Validate merkle root matches header
        if header.merkle_root != merkle_tree.root() {
            anyhow::bail!("Merkle root mismatch");
        }
        
        // Calculate block hash
        let hash = Self::calculate_block_hash(&header)?;
        
        Ok(Self {
            header,
            transactions,
            merkle_tree,
            hash,
            signature: None,
            chain_spec_hash,
            metadata,
        })
    }
    
    /// Calculate deterministic block hash
    pub fn calculate_block_hash(header: &BlockHeader) -> Result<[u8; 32]> {
        let serialized = bincode::serialize(header)
            .context("Failed to serialize block header")?;
        Ok(double_blake3_hash(&serialized))
    }
    
    /// Sign the genesis block with post-quantum signature
    pub fn sign(&mut self, signature: QuantumSignature) {
        self.signature = Some(signature);
    }
    
    /// Verify the genesis block signature
    pub fn verify_signature(&self) -> Result<bool> {
        let signature = self.signature.as_ref()
            .context("No signature present")?;
        
        let serialized = bincode::serialize(&self.header)
            .context("Failed to serialize header for verification")?;
        
        crate::crypto::verify_quantum_signature(&serialized, signature)
    }
    
    /// Get total allocated amount in genesis block
    pub fn total_allocation(&self) -> u64 {
        self.transactions.iter()
            .map(|tx| tx.amount)
            .sum()
    }
    
    /// Get coinbase transaction if present
    pub fn coinbase_transaction(&self) -> Option<&GenesisTransaction> {
        self.transactions.iter()
            .find(|tx| matches!(tx.tx_type, TransactionType::Coinbase))
    }
    
    /// Get allocation transactions
    pub fn allocation_transactions(&self) -> Vec<&GenesisTransaction> {
        self.transactions.iter()
            .filter(|tx| matches!(tx.tx_type, TransactionType::Allocation { .. }))
            .collect()
    }
    
    /// Validate block structure and consistency
    pub fn validate(&self) -> Result<()> {
        // Verify genesis constraints
        if self.header.previous_hash != [0; 32] {
            anyhow::bail!("Genesis block must have zero previous hash");
        }
        
        // Verify merkle root
        if self.header.merkle_root != self.merkle_tree.root() {
            anyhow::bail!("Merkle root mismatch");
        }
        
        // Verify block hash
        let calculated_hash = Self::calculate_block_hash(&self.header)?;
        if self.hash != calculated_hash {
            anyhow::bail!("Block hash mismatch");
        }
        
        // Verify transaction hashes match merkle tree leaves
        let tx_hashes: Vec<[u8; 32]> = self.transactions.iter()
            .map(|tx| tx.hash)
            .collect();
        
        if tx_hashes != self.merkle_tree.leaves {
            anyhow::bail!("Transaction hashes don't match merkle tree leaves");
        }
        
        // Verify transaction indices are sequential
        for (i, tx) in self.transactions.iter().enumerate() {
            if tx.index as usize != i {
                anyhow::bail!("Invalid transaction index: expected {}, got {}", i, tx.index);
            }
        }
        
        // Verify signature if present
        if let Some(_) = &self.signature {
            if !self.verify_signature()? {
                anyhow::bail!("Invalid block signature");
            }
        }
        
        Ok(())
    }
    
    /// Serialize to bytes for storage/transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .context("Failed to serialize genesis block")
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .context("Failed to deserialize genesis block")
    }
    
    /// Export to JSON format
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize to JSON")
    }
    
    /// Import from JSON format
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Failed to deserialize from JSON")
    }
    
    /// Get block hash as hex string
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash)
    }
    
    /// Get chain spec hash as hex string
    pub fn chain_spec_hash_hex(&self) -> String {
        hex::encode(self.chain_spec_hash)
    }
}

impl GenesisTransaction {
    /// Create new coinbase transaction
    pub fn new_coinbase(
        address: String,
        amount: u64,
        message: String,
        index: u32,
        timestamp: DateTime<Utc>,
        extra_nonce: &[u8],
    ) -> Self {
        let hash = crate::merkle::create_coinbase_transaction_hash(
            &message,
            timestamp.timestamp() as u64,
            extra_nonce,
        );
        
        Self {
            tx_type: TransactionType::Coinbase,
            address,
            amount,
            message,
            index,
            hash,
        }
    }
    
    /// Create new allocation transaction
    pub fn new_allocation(
        address: String,
        amount: u64,
        purpose: String,
        index: u32,
    ) -> Self {
        let hash = crate::merkle::create_allocation_transaction_hash(
            &address,
            amount,
            &purpose,
            index,
        );
        
        Self {
            tx_type: TransactionType::Allocation { 
                purpose: purpose.clone() 
            },
            address,
            amount,
            message: purpose,
            index,
            hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_genesis_transaction_creation() {
        let coinbase = GenesisTransaction::new_coinbase(
            "0000000000000000000000000000000000000000".to_string(),
            0, // NO PREMINE - empty genesis coinbase
            "Genesis coinbase - fair launch".to_string(),
            0,
            Utc::now(),
            &[0, 1, 2, 3],
        );
        
        assert!(matches!(coinbase.tx_type, TransactionType::Coinbase));
        assert_eq!(coinbase.amount, 5000000000);
        assert_ne!(coinbase.hash, [0; 32]);
    }
    
    #[test]
    fn test_genesis_block_creation() {
        let header = BlockHeader {
            version: 1,
            previous_hash: [0; 32],
            merkle_root: [0; 32], // Will be updated
            timestamp: Utc::now(),
            difficulty: 0x1d00ffff,
            nonce: 0,
            extra_nonce: vec![0, 1, 2, 3],
        };
        
        let coinbase = GenesisTransaction::new_coinbase(
            "0000000000000000000000000000000000000000".to_string(),
            0, // NO PREMINE - empty genesis coinbase
            "Genesis coinbase - fair launch".to_string(),
            0,
            Utc::now(),
            &[0, 1, 2, 3],
        );
        
        let transactions = vec![coinbase];
        let merkle_tree = MerkleTree::new(transactions.iter().map(|tx| tx.hash).collect()).unwrap();
        
        let mut header = header;
        header.merkle_root = merkle_tree.root();
        
        let metadata = GenesisMetadata {
            creator: "QuantumCoin Genesis Builder".to_string(),
            created_at: Utc::now(),
            chain_spec_version: "2.0.0".to_string(),
            creation_params: CreationParams {
                network_magic: [0x51, 0x54, 0x43, 0x4D],
                genesis_message: "Test genesis".to_string(),
                max_supply: 22000000000000000,
                initial_reward: 5000000000,
                deterministic: true,
            },
        };
        
        let chain_spec_hash = [0x42; 32];
        
        let block = GenesisBlock::new(header, transactions, chain_spec_hash, metadata).unwrap();
        
        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.total_allocation(), 5000000000);
        assert!(block.coinbase_transaction().is_some());
        assert!(block.allocation_transactions().is_empty());
    }
    
    #[test]
    fn test_block_validation() {
        let header = BlockHeader {
            version: 1,
            previous_hash: [0; 32],
            merkle_root: [0; 32],
            timestamp: Utc::now(),
            difficulty: 0x1d00ffff,
            nonce: 0,
            extra_nonce: vec![],
        };
        
        let coinbase = GenesisTransaction::new_coinbase(
            "test".to_string(),
            100,
            "test".to_string(),
            0,
            Utc::now(),
            &[],
        );
        
        let transactions = vec![coinbase];
        let merkle_tree = MerkleTree::new(transactions.iter().map(|tx| tx.hash).collect()).unwrap();
        
        let mut header = header;
        header.merkle_root = merkle_tree.root();
        
        let metadata = GenesisMetadata {
            creator: "Test".to_string(),
            created_at: Utc::now(),
            chain_spec_version: "2.0.0".to_string(),
            creation_params: CreationParams {
                network_magic: [0; 4],
                genesis_message: "".to_string(),
                max_supply: 1000,
                initial_reward: 100,
                deterministic: false,
            },
        };
        
        let block = GenesisBlock::new(header, transactions, [0; 32], metadata).unwrap();
        assert!(block.validate().is_ok());
    }
    
    #[test]
    fn test_serialization() {
        let header = BlockHeader {
            version: 1,
            previous_hash: [0; 32],
            merkle_root: [1; 32],
            timestamp: Utc::now(),
            difficulty: 0x1d00ffff,
            nonce: 42,
            extra_nonce: vec![1, 2, 3],
        };
        
        let coinbase = GenesisTransaction::new_coinbase(
            "test".to_string(),
            100,
            "test".to_string(),
            0,
            Utc::now(),
            &[1, 2, 3],
        );
        
        let transactions = vec![coinbase];
        let merkle_tree = MerkleTree::new(transactions.iter().map(|tx| tx.hash).collect()).unwrap();
        
        let mut header = header;
        header.merkle_root = merkle_tree.root();
        
        let metadata = GenesisMetadata {
            creator: "Test".to_string(),
            created_at: Utc::now(),
            chain_spec_version: "2.0.0".to_string(),
            creation_params: CreationParams {
                network_magic: [0; 4],
                genesis_message: "".to_string(),
                max_supply: 1000,
                initial_reward: 100,
                deterministic: false,
            },
        };
        
        let block = GenesisBlock::new(header, transactions, [0; 32], metadata).unwrap();
        
        // Test binary serialization
        let bytes = block.to_bytes().unwrap();
        let restored = GenesisBlock::from_bytes(&bytes).unwrap();
        assert_eq!(block.hash, restored.hash);
        
        // Test JSON serialization
        let json = block.to_json().unwrap();
        let restored = GenesisBlock::from_json(&json).unwrap();
        assert_eq!(block.hash, restored.hash);
    }
}
