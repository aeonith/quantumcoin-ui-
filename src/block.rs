use crate::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64,
               transactions: Vec<Transaction>,
               previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let nonce = 0;
        let hash = Self::calculate_hash(index, timestamp, &transactions, &previous_hash, nonce);
        Self { index, timestamp, transactions, previous_hash, nonce, hash }
    }

    fn calculate_hash(index: u64,
                      timestamp: i64,
                      txs: &[Transaction],
                      prev: &str,
                      nonce: u64) -> String {
        // 100 % fake hash – replace with real hashing later
        format!("h{:x}{:x}", index ^ nonce, timestamp) + &prev[..8.min(prev.len())]
            + &txs.len().to_string()
    }
}