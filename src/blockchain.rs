use crate::transaction::Transaction;
use crate::wallet::Wallet;
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub hash: String,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u64,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
    pub total_supply: f64,
}

impl Blockchain {
    pub fn new(wallet: &Wallet) -> Self {
        let mut bc = Blockchain {
            chain: Vec::new(),
            difficulty: 2,
            pending_transactions: Vec::new(),
            mining_reward: 50.0,
            total_supply: 21_000_000.0,
        };

        let genesis_transaction = Transaction::new(
            "QuantumChain".to_string(),
            wallet.get_address(),
            1_250_000.0,
            None,
        );

        let genesis_block = bc.create_block(vec![genesis_transaction], "0".to_string());
        bc.chain.push(genesis_block);
        bc
    }

    fn create_block(&self, transactions: Vec<Transaction>, previous_hash: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let mut nonce = 0;
        let mut hash = String::new();

        loop {
            let block_contents = format!("{:?}{:?}{}{}", transactions, timestamp, previous_hash, nonce);
            let mut hasher = Sha256::new();
            hasher.update(block_contents);
            let result = hasher.finalize();
            hash = hex::encode(&result);

            if &hash[..self.difficulty as usize] == "0".repeat(self.difficulty as usize) {
                break;
            }

            nonce += 1;
        }

        Block {
            index: self.chain.len() as u64,
            timestamp,
            previous_hash,
            hash,
            transactions,
            nonce,
        }
    }

    pub fn mine_pending_transactions(&mut self, miner_address: String) -> Block {
        let reward_tx = Transaction::new(
            "QuantumChain".to_string(),
            miner_address.clone(),
            self.mining_reward,
            None,
        );

        self.pending_transactions.push(reward_tx);

        let block = self.create_block(self.pending_transactions.clone(), self.latest_hash());
        self.chain.push(block.clone());
        self.pending_transactions.clear();

        block
    }

    pub fn latest_hash(&self) -> String {
        self.chain.last().unwrap().hash.clone()
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    pub fn save_to_file(&self) {
        let encoded = serde_json::to_string_pretty(self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("blockchain.json")
            .unwrap();
        file.write_all(encoded.as_bytes()).unwrap();
    }

    pub fn load_from_file() -> Option<Blockchain> {
        if let Ok(mut file) = File::open("blockchain.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents).ok()?;
            serde_json::from_str(&contents).ok()
        } else {
            None
        }
    }
}