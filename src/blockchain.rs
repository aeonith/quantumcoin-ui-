use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: usize,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            blocks: vec![],
            pending_transactions: vec![],
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
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

    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        let reward_tx = Transaction::new(
            "SYSTEM".to_string(),
            miner_address.clone(),
            50, // Set your mining reward here
            None,
        );

        self.pending_transactions.push(reward_tx);

        let new_block = Block {
            index: self.blocks.len(),
            transactions: self.pending_transactions.clone(),
            previous_hash: self.blocks.last().map_or("0".to_string(), |b| b.hash.clone()),
            hash: "placeholder".to_string(), // Replace with real hash logic
            nonce: 0,
        };

        self.blocks.push(new_block);
        self.pending_transactions.clear();
    }

    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        let mut all = Vec::new();
        for block in &self.blocks {
            for tx in &block.transactions {
                all.push(tx.clone());
            }
        }
        all
    }

    pub fn load_from_file(filename: &str) -> Option<Self> {
        let data = std::fs::read_to_string(filename).ok()?;
        serde_json::from_str(&data).ok()
    }
use sha2::{Sha256, Digest};

impl Blockchain {
    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        if self.pending_transactions.is_empty() {
            println!("⛏️ No transactions to mine.");
            return;
        }

        let last_hash = self.blocks.last().map_or(String::from("0"), |b| b.hash.clone());

        // Reward for miner
        let reward_tx = Transaction::new(
            "SYSTEM".to_string(),
            miner_address,
            100,
            None,
        );
        self.pending_transactions.push(reward_tx);

        let new_block = Block {
            index: self.blocks.len(),
            transactions: self.pending_transactions.clone(),
            previous_hash: last_hash,
            nonce: 0,
            hash: String::new(),
        };

        let mined_block = Self::mine_block(new_block);
        self.blocks.push(mined_block);
        self.pending_transactions.clear();

        // Save chain after mining
        self.save_to_file("blockchain.json");
    }

    pub fn mine_block(mut block: Block) -> Block {
        loop {
            let guess = format!("{:?}{:?}{:?}", block.index, block.transactions, block.nonce);
            let mut hasher = Sha256::new();
            hasher.update(guess.as_bytes());
            let result = hasher.finalize();
            let hash = hex::encode(&result);

            if &hash[..4] == "0000" {
                block.hash = hash;
                return block;
            } else {
                block.nonce += 1;
            }
        }
    }

    pub fn save_to_file(&self, filename: &str) {
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            std::fs::write(filename, json).expect("Failed to write blockchain file");
        }
    }

    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        self.blocks
            .iter()
            .flat_map(|block| block.transactions.clone())
            .collect()
    }
}