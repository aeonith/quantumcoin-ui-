use serde::{Deserialize, Serialize};
use std::{fs, path::Path, error::Error};

use crate::{block::Block, transaction::Transaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending: Vec<Transaction>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self { chain: Vec::new(), pending: Vec::new() }
    }
}

impl Blockchain {
    /// Constructor that injects the genesis block if the chain is empty.
    pub fn new_with_genesis(miner_addr: &str) -> Self {
        let mut bc = Self::default();
        bc.chain.push(Block::new_genesis(miner_addr));
        bc
    }

    pub fn create_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    pub fn mine_pending(&mut self, miner_addr: &str) {
        if self.pending.is_empty() { return; }
        let reward = Transaction::coinbase(miner_addr);
        self.pending.push(reward);

        let prev_hash = self.chain.last().unwrap().hash.clone();
        let next_index = self.chain.len() as u64;
        let block = Block::new(next_index, prev_hash, self.pending.clone());

        self.chain.push(block);
        self.pending.clear();
    }

    pub fn calculate_balance(&self, addr: &str) -> u64 {
        let mut bal = 0;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.to == addr { bal += tx.amount; }
                if tx.from == addr { bal = bal.saturating_sub(tx.amount); }
            }
        }
        bal
    }

    // ---------- Persistence ----------

    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        if !Path::new(path).exists() {
            return Err("no blockchain file".into());
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }
}