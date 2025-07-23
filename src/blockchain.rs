use crate::wallet::Wallet;
use crate::revstop::is_revstop_active;

use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const BLOCKCHAIN_FILE: &str = "blockchain.json";
const INITIAL_REWARD: u64 = 50;
const HALVING_INTERVAL_SECS: u64 = 60 * 60 * 24 * 365 * 2; // every 2 years
const MAX_SUPPLY: u64 = 22_000_000;
const DIFFICULTY_PREFIX: &str = "0000";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub signature: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub total_mined: u64,
}

impl Blockchain {
    pub fn new(recipient_wallet: &str) -> Self {
        let genesis_transaction = Transaction {
            sender: "GENESIS".to_string(),
            recipient: recipient_wallet.to_string(),
            amount: 1_250_000,
            signature: None,
        };

        let mut block = Block {
            index: 0,
            timestamp: current_timestamp(),
            transactions: vec![genesis_transaction],
            previous_hash: "0".to_string(),
            nonce: 0,
            hash: String::new(),
        };

        block.hash = Blockchain::calculate_hash(&block);

        Blockchain {
            blocks: vec![block],
            total_mined: 1_250_000,
        }
    }

    pub fn load_from_file() -> Option<Self> {
        if let Ok(mut file) = File::open(BLOCKCHAIN_FILE) {
            let mut data = String::new();
            if file.read_to_string(&mut data).is_ok() {
                if let Ok(chain) = serde_json::from_str::<Blockchain>(&data) {
                    return Some(chain);
                }
            }
        }
        None
    }

    pub fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            let _ = fs::write(BLOCKCHAIN_FILE, json);
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if is_revstop_active(&tx.sender) {
            println!("❌ Transaction blocked: sender is RevStop locked.");
            return false;
        }

        // Optionally verify signature here
        self.pending_transactions().push(tx);
        true
    }

    fn pending_transactions(&mut self) -> &mut Vec<Transaction> {
        // All transactions before mining get added to the latest block
        &mut self.blocks.last_mut().unwrap().transactions
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        let previous_block = self.blocks.last().unwrap();
        let index = previous_block.index + 1;
        let timestamp = current_timestamp();
        let previous_hash = previous_block.hash.clone();
        let transactions = self.blocks.last().unwrap().transactions.clone();

        let reward = self.calculate_block_reward(timestamp);
        if self.total_mined + reward > MAX_SUPPLY {
            println!("⚠️ Max supply reached. No new rewards will be issued.");
            return;
        }

        let reward_tx = Transaction {
            sender: "REWARD".to_string(),
            recipient: miner_address.to_string(),
            amount: reward,
            signature: None,
        };

        let mut block = Block {
            index,
            timestamp,
            transactions: vec![reward_tx],
            previous_hash,
            nonce: 0,
            hash: String::new(),
        };

        // Mining loop
        while !Blockchain::calculate_hash(&block).starts_with(DIFFICULTY_PREFIX) {
            block.nonce += 1;
        }

        block.hash = Blockchain::calculate_hash(&block);
        self.total_mined += reward;
        self.blocks.push(block);
        println!("✅ Block mined and added. Reward: {} QTC", reward);
    }

    fn calculate_block_reward(&self, current_time: u64) -> u64 {
        let genesis_time = self.blocks.first().unwrap().timestamp;
        let years_elapsed = (current_time - genesis_time) / HALVING_INTERVAL_SECS;
        INITIAL_REWARD >> years_elapsed // divide by 2^years_elapsed
    }

    fn calculate_hash(block: &Block) -> String {
        let input = format!(
            "{}{}{:?}{}{}",
            block.index,
            block.timestamp,
            block.transactions,
            block.previous_hash,
            block.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            if !Blockchain::calculate_hash(current).starts_with(DIFFICULTY_PREFIX) {
                return false;
            }
        }
        true
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}