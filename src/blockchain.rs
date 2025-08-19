use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use blake3;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,  // Changed from f64 to u64 for better precision
    pub timestamp: DateTime<Utc>,
    pub signature: String,
    pub fee: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub merkle_root: String,
    pub difficulty: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: u64,
    pub difficulty: usize,
    pub balances: HashMap<String, u64>,
    pub total_supply: u64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            mining_reward: 1_000_000_000, // 10 QTC in satoshis
            difficulty: 4,
            balances: HashMap::new(),
            total_supply: 0,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now(),
            transactions: Vec::new(),
            previous_hash: "0".to_string(),
            hash: self.calculate_genesis_hash(),
            nonce: 0,
            merkle_root: "0".to_string(),
            difficulty: self.difficulty,
        };
        self.chain.push(genesis_block);
    }

    fn calculate_genesis_hash(&self) -> String {
        let data = "QuantumCoin Genesis Block";
        let hash = blake3::hash(data.as_bytes());
        hex::encode(hash.as_bytes())
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn create_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, mining_reward_address: String) -> Result<Block> {
        let reward_transaction = Transaction {
            id: format!("reward_{}", Utc::now().timestamp()),
            from: "".to_string(),
            to: mining_reward_address.clone(),
            amount: self.mining_reward,
            timestamp: Utc::now(),
            signature: "".to_string(),
            fee: 0,
        };

        self.pending_transactions.push(reward_transaction);

        let mut block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now(),
            transactions: self.pending_transactions.clone(),
            previous_hash: self.get_latest_block().hash.clone(),
            hash: "".to_string(),
            nonce: 0,
            merkle_root: self.calculate_merkle_root(&self.pending_transactions),
            difficulty: self.difficulty,
        };

        self.mine_block(&mut block);
        self.chain.push(block.clone());
        self.update_balances(&block);
        self.pending_transactions.clear();

        Ok(block)
    }

    fn mine_block(&self, block: &mut Block) {
        let target = "0".repeat(self.difficulty);
        
        while !block.hash.starts_with(&target) {
            block.nonce += 1;
            block.hash = self.calculate_hash(block);
        }

        println!("Block mined: {}", block.hash);
    }

    fn calculate_hash(&self, block: &Block) -> String {
        let data = format!(
            "{}{}{}{}{}{}{}",
            block.index,
            block.timestamp,
            serde_json::to_string(&block.transactions).unwrap(),
            block.previous_hash,
            block.nonce,
            block.merkle_root,
            block.difficulty
        );
        
        let hash = blake3::hash(data.as_bytes());
        hex::encode(hash.as_bytes())
    }

    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0".to_string();
        }

        let mut tx_hashes: Vec<String> = transactions
            .iter()
            .map(|tx| {
                let tx_data = format!("{}{}{}{}", tx.id, tx.from, tx.to, tx.amount);
                let hash = blake3::hash(tx_data.as_bytes());
                hex::encode(hash.as_bytes())
            })
            .collect();

        while tx_hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in tx_hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    format!("{}{}", chunk[0], chunk[0])
                };
                let hash = blake3::hash(combined.as_bytes());
                next_level.push(hex::encode(hash.as_bytes()));
            }
            
            tx_hashes = next_level;
        }

        tx_hashes.into_iter().next().unwrap_or_else(|| "0".to_string())
    }

    fn update_balances(&mut self, block: &Block) {
        for transaction in &block.transactions {
            if !transaction.from.is_empty() {
                let from_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                if *from_balance >= transaction.amount + transaction.fee {
                    self.balances.insert(
                        transaction.from.clone(), 
                        from_balance - transaction.amount - transaction.fee
                    );
                }
            }
            
            let to_balance = self.balances.get(&transaction.to).unwrap_or(&0);
            self.balances.insert(transaction.to.clone(), to_balance + transaction.amount);
        }
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        *self.balances.get(address).unwrap_or(&0)
    }

    pub fn get_current_mining_reward(&self) -> u64 {
        // Halving every 210,000 blocks like Bitcoin
        let halvings = self.chain.len() / 210_000;
        self.mining_reward >> halvings
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != self.calculate_hash(current_block) {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }

    pub fn adjust_difficulty(&mut self) {
        const TARGET_BLOCK_TIME: u64 = 600; // 10 minutes in seconds
        const DIFFICULTY_ADJUSTMENT_INTERVAL: usize = 2016; // Every 2 weeks

        if self.chain.len() % DIFFICULTY_ADJUSTMENT_INTERVAL == 0 && self.chain.len() > DIFFICULTY_ADJUSTMENT_INTERVAL {
            let recent_block = &self.chain[self.chain.len() - 1];
            let old_block = &self.chain[self.chain.len() - DIFFICULTY_ADJUSTMENT_INTERVAL];
            
            let time_taken = recent_block.timestamp.timestamp() - old_block.timestamp.timestamp();
            let expected_time = (DIFFICULTY_ADJUSTMENT_INTERVAL as u64) * TARGET_BLOCK_TIME;
            
            if time_taken < expected_time / 2 {
                self.difficulty += 1;
            } else if time_taken > expected_time * 2 {
                if self.difficulty > 1 {
                    self.difficulty -= 1;
                }
            }
        }
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}
