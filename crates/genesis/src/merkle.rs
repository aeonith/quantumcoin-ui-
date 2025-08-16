//! Production-grade Merkle tree implementation for genesis block

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use crate::crypto::{blake3_hash, double_blake3_hash};

/// Merkle tree for transaction organization and verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    /// Root hash of the tree
    pub root: [u8; 32],
    /// All leaf hashes (transaction hashes)
    pub leaves: Vec<[u8; 32]>,
    /// Complete tree structure (level 0 = leaves, top level = root)
    pub levels: Vec<Vec<[u8; 32]>>,
    /// Tree depth
    pub depth: usize,
}

/// Merkle proof for transaction inclusion verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Transaction hash being proved
    pub transaction_hash: [u8; 32],
    /// Index of transaction in leaves
    pub index: usize,
    /// Hashes needed to reconstruct root
    pub path: Vec<[u8; 32]>,
    /// Direction for each level (true = right sibling, false = left sibling)
    pub directions: Vec<bool>,
}

impl MerkleTree {
    /// Create new Merkle tree from transaction hashes
    pub fn new(transaction_hashes: Vec<[u8; 32]>) -> Result<Self> {
        if transaction_hashes.is_empty() {
            return Ok(Self::empty());
        }
        
        let mut levels = Vec::new();
        let mut current_level = transaction_hashes.clone();
        
        // Ensure even number of leaves by duplicating last element if odd
        if current_level.len() % 2 != 0 {
            current_level.push(*current_level.last().unwrap());
        }
        
        levels.push(current_level.clone());
        
        // Build tree bottom-up
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let left = chunk[0];
                let right = chunk.get(1).copied().unwrap_or(left);
                
                let combined_hash = Self::combine_hashes(left, right);
                next_level.push(combined_hash);
            }
            
            levels.push(next_level.clone());
            current_level = next_level;
        }
        
        let root = current_level[0];
        let depth = levels.len() - 1;
        
        Ok(Self {
            root,
            leaves: transaction_hashes,
            levels,
            depth,
        })
    }
    
    /// Create empty Merkle tree
    pub fn empty() -> Self {
        Self {
            root: [0; 32],
            leaves: Vec::new(),
            levels: Vec::new(),
            depth: 0,
        }
    }
    
    /// Combine two hashes using double BLAKE3
    fn combine_hashes(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(&left);
        combined.extend_from_slice(&right);
        double_blake3_hash(&combined)
    }
    
    /// Get the root hash
    pub fn root(&self) -> [u8; 32] {
        self.root
    }
    
    /// Generate Merkle proof for a transaction
    pub fn generate_proof(&self, transaction_hash: [u8; 32]) -> Result<MerkleProof> {
        let index = self.leaves.iter()
            .position(|&leaf| leaf == transaction_hash)
            .context("Transaction hash not found in tree")?;
        
        let mut path = Vec::new();
        let mut directions = Vec::new();
        let mut current_index = index;
        
        // Traverse from leaves to root
        for level in &self.levels[..self.levels.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            
            if sibling_index < level.len() {
                path.push(level[sibling_index]);
                directions.push(current_index % 2 == 0); // true if we're left child
            } else {
                // No sibling (odd number of nodes), use self
                path.push(level[current_index]);
                directions.push(false);
            }
            
            current_index /= 2;
        }
        
        Ok(MerkleProof {
            transaction_hash,
            index,
            path,
            directions,
        })
    }
    
    /// Verify a Merkle proof
    pub fn verify_proof(&self, proof: &MerkleProof) -> bool {
        self.verify_proof_against_root(proof, self.root)
    }
    
    /// Verify a Merkle proof against a specific root
    pub fn verify_proof_against_root(&self, proof: &MerkleProof, root: [u8; 32]) -> bool {
        let mut current_hash = proof.transaction_hash;
        
        for (sibling_hash, &is_left) in proof.path.iter().zip(proof.directions.iter()) {
            if is_left {
                // We are left child, sibling is right
                current_hash = Self::combine_hashes(current_hash, *sibling_hash);
            } else {
                // We are right child, sibling is left
                current_hash = Self::combine_hashes(*sibling_hash, current_hash);
            }
        }
        
        current_hash == root
    }
    
    /// Calculate root from transaction hashes (static method)
    pub fn calculate_root(transaction_hashes: &[[u8; 32]]) -> Result<[u8; 32]> {
        if transaction_hashes.is_empty() {
            return Ok([0; 32]);
        }
        
        let tree = Self::new(transaction_hashes.to_vec())?;
        Ok(tree.root)
    }
}

/// Create deterministic transaction hash for genesis allocations
pub fn create_allocation_transaction_hash(
    address: &str,
    amount: u64,
    purpose: &str,
    index: u32,
) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(address.as_bytes());
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(purpose.as_bytes());
    data.extend_from_slice(&index.to_le_bytes());
    data.extend_from_slice(b"QuantumCoin Genesis Allocation");
    
    double_blake3_hash(&data)
}

/// Create coinbase transaction hash for genesis block
pub fn create_coinbase_transaction_hash(
    message: &str,
    timestamp: u64,
    extra_nonce: &[u8],
) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(message.as_bytes());
    data.extend_from_slice(&timestamp.to_le_bytes());
    data.extend_from_slice(extra_nonce);
    data.extend_from_slice(b"QuantumCoin Genesis Coinbase");
    
    double_blake3_hash(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_merkle_tree() {
        let tree = MerkleTree::empty();
        assert_eq!(tree.root, [0; 32]);
        assert_eq!(tree.depth, 0);
    }
    
    #[test]
    fn test_single_transaction_tree() {
        let tx_hash = [0x42; 32];
        let tree = MerkleTree::new(vec![tx_hash]).unwrap();
        
        // Single transaction trees should have the transaction hash duplicated
        assert_ne!(tree.root, tx_hash); // Root should be hash of (tx_hash || tx_hash)
        assert_eq!(tree.leaves.len(), 1);
        assert_eq!(tree.depth, 1);
    }
    
    #[test]
    fn test_multiple_transaction_tree() {
        let tx_hashes = vec![
            [0x01; 32],
            [0x02; 32],
            [0x03; 32],
            [0x04; 32],
        ];
        
        let tree = MerkleTree::new(tx_hashes.clone()).unwrap();
        assert_eq!(tree.leaves, tx_hashes);
        assert_eq!(tree.depth, 2); // 4 leaves = 2 levels
        assert_ne!(tree.root, [0; 32]);
    }
    
    #[test]
    fn test_odd_number_transactions() {
        let tx_hashes = vec![
            [0x01; 32],
            [0x02; 32],
            [0x03; 32],
        ];
        
        let tree = MerkleTree::new(tx_hashes.clone()).unwrap();
        assert_eq!(tree.leaves, tx_hashes);
        
        // Should handle odd number by duplicating last transaction
        assert_eq!(tree.levels[0].len(), 4); // Padded to 4
        assert_eq!(tree.levels[0][3], [0x03; 32]); // Last tx duplicated
    }
    
    #[test]
    fn test_merkle_proof_generation_and_verification() {
        let tx_hashes = vec![
            [0x01; 32],
            [0x02; 32],
            [0x03; 32],
            [0x04; 32],
        ];
        
        let tree = MerkleTree::new(tx_hashes.clone()).unwrap();
        
        // Test proof for each transaction
        for (i, &tx_hash) in tx_hashes.iter().enumerate() {
            let proof = tree.generate_proof(tx_hash).unwrap();
            assert_eq!(proof.transaction_hash, tx_hash);
            assert_eq!(proof.index, i);
            assert!(tree.verify_proof(&proof));
            
            // Should also verify against the root directly
            assert!(tree.verify_proof_against_root(&proof, tree.root));
        }
    }
    
    #[test]
    fn test_invalid_proof_verification() {
        let tx_hashes = vec![
            [0x01; 32],
            [0x02; 32],
        ];
        
        let tree = MerkleTree::new(tx_hashes).unwrap();
        let proof = tree.generate_proof([0x01; 32]).unwrap();
        
        // Tamper with proof
        let mut tampered_proof = proof.clone();
        tampered_proof.transaction_hash = [0xFF; 32];
        assert!(!tree.verify_proof(&tampered_proof));
        
        // Test against wrong root
        let wrong_root = [0xFF; 32];
        assert!(!tree.verify_proof_against_root(&proof, wrong_root));
    }
    
    #[test]
    fn test_deterministic_hashing() {
        // Same inputs should produce same hashes
        let hash1 = create_allocation_transaction_hash("address1", 1000, "test", 0);
        let hash2 = create_allocation_transaction_hash("address1", 1000, "test", 0);
        assert_eq!(hash1, hash2);
        
        // Different inputs should produce different hashes
        let hash3 = create_allocation_transaction_hash("address2", 1000, "test", 0);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_coinbase_transaction_hash() {
        let hash1 = create_coinbase_transaction_hash("message", 1234567890, &[0, 1, 2, 3]);
        let hash2 = create_coinbase_transaction_hash("message", 1234567890, &[0, 1, 2, 3]);
        assert_eq!(hash1, hash2);
        
        let hash3 = create_coinbase_transaction_hash("different", 1234567890, &[0, 1, 2, 3]);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_calculate_root_static_method() {
        let tx_hashes = vec![
            [0x01; 32],
            [0x02; 32],
        ];
        
        let root1 = MerkleTree::calculate_root(&tx_hashes).unwrap();
        let tree = MerkleTree::new(tx_hashes).unwrap();
        assert_eq!(root1, tree.root);
    }
}
