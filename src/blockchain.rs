use crate::transaction::Transaction;
use crate::block::Block;
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
    pub difficulty: usize,
    pub halving_interval: u64,
    pub start_time: u64,
}

impl Blockchain {
    pub fn new(genesis_recipient: &str) -> Self {
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            mining_reward: 50.0,
            difficulty: 4,
            halving_interval: 63_072_000, // 2 years
            start_time,
        };

        let genesis_tx = Transaction {
            sender: "GENESIS".into(),
            recipient: genesis_recipient.into(),
            amount: 1_250_000.0,
            signature: None,
        };

        let genesis_block = Block::new(0, vec![genesis_tx], "0".into(), blockchain.difficulty);
        blockchain.chain.push(genesis_block);
        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, wallet: &crate::wallet::Wallet) {
        let reward_tx = Transaction {
            sender: "REWARD".into(),
            recipient: wallet.get_address(),
            amount: self.mining_reward,
            signature: None,
        };

        self.pending_transactions.push(reward_tx);

        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            previous_hash,
            self.difficulty,
        );

        self.chain.push(new_block);
        self.pending_transactions.clear();
        self.update_halving();
    }

    pub fn update_halving(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let years_passed = (now - self.start_time) / self.halving_interval;
        self.mining_reward = 50.0 / 2f64.powf(years_passed as f64);
        if self.mining_reward < 0.0001 {
            self.mining_reward = 0.0001;
        }
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.calculate_hash() || current.previous_hash != previous.hash {
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
                } else if tx.sender == address {
                    balance -= tx.amount;
                }
            }
        }
        balance
    }

    pub fn save_to_file(&self, filename: &str) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(filename).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    pub fn load_from_file(filename: &str) -> Option<Self> {
        let mut file = OpenOptions::new().read(true).open(filename).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        serde_json::from_str(&contents).ok()
    }

    pub fn show_mining_progress(&self) {
        println!("⛏️ Total Blocks Mined: {}", self.chain.len());
        println!("📉 Current Reward: {:.6} QTC", self.mining_reward);
        println!("📦 Pending Transactions: {}", self.pending_transactions.len());
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