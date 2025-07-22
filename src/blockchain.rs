use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::block::Block;
use crate::transaction::Transaction;
use crate::wallet::SecureWallet;

use serde::{Serialize, Deserialize};

const BLOCKCHAIN_FILE: &str = "data/blockchain.json";

const INITIAL_REWARD: u64 = 50 * 100; // scaled by 100 for decimals
const MAX_SUPPLY: u64 = 22_000_000 * 100; // total supply scaled
const BLOCKS_PER_YEAR: u64 = 365 * 144; // approx 144 blocks/day

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    pub total_supply: u64,
}

impl Blockchain {
    pub fn new(genesis_wallet: &SecureWallet) -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            difficulty: 4,
            total_supply: 0,
        };

        let genesis_tx = Transaction::new_genesis(
            "QuantumCoin_Genesis".to_string(),
            genesis_wallet.get_address(),
            1_250_000 * 100, // 1.25 million QTC
        );

        let genesis_block = Block::new(
            0,
            vec![genesis_tx.clone()],
            String::from("0"),
            blockchain.difficulty,
        );

        blockchain.total_supply += 1_250_000 * 100;
        blockchain.chain.push(genesis_block);
        blockchain
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        if self.total_supply >= MAX_SUPPLY {
            println!("💡 Maximum supply reached. No new coins will be created.");
            return;
        }

        let reward = self.current_reward();
        let reward_tx = Transaction::new_reward(miner_address.to_string(), reward);
        let mut transactions = self.pending_transactions.clone();
        transactions.push(reward_tx);

        let last_hash = self.latest_block().hash.clone();
        let new_block = Block::new(
            self.chain.len() as u64,
            transactions,
            last_hash,
            self.difficulty,
        );

        self.total_supply += reward;
        self.chain.push(new_block);
        self.pending_transactions.clear();
    }

    fn current_reward(&self) -> u64 {
        let blocks_mined = self.chain.len() as u64;
        let years_since_genesis = blocks_mined / BLOCKS_PER_YEAR;
        let halvings = years_since_genesis / 2;
        INITIAL_REWARD >> halvings
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().expect("Chain should have at least one block.")
    }

    pub fn save_to_disk(&self) {
        if let Ok(serialized) = serde_json::to_string_pretty(self) {
            fs::create_dir_all("data").unwrap();
            let mut file = File::create(BLOCKCHAIN_FILE).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
        }
    }

    pub fn load_from_disk() -> Option<Self> {
        if Path::new(BLOCKCHAIN_FILE).exists() {
            let mut file = File::open(BLOCKCHAIN_FILE).unwrap();
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance: i64 = 0;

        for block in &self.chain {
            for tx in &block.transactions {
                if tx.recipient == address {
                    balance += tx.amount as i64;
                }
                if tx.sender == address {
                    balance -= tx.amount as i64;
                }
            }
        }

        if balance < 0 { 0 } else { balance as u64 }
    }

    pub fn is_valid_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let prev = &self.chain[i - 1];
            let curr = &self.chain[i];

            if curr.previous_hash != prev.hash {
                return false;
            }

            if curr.hash != curr.calculate_hash() {
                return false;
            }
        }
        true
    }

    pub fn get_last_n_transactions(&self, n: usize) -> Vec<Transaction> {
        let mut txs = vec![];
        for block in self.chain.iter().rev() {
            for tx in block.transactions.iter().rev() {
                txs.push(tx.clone());
                if txs.len() == n {
                    return txs;
                }
            }
        }
        txs
    }
}