use crate::block::Block;
use crate::transaction::Transaction;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    pub mining_reward: u64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            difficulty: 4, // Number of leading zeros required
            mining_reward: 50,
        };

        // Only load from disk — no genesis block
        blockchain.load_from_disk("blockchain.json");

        blockchain
    }

    pub fn create_block(&mut self, miner_address: &str) -> Block {
        let reward_tx = Transaction {
            sender: "Mining Reward".to_string(),
            recipient: miner_address.to_string(),
            amount: self.mining_reward,
            signature: None,
        };

        self.pending_transactions.insert(0, reward_tx);

        let last_hash = self.chain.last().map_or(String::from("0"), |b| b.hash.clone());

        let new_block = Block::new(
            self.pending_transactions.clone(),
            last_hash,
            self.difficulty,
        );

        self.chain.push(new_block.clone());
        self.pending_transactions.clear();

        self.save_to_disk("blockchain.json");

        new_block
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        self.create_block(miner_address);
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0;

        for block in &self.chain {
            for tx in &block.transactions {
                if tx.recipient == address {
                    balance += tx.amount;
                }
                if tx.sender == address {
                    balance -= tx.amount;
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

    pub fn save_to_disk(&self, path: &str) {
        if let Ok(json) = serde_json::to_string(&self.chain) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load_from_disk(&mut self, path: &str) {
        if Path::new(path).exists() {
            if let Ok(mut file) = File::open(path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(chain) = serde_json::from_str::<Vec<Block>>(&contents) {
                        self.chain = chain;
                    }
                }
            }
        }
    }
}