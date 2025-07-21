use serde::{Deserialize, Serialize};
use std::{fs, path::Path, error::Error};

use crate::{block::Block, transaction::Transaction};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending: Vec<Transaction>,
    pub difficulty: usize,
    pub reward: u64,
}

impl Blockchain {
    pub fn new_with_genesis(miner: &str) -> Self {
        Self {
            chain: vec![Block::new_genesis(miner)],
            pending: vec![],
            difficulty: 4,
            reward: 25,
        }
    }

    pub fn create_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    pub fn mine_pending_transactions(&mut self, miner: &str) {
        if self.pending.is_empty() { return; }
        self.pending.push(Transaction::coinbase(miner));
        let last = &self.chain[self.chain.len()-1];
        let block = Block::new(
            last.index + 1,
            last.hash.clone(),
            self.pending.clone(),
            self.difficulty,
        );
        self.chain.push(block);
        self.pending.clear();
    }

    pub fn get_balance(&self, addr: &str) -> u64 {
        self.chain.iter().flat_map(|b| &b.transactions)
            .filter(|tx| tx.to == addr).map(|tx| tx.amount).sum::<u64>()
        + self.chain.iter().flat_map(|b| &b.transactions)
            .filter(|tx| tx.from == addr).map(|tx| -(tx.amount as i64))
            .sum::<i64>().max(0) as u64
    }

    pub fn save(&self, file: &str) -> Result<(), Box<dyn Error>> {
        fs::write(file, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn load(file: &str) -> Result<Self, Box<dyn Error>> {
        if !Path::new(file).exists() {
            return Err("no file".into());
        }
        let s = fs::read_to_string(file)?;
        Ok(serde_json::from_str(&s)?)
    }
}