use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::transaction::Transaction;

const CHAIN_FILE: &str = "blockchain.json";
const DIFFICULTY_INTERVAL: usize = 10;    // adjust every 10 blocks
const TARGET_TIME: u64 = 60;             // target 60s per block
const INITIAL_DIFFICULTY: usize = 4;
const INITIAL_REWARD: u64 = 50;
const GENESIS_REWARD: u64 = 1_250_000;

#[derive(Serialize, Deserialize, Clone)]
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
        reward: u64,
    ) -> Self {
        let mut blk = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            reward,
        };
        blk.hash = blk.calculate_hash();
        blk
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending: Vec<Transaction>,
    pub difficulty: usize,
    pub reward: u64,
}

impl Blockchain {
    pub fn load_or_create() -> Self {
        if let Ok(mut f) = File::open(CHAIN_FILE) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(chain) = serde_json::from_str(&s) {
                    return chain;
                }
            }
        }
        let mut bc = Blockchain {
            blocks: Vec::new(),
            pending: Vec::new(),
            difficulty: INITIAL_DIFFICULTY,
            reward: INITIAL_REWARD,
        };
        bc.create_genesis();
        bc
    }

    fn create_genesis(&mut self) {
        let ts = now();
        let tx = Transaction::new("0".into(), "GENESIS".into(), GENESIS_REWARD, ts, None);
        let mut g = Block::new(0, ts, vec![tx], "0".into(), 0);
        g.mine(self.difficulty);
        self.blocks.push(g);
        self.save();
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
        self.save();
    }

    pub fn mine_pending(&mut self, miner: &str) {
        let ts = now();
        let reward_tx = Transaction::new("0".into(), miner.into(), self.reward, ts, None);
        self.pending.push(reward_tx);

        let last = self.blocks.last().unwrap();
        let mut blk = Block::new(
            self.blocks.len(),
            now(),
            self.pending.clone(),
            last.hash.clone(),
            self.reward,
        );
        blk.mine(self.difficulty);
        self.blocks.push(blk);
        self.pending.clear();

        if self.blocks.len() % DIFFICULTY_INTERVAL == 0 {
            self.adjust_difficulty();
        }
        self.save();
    }

    fn adjust_difficulty(&mut self) {
        let len = self.blocks.len();
        let last = &self.blocks[len - 1];
        let prev = &self.blocks[len - 1 - DIFFICULTY_INTERVAL];
        let actual = last.timestamp - prev.timestamp;
        let expected = (DIFFICULTY_INTERVAL as u64) * TARGET_TIME;
        if actual < expected / 2 {
            self.difficulty += 1;
        } else if actual > expected * 2 && self.difficulty > 1 {
            self.difficulty -= 1;
        }
        println!("🔧 Difficulty adjusted to {}", self.difficulty);
    }

    pub fn get_balance(&self, addr: &str) -> u64 {
        let mut bal: u64 = 0;
        for blk in &self.blocks {
            for tx in &blk.transactions {
                if tx.recipient == addr {
                    bal = bal.saturating_add(tx.amount);
                }
                if tx.sender == addr {
                    bal = bal.saturating_sub(tx.amount);
                }
            }
        }
        bal
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let c = &self.blocks[i];
            let p = &self.blocks[i - 1];
            if c.hash != c.calculate_hash() || c.previous_hash != p.hash {
                return false;
            }
        }
        true
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(CHAIN_FILE, json);
        }
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}