use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::collections::VecDeque;
use crate::wallet::Wallet;

const DIFFICULTY_PREFIX: &str = "0000";
const MAX_SUPPLY: u64 = 22_000_000;
const HALVING_INTERVAL_SECS: u64 = 2 * 365 * 24 * 60 * 60; // 2 years
const INITIAL_REWARD: u64 = 50;
const CHAIN_FILE: &str = "blockchain.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub mempool: VecDeque<Transaction>,
    pub total_supply: u64,
    pub genesis_time: u64,
}

impl Blockchain {
    pub fn load_or_create() -> Self {
        if let Ok(mut file) = File::open(CHAIN_FILE) {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            Blockchain::new()
        }
    }

    fn new() -> Self {
        let mut chain = Blockchain {
            chain: vec![],
            mempool: VecDeque::new(),
            total_supply: 0,
            genesis_time: current_time(),
        };

        let wallet_address = "tNzCy5NT+GORGIA+JCVIGAJUIBM...QNSATLVTHNBWXMZA783YP/ALNCM2GEAO1TZ==".to_string();
        let dev_fund = "DEVFUNDPUBLICKEYEXAMPLESTRING==".to_string();

        let genesis_tx = Transaction {
            sender: "GENESIS".into(),
            recipient: wallet_address.clone(),
            amount: 1_000_000,
            signature: None,
        };

        let dev_tx = Transaction {
            sender: "GENESIS".into(),
            recipient: dev_fund,
            amount: 250_000,
            signature: None,
        };

        let genesis_block = Block {
            index: 0,
            timestamp: chain.genesis_time,
            transactions: vec![genesis_tx.clone(), dev_tx.clone()],
            previous_hash: "0".repeat(64),
            nonce: 0,
            hash: "GENESIS_HASH".to_string(),
        };

        chain.chain.push(genesis_block);
        chain.total_supply = 1_250_000;
        chain
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.mempool.push_back(tx);
    }

    pub fn mine_pending_transactions(&mut self, wallet: &Wallet) {
        if self.total_supply >= MAX_SUPPLY || self.mempool.is_empty() {
            return;
        }

        let reward = self.calculate_block_reward();
        let reward_tx = Transaction {
            sender: "COINBASE".to_string(),
            recipient: wallet.address(),
            amount: reward,
            signature: None,
        };

        let mut transactions: Vec<Transaction> = self.mempool.drain(..).collect();
        transactions.push(reward_tx);

        let last_block = self.chain.last().unwrap();
        let previous_hash = last_block.hash.clone();
        let timestamp = current_time();
        let index = last_block.index + 1;

        let mut nonce = 0;
        let hash;
        loop {
            let candidate = format!("{index}{timestamp}{:?}{previous_hash}{nonce}", transactions);
            let candidate_hash = sha256(&candidate);
            if candidate_hash.starts_with(DIFFICULTY_PREFIX) {
                hash = candidate_hash;
                break;
            }
            nonce += 1;
        }

        let block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            nonce,
            hash,
        };

        self.total_supply += reward;
        self.chain.push(block);
        self.save_to_file().unwrap();
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

    pub fn save_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(CHAIN_FILE)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn calculate_block_reward(&self) -> u64 {
        let elapsed = current_time() - self.genesis_time;
        let halvings = elapsed / HALVING_INTERVAL_SECS;
        let reward = INITIAL_REWARD >> halvings;
        reward.max(1)
    }
}

fn current_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn sha256(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}