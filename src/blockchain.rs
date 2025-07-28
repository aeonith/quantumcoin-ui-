use crate::block::{Block, calculate_hash};
use crate::transaction::Transaction;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut chain = Vec::new();

        // Create empty genesis block
        let genesis_block = Block {
            index: 0,
            timestamp: Blockchain::now(),
            transactions: vec![],
            previous_hash: String::from("0"),
            nonce: 0,
            hash: String::new(),
        };

        let mut genesis_block = genesis_block;
        genesis_block.hash = calculate_hash(&genesis_block);

        chain.push(genesis_block);

        Blockchain {
            chain,
            difficulty: 4,
            pending_transactions: vec![],
            mining_reward: 50.0,
        }
    }

    fn now() -> u128 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        let reward_tx = Transaction {
            sender: "SYSTEM".to_string(),
            recipient: miner_address.to_string(),
            amount: self.mining_reward,
            signature: None,
        };
        self.pending_transactions.push(reward_tx);

        let block = Block::new(
            self.chain.len() as u64,
            Blockchain::now(),
            self.pending_transactions.clone(),
            &self.chain.last().unwrap().hash,
            self.difficulty,
        );

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

            if current.hash != calculate_hash(current) {
                return false;
            }

            if current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }

    pub fn save_to_disk(&self, path: &str) {
        let json = serde_json::to_string_pretty(self).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    pub fn load_from_disk(path: &str) -> Option<Self> {
        if let Ok(mut file) = File::open(path) {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }
}