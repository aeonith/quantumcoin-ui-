use crate::block::Block;
use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending: Vec<Transaction>,
    pub difficulty: usize,
    pub mining_reward: f64,
}

impl Blockchain {
    pub fn new(genesis_addr: String) -> Blockchain {
        let mut bc = Blockchain {
            chain: Vec::new(),
            pending: Vec::new(),
            difficulty: 2,
            mining_reward: 25.0,
        };
        let genesis = Transaction::new("".into(), genesis_addr, 1_250_000.0);
        let mut block = Block::new(0, vec![genesis], String::from("0"));
        block.mine(bc.difficulty);
        bc.chain.push(block);
        bc
    }

    pub fn load_from_disk() -> Option<Blockchain> {
        if !Path::new("chain.json").exists() { return None; }
        let data = fs::read_to_string("chain.json").ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save_to_disk(&self) {
        let data = serde_json::to_string_pretty(self).unwrap();
        fs::write("chain.json", data).unwrap();
    }

    pub fn mine_pending_transactions(&mut self, miner_addr: String) {
        let txs = self.pending.drain(..).collect();
        let mut block = Block::new(self.chain.len() as u64, txs, self.chain.last().unwrap().hash.clone());
        block.mine(self.difficulty);
        self.chain.push(block);
        self.pending.push(Transaction::new("".into(), miner_addr, self.mining_reward));
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from == address { balance -= tx.amount; }
                if tx.to   == address { balance += tx.amount; }
            }
        }
        balance
    }
}