use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

use crate::transaction::{Block, Transaction};

const DIFFICULTY_PREFIX: &str = "0000";
const CHAIN_FILE: &str = "blockchain.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blocks: vec![],
            pending_transactions: vec![],
        };
        blockchain.create_genesis_block();
        blockchain
    }

    pub fn load_or_create() -> Self {
        if let Ok(mut file) = File::open(CHAIN_FILE) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap_or_else(|_| Blockchain::new())
        } else {
            Blockchain::new()
        }
    }

    fn create_genesis_block(&mut self) {
        let tx = Transaction::new(
            "genesis".to_string(),
            "tNzCy5NT+GORGIA+JCVIGAJUIBM...QNSATLVTHNBWXMZA783YP/ALNCM2GEAO1TZ==".to_string(),
            1_250_000,
            None,
        );

        let genesis_block = Block {
            index: 0,
            timestamp: current_timestamp(),
            transactions: vec![tx],
            previous_hash: String::from("0"),
            hash: String::new(),
            nonce: 0,
        };

        self.blocks.push(genesis_block);
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        self.pending_transactions.push(tx);
        true
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        let reward_tx = Transaction::new(
            "network".to_string(),
            miner_address.to_string(),
            50,
            None,
        );
        self.pending_transactions.push(reward_tx);

        let last_block = self.blocks.last().unwrap();
        let previous_hash = last_block.hash.clone();

        let mut block = Block {
            index: self.blocks.len() as u64,
            timestamp: current_timestamp(),
            transactions: self.pending_transactions.clone(),
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };

        self.proof_of_work(&mut block);
        block.hash = self.calculate_hash(&block);
        self.blocks.push(block);
        self.pending_transactions.clear();

        self.save_to_disk();
    }

    fn proof_of_work(&self, block: &mut Block) {
        loop {
            let hash = self.calculate_hash(block);
            if hash.starts_with(DIFFICULTY_PREFIX) {
                break;
            } else {
                block.nonce += 1;
            }
        }
    }

    fn calculate_hash(&self, block: &Block) -> String {
        let block_data = format!(
            "{}{}{:?}{}{}",
            block.index,
            block.timestamp,
            block.transactions,
            block.previous_hash,
            block.nonce
        );
        let digest = ring::digest::digest(&ring::digest::SHA256, block_data.as_bytes());
        hex::encode(digest)
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

    fn save_to_disk(&self) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        fs::write(CHAIN_FILE, json).unwrap();
    }
}

fn current_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}