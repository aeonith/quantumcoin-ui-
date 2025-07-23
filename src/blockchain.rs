use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: u64,
}

impl Blockchain {
    pub fn load_or_create() -> Self {
        if let Ok(mut file) = File::open("blockchain.json") {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            let mut blockchain = Blockchain {
                chain: vec![],
                pending_transactions: vec![],
                difficulty: 3,
            };
            blockchain.create_genesis_block();
            blockchain.save_to_file();
            blockchain
        }
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block {
            index: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            transactions: vec![],
            previous_hash: String::from("0"),
            hash: String::new(),
            nonce: 0,
        };
        self.chain.push(genesis_block);
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    pub fn mine_pending_transactions(&mut self, miner: &str) -> bool {
        let block = Block {
            index: self.chain.len() as u64,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            transactions: self.pending_transactions.clone(),
            previous_hash: self.chain.last().unwrap().hash.clone(),
            hash: String::new(),
            nonce: 0,
        };

        let hash = self.proof_of_work(&block);
        let mut mined_block = block.clone();
        mined_block.hash = hash;

        self.chain.push(mined_block);
        self.pending_transactions.clear();

        self.save_to_file();
        true
    }

    fn proof_of_work(&self, block: &Block) -> String {
        let mut nonce = 0;
        loop {
            let hash = self.calculate_hash(block, nonce);
            if &hash[..self.difficulty as usize] == "0".repeat(self.difficulty as usize) {
                return hash;
            }
            nonce += 1;
        }
    }

    fn calculate_hash(&self, block: &Block, nonce: u64) -> String {
        let data = format!(
            "{}{}{:?}{}{}",
            block.index,
            block.timestamp,
            block.transactions,
            block.previous_hash,
            nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn save_to_file(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        let _ = create_dir_all(".");
        let mut file = File::create("blockchain.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}