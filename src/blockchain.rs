use crate::block::Block;
use crate::transaction::Transaction;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blocks: vec![],
            difficulty: 4,
            pending_transactions: vec![],
        };

        // Load from disk if exists
        if let Ok(mut file) = File::open("blockchain.json") {
            let mut data = String::new();
            if file.read_to_string(&mut data).is_ok() {
                if let Ok(blocks) = serde_json::from_str::<Vec<Block>>(&data) {
                    blockchain.blocks = blocks;
                }
            }
        }

        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        let reward_tx = Transaction {
            sender: "Network".to_string(),
            recipient: miner_address.to_string(),
            amount: 1.0,
            signature: None,
        };
        self.pending_transactions.push(reward_tx);

        let last_hash = self.get_latest_hash();
        let new_block = Block::new(self.pending_transactions.clone(), last_hash, self.difficulty);
        self.blocks.push(new_block);

        self.pending_transactions.clear();
        self.save_to_disk();
    }

    pub fn get_latest_hash(&self) -> String {
        self.blocks
            .last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| String::from("0"))
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            if current.hash != current.calculate_hash() {
                return false;
            }
        }
        true
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;

        for block in &self.blocks {
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

    pub fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.blocks) {
            if let Ok(mut file) = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("blockchain.json")
            {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }
}