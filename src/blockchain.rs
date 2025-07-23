use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::transaction::Transaction;

const CHAIN_FILE: &str = "blockchain.json";
const DIFFICULTY_INTERVAL: usize = 10;     // adjust every 10 blocks
const TARGET_BLOCK_TIME: u64 = 60;         // target 60s per block
const INITIAL_DIFFICULTY: usize = 4;
const INITIAL_REWARD: u64 = 50;
const GENESIS_REWARD: u64 = 1_250_000;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: usize,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub reward: u64,
}

impl Block {
    pub fn new(
        index: usize,
        timestamp: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        nonce: u64,
        reward: u64,
    ) -> Self {
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce,
            reward,
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{:?}{}{}{}",
            self.index, self.timestamp, self.transactions,
            self.previous_hash, self.nonce, self.reward
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    pub reward: u64,
}

impl Blockchain {
    pub fn load_or_create() -> Self {
        // Try load
        if let Ok(mut f) = File::open(CHAIN_FILE) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(chain) = serde_json::from_str(&s) {
                    return chain;
                }
            }
        }
        // Else new
        let mut bc = Blockchain {
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty: INITIAL_DIFFICULTY,
            reward: INITIAL_REWARD,
        };
        bc.create_genesis_block();
        bc
    }

    pub fn save_to_file(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(CHAIN_FILE, json);
        }
    }

    fn create_genesis_block(&mut self) {
        let timestamp = current_timestamp();
        let tx = Transaction::new(
            "0".into(),
            "GENESIS_ADDRESS".into(),
            GENESIS_REWARD,
            None,
            timestamp,
        );
        let mut genesis = Block::new(0, timestamp, vec![tx], "0".into(), 0, 0);
        genesis.mine(self.difficulty);
        self.blocks.push(genesis);
        self.save_to_file();
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
        self.save_to_file();
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        // reward tx
        let timestamp = current_timestamp();
        let reward_tx = Transaction::new(
            "0".into(),
            miner_address.into(),
            self.reward,
            None,
            timestamp,
        );
        self.pending_transactions.push(reward_tx);

        // create new block
        let last = self.blocks.last().unwrap();
        let mut block = Block::new(
            self.blocks.len(),
            current_timestamp(),
            self.pending_transactions.clone(),
            last.hash.clone(),
            0,
            self.reward,
        );
        block.mine(self.difficulty);
        self.blocks.push(block);
        self.pending_transactions.clear();

        // adjust difficulty
        if self.blocks.len() % DIFFICULTY_INTERVAL == 0 {
            self.adjust_difficulty();
        }

        self.save_to_file();
    }

    fn adjust_difficulty(&mut self) {
        let len = self.blocks.len();
        let last = &self.blocks[len - 1];
        let prev = &self.blocks[len - 1 - DIFFICULTY_INTERVAL];
        let actual = last.timestamp - prev.timestamp;
        let expected = (DIFFICULTY_INTERVAL as u64) * TARGET_BLOCK_TIME;

        if actual < expected / 2 {
            self.difficulty += 1;
        } else if actual > expected * 2 && self.difficulty > 1 {
            self.difficulty -= 1;
        }
        println!("🔧 Difficulty now {}", self.difficulty);
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut bal = 0;
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.recipient == address {
                    bal = bal.saturating_add(tx.amount);
                }
                if tx.sender == address {
                    bal = bal.saturating_sub(tx.amount);
                }
            }
        }
        bal
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let cur = &self.blocks[i];
            let prev = &self.blocks[i - 1];
            if cur.hash != cur.calculate_hash() || cur.previous_hash != prev.hash {
                return false;
            }
        }
        true
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}