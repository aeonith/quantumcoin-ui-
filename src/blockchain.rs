use crate::block::Block;
use crate::transaction::Transaction;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    pub mining_reward: f64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            difficulty: 4,
            mining_reward: 50.0,
        };

        if Path::new("blockchain.json").exists() {
            blockchain.load_from_disk("blockchain.json");
        } else {
            let genesis_block = Block::new(0, vec![], "0");
            blockchain.chain.push(genesis_block);
            blockchain.save_to_disk("blockchain.json");
        }

        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        let reward_tx = Transaction {
            sender: "System".to_string(),
            recipient: miner_address.to_string(),
            amount: self.mining_reward,
            signature: None,
        };

        self.pending_transactions.push(reward_tx);

        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut block = Block::new(self.chain.len() as u64, self.pending_transactions.clone(), &previous_hash);
        block.mine(self.difficulty);

        self.chain.push(block);
        self.pending_transactions.clear();

        self.save_to_disk("blockchain.json");
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;

        for block in &self.chain {
            for tx in &block.transactions {
                if tx.sender == address {
                    balance -= tx.amount;
                }
                if tx.recipient == address {
                    balance += tx.amount;
                }
            }
        }

        balance
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.calculate_hash() {
                return false;
            }

            if current.previous_hash != previous.hash {
                return false;
            }
        }

        true
    }

    pub fn save_to_disk(&self, filename: &str) {
        let serialized = serde_json::to_string(&self.chain).unwrap();
        let mut file = File::create(filename).unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }

    pub fn load_from_disk(&mut self, filename: &str) {
        if let Ok(mut file) = File::open(filename) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            if let Ok(chain) = serde_json::from_str(&contents) {
                self.chain = chain;
            }
        }
    }
}