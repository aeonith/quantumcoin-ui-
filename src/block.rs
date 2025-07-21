use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;

use crate::transaction::Transaction;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub prev_hash: String,
    pub nonce: u64,
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    fn calc_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(self.nonce.to_le_bytes());
        for tx in &self.transactions {
            hasher.update(tx.id.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calc_hash();
        }
    }

    pub fn new_genesis(recipient: &str) -> Self {
        let mut b = Block {
            index: 0,
            timestamp: Utc::now().timestamp(),
            prev_hash: "0".repeat(64),
            nonce: 0,
            hash: String::new(),
            transactions: vec![crate::transaction::Transaction::coinbase(recipient)],
        };
        b.hash = b.calc_hash();
        b
    }

    pub fn new(index: u64, prev_hash: String, txs: Vec<Transaction>, diff: usize) -> Self {
        let mut b = Block {
            index,
            timestamp: Utc::now().timestamp(),
            prev_hash,
            nonce: 0,
            hash: String::new(),
            transactions: txs,
        };
        b.mine(diff);
        b
    }
}