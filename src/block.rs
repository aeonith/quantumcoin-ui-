use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, difficulty: u64) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let mut nonce = 0;
        let mut hash = String::new();

        loop {
            let block_data = format!("{}{}{:?}{}{}", index, timestamp, transactions, previous_hash, nonce);
            let mut hasher = Sha256::new();
            hasher.update(block_data.as_bytes());
            let result = hasher.finalize();
            hash = format!("{:x}", result);

            if &hash[..difficulty as usize] == "0".repeat(difficulty as usize) {
                break;
            }

            nonce += 1;
        }

        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
            nonce,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let block_data = format!("{}{}{:?}{}{}", self.index, self.timestamp, self.transactions, self.previous_hash, self.nonce);
        let mut hasher = Sha256::new();
        hasher.update(block_data.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}