// src/blockchain.rs

use crate::block::Block;
use crate::transaction::Transaction;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

const BLOCKCHAIN_FILE: &str = "blockchain.json";
const GENESIS_PREMINE_ADDRESS: &str = "tNzCy5NT+GORGIA+JCVIGAJUIBM...QNSATLVTHNBWXMZA783YP/ALNCM2GEAO1TZ=="; // ← your base64 wallet address

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub block_reward: f64,
}

impl Blockchain {
    pub fn new() -> Self {
        if Path::new(BLOCKCHAIN_FILE).exists() {
            let mut file = File::open(BLOCKCHAIN_FILE).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap()
        } else {
            let mut blockchain = Blockchain {
                chain: vec![],
                difficulty: 5,
                block_reward: 50.0,
            };
            blockchain.create_genesis_block();
            blockchain
        }
    }

    fn create_genesis_block(&mut self) {
        let genesis_tx = Transaction {
            sender: "GENESIS".to_string(),
            recipient: GENESIS_PREMINE_ADDRESS.to_string(),
            amount: 1_250_000.0,
            signature: None,
        };

        let genesis_block = Block::new(0, vec![genesis_tx], "0".to_string(), self.difficulty);
        self.chain.push(genesis_block);
        self.save_to_disk();
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let last_block = self.chain.last().unwrap();
        let mut new_block = Block::new(
            last_block.index + 1,
            transactions,
            last_block.hash.clone(),
            self.difficulty,
        );
        new_block.mine_block();
        self.chain.push(new_block);

        if self.chain.len() % 1051200 == 0 && self.block_reward > 0.01 {
            self.block_reward /= 2.0;
        }

        self.save_to_disk();
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.calculate_hash() {
                return false;
            }

            if current.previous_hash != previous.hash {
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

    pub fn save_to_disk(&self) {
        let serialized = serde_json::to_string_pretty(self).unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(BLOCKCHAIN_FILE)
            .unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }
}

impl serde::Serialize for Blockchain {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let chain = &self.chain;
        let difficulty = self.difficulty;
        let block_reward = self.block_reward;
        let mut state = serializer.serialize_struct("Blockchain", 3)?;
        state.serialize_field("chain", chain)?;
        state.serialize_field("difficulty", &difficulty)?;
        state.serialize_field("block_reward", &block_reward)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Blockchain {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct BlockchainData {
            chain: Vec<Block>,
            difficulty: usize,
            block_reward: f64,
        }

        let data = BlockchainData::deserialize(deserializer)?;
        Ok(Blockchain {
            chain: data.chain,
            difficulty: data.difficulty,
            block_reward: data.block_reward,
        })
    }
}