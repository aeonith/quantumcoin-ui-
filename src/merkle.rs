use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleTree {
    pub root: String,
    pub leaves: Vec<String>,
    pub tree: Vec<Vec<String>>,
}

impl MerkleTree {
    pub fn new(transactions: &[Transaction]) -> Self {
        if transactions.is_empty() {
            return MerkleTree {
                root: String::new(),
                leaves: vec![],
                tree: vec![],
            };
        }

        let mut leaves: Vec<String> = transactions
            .iter()
            .map(|tx| tx.calculate_hash())
            .collect();

        // Ensure even number of leaves
        if leaves.len() % 2 != 0 {
            leaves.push(leaves.last().unwrap().clone());
        }

        let mut tree = vec![leaves.clone()];
        let mut current_level = leaves;

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    left // Duplicate if odd number
                };
                
                let combined = format!("{}{}", left, right);
                let hash = Self::hash(&combined);
                next_level.push(hash);
            }
            
            tree.push(next_level.clone());
            current_level = next_level;
        }

        let root = current_level.first().unwrap_or(&String::new()).clone();

        MerkleTree {
            root,
            leaves,
            tree,
        }
    }

    pub fn generate_proof(&self, transaction_hash: &str) -> Option<Vec<String>> {
        // Find the index of the transaction in leaves
        let index = self.leaves.iter().position(|leaf| leaf == transaction_hash)?;
        
        let mut proof = Vec::new();
        let mut current_index = index;
        
        for level in &self.tree[..self.tree.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            
            if sibling_index < level.len() {
                proof.push(level[sibling_index].clone());
            }
            
            current_index /= 2;
        }
        
        Some(proof)
    }

    pub fn verify_proof(
        transaction_hash: &str,
        proof: &[String],
        root: &str,
        index: usize,
    ) -> bool {
        let mut current_hash = transaction_hash.to_string();
        let mut current_index = index;
        
        for sibling in proof {
            if current_index % 2 == 0 {
                current_hash = Self::hash(&format!("{}{}", current_hash, sibling));
            } else {
                current_hash = Self::hash(&format!("{}{}", sibling, current_hash));
            }
            current_index /= 2;
        }
        
        current_hash == root
    }

    fn hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn get_root(&self) -> &str {
        &self.root
    }

    pub fn get_depth(&self) -> usize {
        self.tree.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    #[test]
    fn test_merkle_tree_single_transaction() {
        let transactions = vec![
            Transaction::new("Alice".to_string(), "Bob".to_string(), 100, 10, 1)
        ];
        let tree = MerkleTree::new(&transactions);
        assert!(!tree.root.is_empty());
    }

    #[test]
    fn test_merkle_proof() {
        let transactions = vec![
            Transaction::new("Alice".to_string(), "Bob".to_string(), 100, 10, 1),
            Transaction::new("Bob".to_string(), "Charlie".to_string(), 50, 10, 2),
        ];
        let tree = MerkleTree::new(&transactions);
        let tx_hash = transactions[0].calculate_hash();
        let proof = tree.generate_proof(&tx_hash).unwrap();
        
        assert!(MerkleTree::verify_proof(&tx_hash, &proof, &tree.root, 0));
    }
}
