use crate::block::Block;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const GENESIS_REWARD: f64 = 1_250_000.0;
const HALVING_INTERVAL_YEARS: u64 = 2;
const BLOCKS_PER_YEAR: u64 = 52_560; // 1 block every 10 minutes
const BLOCKS_PER_HALVING: u64 = HALVING_INTERVAL_YEARS * BLOCKS_PER_YEAR;

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub mempool: Vec<Transaction>,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(genesis_address: String) -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            mempool: vec![],
            difficulty: 6, // Bitcoin-level
        };
        let genesis_tx = Transaction {
            sender: "GENESIS".to_string(),
            recipient: genesis_address,
            amount: GENESIS_REWARD,
            signature: None,
        };
        let genesis_block = Block::new(0, vec![genesis_tx], "0".to_string(), blockchain.difficulty);
        blockchain.chain.push(genesis_block);
        blockchain
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.mempool.push(tx);
    }

    pub fn mine_pending_transactions(&mut self, wallet: &Wallet) {
        if self.mempool.is_empty() {
            println!("📭 No transactions to mine.");
            return;
        }

        let reward = self.current_block_reward();
        let reward_tx = Transaction {
            sender: "MINING_REWARD".to_string(),
            recipient: wallet.get_address(),
            amount: reward,
            signature: None,
        };

        let mut txs = vec![reward_tx];
        txs.append(&mut self.mempool);

        let last_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(self.chain.len() as u64, txs, last_hash, self.difficulty);
        self.chain.push(new_block);
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];
            if current.previous_hash != previous.hash {
                return false;
            }
            if !current.is_valid() {
                return false;
            }
        }
        true
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
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

    pub fn current_block_reward(&self) -> f64 {
        let height = self.chain.len() as u64;
        let halvings = height / BLOCKS_PER_HALVING;
        let mut reward = 50.0;
        for _ in 0..halvings {
            reward /= 2.0;
        }
        reward.max(0.0001)
    }

    pub fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string(&self) {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("blockchain.json")
                .unwrap();
            file.write_all(json.as_bytes()).unwrap();
        }
    }

    pub fn load_from_disk() -> Option<Blockchain> {
        if let Ok(mut file) = File::open("blockchain.json") {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            if let Ok(bc) = serde_json::from_str(&data) {
                return Some(bc);
            }
        }
        None
    }

    pub fn show_mining_progress(&self) {
        let total = self.chain.len();
        let reward = self.current_block_reward();
        println!("📊 Height: {total}, Next Reward: {reward:.4} QTC");
    }
}