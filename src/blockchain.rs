use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest}; // ✅ Make sure this is correct
use base64::{encode};
use crate::wallet::Wallet;

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from_addr: String,
    pub to_addr: String,
    pub amount: f64,
    pub signature: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u64,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
}

impl Blockchain {
    pub fn new(wallet_address: &str) -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            difficulty: 3,
            pending_transactions: vec![],
            mining_reward: 50.0,
        };

        let genesis_transaction = Transaction {
            from_addr: String::from("GENESIS"),
            to_addr: wallet_address.to_string(),
            amount: 1_250_000.0,
            signature: None,
        };

        let genesis_block = Block {
            timestamp: Blockchain::now(),
            transactions: vec![genesis_transaction],
            previous_hash: String::from("0"),
            nonce: 0,
            hash: String::new(),
        };

        let mined_genesis = blockchain.mine_block(genesis_block);
        blockchain.chain.push(mined_genesis);
        blockchain
    }

    pub fn now() -> u128 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
    }

    pub fn create_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        let reward_tx = Transaction {
            from_addr: String::from("System"),
            to_addr: miner_address.to_string(),
            amount: self.mining_reward,
            signature: None,
        };

        self.pending_transactions.push(reward_tx);

        let last_hash = self.chain.last().unwrap().hash.clone();
        let block = Block {
            timestamp: Blockchain::now(),
            transactions: self.pending_transactions.clone(),
            previous_hash: last_hash,
            nonce: 0,
            hash: String::new(),
        };

        let mined = self.mine_block(block);
        self.chain.push(mined);
        self.pending_transactions.clear();
    }

    pub fn mine_block(&self, mut block: Block) -> Block {
        while !block.hash.starts_with(&"0".repeat(self.difficulty as usize)) {
            block.nonce += 1;
            block.hash = Blockchain::calculate_hash(&block);
        }
        block
    }

    pub fn calculate_hash(block: &Block) -> String {
        let data = format!(
            "{}{:?}{}{}",
            block.timestamp,
            block.transactions,
            block.previous_hash,
            block.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from_addr == address {
                    balance -= tx.amount;
                }
                if tx.to_addr == address {
                    balance += tx.amount;
                }
            }
        }
        balance
    }

    pub fn save_to_file(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write("blockchain.json", json).unwrap();
    }

    pub fn load_from_file() -> Option<Self> {
        if let Ok(mut file) = File::open("blockchain.json") {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            Some(serde_json::from_str(&data).unwrap())
        } else {
            None
        }
    }
}