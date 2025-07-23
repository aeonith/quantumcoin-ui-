use crate::transaction::Transaction;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u64,
    pub reward: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending: Vec<Transaction>,
    pub total_mined: u64,
}

impl Blockchain {
    pub fn load_or_create() -> Self {
        if let Ok(mut file) = File::open("blockchain.json") {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            let genesis = Block {
                index: 0,
                timestamp: Utc::now().timestamp(),
                transactions: vec![Transaction {
                    sender: "GENESIS".to_string(),
                    recipient: "tNzCy5MT+GQRGlA+JCVIG8juIbmR0MhMSvCP7W0BauzccIB+UKuWBnyOl+nDv91JP2bTkOY30d+tBrlcYZ4wnbELEaNeue4MsLeBATOt0u/z...".to_string(),
                    amount: 1_250_000,
                    signature: None,
                }],
                prev_hash: String::from("0"),
                hash: String::new(),
                nonce: 0,
                difficulty: 4,
                reward: 50,
            };

            let mut bc = Blockchain {
                blocks: vec![],
                pending: vec![],
                total_mined: 1_250_000,
            };

            bc.add_block(genesis);
            bc
        }
    }

    pub fn add_block(&mut self, mut block: Block) {
        block.hash = Blockchain::calculate_hash(&block);
        self.blocks.push(block);
        self.pending.clear();
        self.save_to_file();
    }

    pub fn calculate_hash(block: &Block) -> String {
        let data = format!(
            "{}{}{:?}{}{}{}{}",
            block.index,
            block.timestamp,
            block.transactions,
            block.prev_hash,
            block.nonce,
            block.difficulty,
            block.reward
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn mine_pending_transactions(&mut self, wallet_addr: &str) {
        if self.total_mined >= 22_000_000 {
            println!("💥 Max supply reached.");
            return;
        }

        let difficulty = self.get_difficulty();
        let reward = self.get_reward();

        let mut block = Block {
            index: self.blocks.len() as u64,
            timestamp: Utc::now().timestamp(),
            transactions: self.pending.clone(),
            prev_hash: self.blocks.last().unwrap().hash.clone(),
            hash: String::new(),
            nonce: 0,
            difficulty,
            reward,
        };

        let reward_tx = Transaction {
            sender: "SYSTEM".to_string(),
            recipient: wallet_addr.to_string(),
            amount: reward,
            signature: None,
        };

        block.transactions.push(reward_tx.clone());

        while !Blockchain::calculate_hash(&block).starts_with(&"0".repeat(difficulty as usize)) {
            block.nonce += 1;
        }

        self.total_mined += reward;
        self.add_block(block);
        println!("⛏️ Block mined successfully. Reward: {} QTC", reward);
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0;
        for block in &self.blocks {
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

    pub fn get_difficulty(&self) -> u64 {
        let blocks = self.blocks.len() as u64;
        4 + (blocks / 1000) // Increase difficulty every 1000 blocks
    }

    pub fn get_reward(&self) -> u64 {
        let years = self.blocks.len() / (365 * 2); // Halve reward every 2 years
        let base = 50;
        base >> years.min(10) // Cap halving after 10 cycles
    }

    pub fn save_to_file(&self) {
        let data = serde_json::to_string(self).unwrap();
        fs::write("blockchain.json", data).unwrap();
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    pub fn get_last_n_transactions(&self, address: &str, n: usize) -> Vec<Transaction> {
        let mut txs = vec![];
        for block in self.blocks.iter().rev() {
            for tx in &block.transactions {
                if tx.recipient == address || tx.sender == address {
                    txs.push(tx.clone());
                }
                if txs.len() >= n {
                    return txs;
                }
            }
        }
        txs
    }
}