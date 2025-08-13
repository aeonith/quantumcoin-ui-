use crate::Transaction;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub previous_hash: String,
    pub transactions: Vec<Transaction>,
    pub merkle_root: String,
    pub nonce: u64,
    pub hash: String,
}

impl Block {
    pub fn new(previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let mut block = Block {
            index: 0,
            timestamp: Utc::now(),
            previous_hash,
            transactions,
            merkle_root: String::new(),
            nonce: 0,
            hash: String::new(),
        };
        
        block.merkle_root = block.calculate_merkle_root();
        block
    }
    
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let data = format!(
            "{}{}{}{}{}{}",
            self.index,
            self.timestamp.timestamp(),
            self.previous_hash,
            self.merkle_root,
            self.nonce,
            self.transactions.len()
        );
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    pub fn calculate_merkle_root(&self) -> String {
        if self.transactions.is_empty() {
            return String::new();
        }
        
        let mut hashes: Vec<String> = self.transactions
            .iter()
            .map(|tx| {
                let mut hasher = Sha256::new();
                hasher.update(format!("{}{}{}{}", tx.id, tx.sender, tx.recipient, tx.amount));
                format!("{:x}", hasher.finalize())
            })
            .collect();
        
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    format!("{}{}", chunk[0], chunk[0])
                };
                
                let mut hasher = Sha256::new();
                hasher.update(combined);
                next_level.push(format!("{:x}", hasher.finalize()));
            }
            
            hashes = next_level;
        }
        
        hashes.into_iter().next().unwrap_or_default()
    }
    
    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash() && self.merkle_root == self.calculate_merkle_root()
    }
}
