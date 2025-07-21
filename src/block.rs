use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;

use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub prev_hash: String,
    pub nonce: u64,
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Hashes all block fields except `hash` itself.
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

    /// Simple, slow proof-of-work (four leading zeros).
    pub fn mine(&mut self) {
        while !self.hash.starts_with("0000") {
            self.nonce += 1;
            self.hash = self.calc_hash();
        }
    }

    pub fn new_genesis(recipient_addr: &str) -> Self {
        let mut genesis = Self {
            index: 0,
            timestamp: Utc::now().timestamp(),
            prev_hash: "0".repeat(64),
            nonce: 0,
            hash: String::new(),
            transactions: vec![Transaction::coinbase(recipient_addr)],
        };
        genesis.hash = genesis.calc_hash();
        genesis
    }

    pub fn new(next_index: u64,
               prev_hash: String,
               txs: Vec<Transaction>) -> Self {
        let mut block = Self {
            index: next_index,
            timestamp: Utc::now().timestamp(),
            prev_hash,
            nonce: 0,
            hash: String::new(),
            transactions: txs,
        };
        block.mine();
        block
    }
}