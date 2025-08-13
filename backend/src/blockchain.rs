use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
    pub signature: String,
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
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
    pub difficulty: usize,
    pub balances: HashMap<String, f64>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            mining_reward: 10.0,
            difficulty: 4,
            balances: HashMap::new(),
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
            hash: "0".to_string(),
            nonce: 0,
            merkle_root: "0".to_string(),
        };
        self.chain.push(genesis_block);
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn create_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, mining_reward_address: String) -> Block {
        let reward_transaction = Transaction {
            id: format!("reward_{}", Utc::now().timestamp()),
            from: "".to_string(),
            to: mining_reward_address.clone(),
            amount: self.mining_reward,
            timestamp: Utc::now(),
            signature: "".to_string(),
        };

        self.pending_transactions.push(reward_transaction);

        let block = Block {
            index: self.chain.len() as u64,
            timestamp: Utc::now(),
            transactions: self.pending_transactions.clone(),
            previous_hash: self.get_latest_block().hash.clone(),
            hash: "".to_string(),
            nonce: 0,
            merkle_root: self.calculate_merkle_root(&self.pending_transactions),
        };

        let mined_block = self.mine_block(block);
        self.chain.push(mined_block.clone());
        self.update_balances(&mined_block);
        self.pending_transactions.clear();

        mined_block
    }

    fn mine_block(&self, mut block: Block) -> Block {
        let target = "0".repeat(self.difficulty);
        
        while !block.hash.starts_with(&target) {
            block.nonce += 1;
            block.hash = self.calculate_hash(&block);
        }

        println!("Block mined: {}", block.hash);
        block
    }

    fn calculate_hash(&self, block: &Block) -> String {
        let data = format!(
            "{}{}{}{}{}{}",
            block.index,
            block.timestamp,
            serde_json::to_string(&block.transactions).unwrap(),
            block.previous_hash,
            block.nonce,
            block.merkle_root
        );
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0".to_string();
        }

        let mut hasher = Sha256::new();
        for tx in transactions {
            hasher.update(tx.id.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    fn update_balances(&mut self, block: &Block) {
        for transaction in &block.transactions {
            if !transaction.from.is_empty() {
                let from_balance = self.balances.get(&transaction.from).unwrap_or(&0.0);
                self.balances.insert(transaction.from.clone(), from_balance - transaction.amount);
            }
            
            let to_balance = self.balances.get(&transaction.to).unwrap_or(&0.0);
            self.balances.insert(transaction.to.clone(), to_balance + transaction.amount);
        }
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
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
}
