use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{:?}{}", index, &previous_hash, &transactions, timestamp));
        let hash = format!("{:x}", hasher.finalize());

        Block { index, previous_hash, timestamp, transactions, hash }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::new(0, "0".into(), vec![]);
        Blockchain { chain: vec![genesis] }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let prev = self.chain.last().unwrap();
        let block = Block::new(prev.index + 1, prev.hash.clone(), transactions);
        self.chain.push(block);
    }

    pub fn last_hash(&self) -> String {
        self.chain.last().unwrap().hash.clone()
    }
}