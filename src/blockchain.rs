use crate::block::Block;
use crate::transaction::{Transaction, TransactionError};
use crate::merkle::MerkleTree;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, Duration};
use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Invalid block")]
    InvalidBlock,
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(#[from] TransactionError),
    #[error("Block not found")]
    BlockNotFound,
    #[error("Chain validation failed")]
    ChainValidationFailed,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Double spending detected")]
    DoubleSpending,
    #[error("Mining difficulty adjustment failed")]
    DifficultyAdjustmentFailed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DifficultyAdjustment {
    pub target_block_time: u64, // seconds
    pub adjustment_interval: u64, // blocks
    pub max_adjustment_factor: f64,
}

impl Default for DifficultyAdjustment {
    fn default() -> Self {
        Self {
            target_block_time: 600, // 10 minutes like Bitcoin
            adjustment_interval: 144, // ~1 day at 10min blocks
            max_adjustment_factor: 4.0, // Max 4x up/down per adjustment
        }
    }
}

#[derive(Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
    pub mining_reward: u64,
    pub utxo_set: HashMap<String, u64>, // address -> balance
    pub transaction_pool: HashMap<String, Transaction>, // tx_id -> transaction
    pub difficulty_config: DifficultyAdjustment,
    pub halving_interval: u64, // blocks until reward halves
    pub total_supply: u64,
    pub max_supply: u64,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            difficulty: 4,
            mining_reward: 50_000_000, // 50 QTC in satoshis
            utxo_set: HashMap::new(),
            transaction_pool: HashMap::new(),
            difficulty_config: DifficultyAdjustment::default(),
            halving_interval: 210_000, // Blocks until reward halves
            total_supply: 0,
            max_supply: 21_000_000_000_000, // 21M QTC in satoshis
        };

        // Load from disk first
        blockchain.load_from_disk("blockchain.json");
        
        // Create genesis block if chain is empty
        if blockchain.chain.is_empty() {
            blockchain.create_genesis_block();
        }
        
        // Rebuild UTXO set from chain
        blockchain.rebuild_utxo_set();

        blockchain
    }
    
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        // Validate block before adding
        self.validate_block(&block)?;
        
        // Check if we need to adjust difficulty
        if self.chain.len() > 0 && (self.chain.len() + 1) % self.difficulty_config.adjustment_interval as usize == 0 {
            self.adjust_difficulty()?;
        }
        
        // Add block to chain
        self.chain.push(block.clone());
        
        // Update UTXO set
        self.update_utxo_set(&block)?;
        
        // Update total supply
        self.total_supply += self.get_current_mining_reward();
        
        // Save to disk
        self.save_to_disk("blockchain.json");
        
        Ok(())
    }
    
    pub fn validate_block(&self, block: &Block) -> Result<(), BlockchainError> {
        // Check if block hash is valid
        if block.hash != block.calculate_hash() {
            return Err(BlockchainError::InvalidBlock);
        }
        
        // Check proof of work
        if !self.is_valid_proof_of_work(&block.hash) {
            return Err(BlockchainError::InvalidBlock);
        }
        
        // Check previous hash
        if let Some(last_block) = self.chain.last() {
            if block.previous_hash != last_block.hash {
                return Err(BlockchainError::InvalidBlock);
            }
            
            // Check timestamp (not too far in future or past)
            let time_diff = block.timestamp.signed_duration_since(last_block.timestamp);
            if time_diff.num_seconds() < -7200 || time_diff.num_seconds() > 7200 {
                return Err(BlockchainError::InvalidBlock);
            }
        } else if block.previous_hash != "0" {
            return Err(BlockchainError::InvalidBlock);
        }
        
        // Validate all transactions in the block
        for tx in &block.transactions {
            self.validate_transaction(tx)?;
        }
        
        // Check block reward
        let reward_tx = block.transactions.first()
            .ok_or(BlockchainError::InvalidBlock)?;
        
        if reward_tx.sender != "NETWORK" || reward_tx.amount != self.get_current_mining_reward() {
            return Err(BlockchainError::InvalidBlock);
        }
        
        Ok(())
    }
    
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), BlockchainError> {
        // Check transaction format
        if tx.amount == 0 {
            return Err(BlockchainError::InvalidTransaction(TransactionError::InvalidAmount));
        }
        
        if tx.fee < self.calculate_minimum_fee(tx) {
            return Err(BlockchainError::InvalidTransaction(TransactionError::FeeTooLow));
        }
        
        // Skip validation for coinbase transactions
        if tx.sender == "NETWORK" {
            return Ok(());
        }
        
        // Check if sender has enough balance
        let sender_balance = self.get_balance(&tx.sender);
        if sender_balance < tx.amount + tx.fee {
            return Err(BlockchainError::InsufficientBalance);
        }
        
        // Verify signature
        if let (Some(signature), Some(public_key)) = (&tx.signature, &tx.public_key) {
            let tx_data = format!("{}{}{}{}", tx.sender, tx.recipient, tx.amount, tx.nonce);
            if !self.verify_signature(&tx_data, signature, public_key) {
                return Err(BlockchainError::InvalidTransaction(TransactionError::InvalidSignature));
            }
        } else {
            return Err(BlockchainError::InvalidTransaction(TransactionError::InvalidSignature));
        }
        
        Ok(())
    }
    
    fn verify_signature(&self, data: &str, signature: &str, public_key: &str) -> bool {
        use pqcrypto_dilithium::dilithium2::{DetachedSignature, PublicKey};
        use pqcrypto_traits::sign::Verifier;
        
        if let (Ok(sig), Ok(pk)) = (
            DetachedSignature::from_bytes(&base64::decode(signature).unwrap_or_default()),
            PublicKey::from_bytes(&base64::decode(public_key).unwrap_or_default())
        ) {
            pk.verify(data.as_bytes(), &sig).is_ok()
        } else {
            false
        }
    }
    
    fn is_valid_proof_of_work(&self, hash: &str) -> bool {
        let target = "0".repeat(self.difficulty);
        hash.starts_with(&target)
    }
    
    fn adjust_difficulty(&mut self) -> Result<(), BlockchainError> {
        let current_height = self.chain.len();
        let adjustment_interval = self.difficulty_config.adjustment_interval as usize;
        
        if current_height < adjustment_interval {
            return Ok(());
        }
        
        let start_block = &self.chain[current_height - adjustment_interval];
        let end_block = &self.chain[current_height - 1];
        
        let time_taken = end_block.timestamp
            .signed_duration_since(start_block.timestamp)
            .num_seconds() as u64;
        
        let expected_time = self.difficulty_config.target_block_time * self.difficulty_config.adjustment_interval;
        
        let ratio = time_taken as f64 / expected_time as f64;
        
        if ratio > self.difficulty_config.max_adjustment_factor {
            // Too slow, decrease difficulty
            if self.difficulty > 1 {
                self.difficulty -= 1;
            }
        } else if ratio < 1.0 / self.difficulty_config.max_adjustment_factor {
            // Too fast, increase difficulty
            self.difficulty += 1;
        }
        
        Ok(())
    }
    
    fn update_utxo_set(&mut self, block: &Block) -> Result<(), BlockchainError> {
        for tx in &block.transactions {
            if tx.sender != "NETWORK" {
                // Subtract from sender
                let sender_balance = self.utxo_set.get(&tx.sender).copied().unwrap_or(0);
                if sender_balance < tx.amount + tx.fee {
                    return Err(BlockchainError::InsufficientBalance);
                }
                self.utxo_set.insert(tx.sender.clone(), sender_balance - tx.amount - tx.fee);
            }
            
            // Add to recipient
            let recipient_balance = self.utxo_set.get(&tx.recipient).copied().unwrap_or(0);
            self.utxo_set.insert(tx.recipient.clone(), recipient_balance + tx.amount);
        }
        Ok(())
    }
    
    pub fn get_balance(&self, address: &str) -> u64 {
        self.utxo_set.get(address).copied().unwrap_or(0)
    }
    
    pub fn get_current_mining_reward(&self) -> u64 {
        let halvings = self.chain.len() as u64 / self.halving_interval;
        self.mining_reward >> halvings
    }
    
    fn calculate_minimum_fee(&self, _tx: &Transaction) -> u64 {
        1000 // 0.00001 QTC minimum fee
    }
    
    pub fn calculate_total_work(&self) -> u64 {
        self.chain.iter().map(|_| 2_u64.pow(self.difficulty as u32)).sum()
    }
    
    pub fn get_latest_block_hash(&self) -> String {
        self.chain.last().map(|b| b.hash.clone()).unwrap_or_else(|| "0".to_string())
    }
    
    pub fn get_blocks_range(&self, start_hash: &str, end_hash: Option<&str>, limit: usize) -> Vec<Block> {
        let start_idx = if start_hash == "0" {
            0
        } else {
            self.chain.iter().position(|b| b.hash == start_hash).unwrap_or(0)
        };
        
        let end_idx = if let Some(end_hash) = end_hash {
            self.chain.iter().position(|b| b.hash == end_hash).unwrap_or(self.chain.len())
        } else {
            self.chain.len()
        };
        
        let actual_limit = std::cmp::min(limit, end_idx - start_idx);
        self.chain[start_idx..start_idx + actual_limit].to_vec()
    }
    
    fn rebuild_utxo_set(&mut self) {
        self.utxo_set.clear();
        for block in &self.chain {
            let _ = self.update_utxo_set(block);
        }
    }
    
    pub fn save_to_disk(&self, filename: &str) {
        if let Ok(serialized) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(filename, serialized);
        }
    }
    
    pub fn load_from_disk(&mut self, filename: &str) {
        if let Ok(content) = std::fs::read_to_string(filename) {
            if let Ok(blockchain) = serde_json::from_str::<Blockchain>(&content) {
                *self = blockchain;
            }
        }
    }

    pub fn create_genesis_block(&mut self) {
        use crate::transaction::Transaction;
        use chrono::Utc;
        
        let genesis_tx = Transaction {
            id: "genesis".to_string(),
            sender: "GENESIS".to_string(),
            recipient: "GENESIS".to_string(),
            amount: 0,
            fee: 0,
            timestamp: Utc::now(),
            signature: None,
            public_key: None,
            nonce: 0,
        };

        let genesis_block = Block {
            index: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            transactions: vec![genesis_tx],
            previous_hash: "0".to_string(),
            hash: "0".to_string(),
            nonce: 0,
        };

        self.chain.push(genesis_block);
        self.save_to_disk("blockchain.json");
    }

    pub fn create_block(&mut self, miner_address: &str) -> Result<Block, BlockchainError> {
        // Calculate current mining reward with halving
        let current_reward = self.calculate_mining_reward();
        
        // Create mining reward transaction
        let reward_tx = Transaction {
            id: format!("reward_{}", self.chain.len()),
            sender: "MINING_REWARD".to_string(),
            recipient: miner_address.to_string(),
            amount: current_reward,
            fee: 0,
            timestamp: chrono::Utc::now(),
            signature: None,
            public_key: None,
            nonce: 0,
        };

        // Validate all pending transactions
        let mut valid_transactions = Vec::new();
        let mut total_fees = 0;

        for tx in &self.pending_transactions {
            if let Ok(true) = self.validate_transaction_against_utxo(tx) {
                valid_transactions.push(tx.clone());
                total_fees += tx.fee;
            }
        }

        // Add reward transaction with fees
        let mut final_reward_tx = reward_tx;
        final_reward_tx.amount += total_fees;
        valid_transactions.insert(0, final_reward_tx);

        let last_hash = self.chain.last().map_or(String::from("0"), |b| b.hash.clone());
        let index = self.chain.len() as u64;

        let mut new_block = Block {
            index,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            transactions: valid_transactions,
            previous_hash: last_hash,
            hash: String::new(),
            nonce: 0,
        };

        // Mine the block
        self.mine_block(&mut new_block)?;

        // Validate block before adding
        if !self.validate_block(&new_block)? {
            return Err(BlockchainError::InvalidBlock);
        }

        // Update UTXO set
        self.update_utxo_set(&new_block);
        
        // Update total supply
        self.total_supply += current_reward + total_fees;

        // Add block to chain
        self.chain.push(new_block.clone());
        
        // Clear processed transactions from pool
        for tx in &new_block.transactions {
            self.transaction_pool.remove(&tx.id);
        }
        self.pending_transactions.clear();

        // Adjust difficulty if needed
        self.adjust_difficulty()?;

        self.save_to_disk("blockchain.json");

        Ok(new_block)
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), BlockchainError> {
        // Validate transaction first
        if self.validate_transaction_against_utxo(&transaction)? {
            self.add_to_pool(transaction)?;
            Ok(())
        } else {
            Err(BlockchainError::InvalidTransaction(TransactionError::InvalidSignature))
        }
    }

    pub fn mine_pending_transactions(&mut self, miner_address: &str) -> Result<Block, BlockchainError> {
        self.create_block(miner_address)
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        self.utxo_set.get(address).copied().unwrap_or(0)
    }

    pub fn is_chain_valid(&self) -> Result<bool, BlockchainError> {
        if self.chain.is_empty() {
            return Ok(true);
        }

        // Validate genesis block
        if self.chain[0].index != 0 || self.chain[0].previous_hash != "0" {
            return Ok(false);
        }

        // Validate each block
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            // Check block structure
            if !self.validate_block(current)? {
                return Ok(false);
            }

            // Check hash integrity
            if current.hash != current.calculate_hash() {
                return Ok(false);
            }

            // Check previous hash link
            if current.previous_hash != previous.hash {
                return Ok(false);
            }

            // Check index sequence
            if current.index != previous.index + 1 {
                return Ok(false);
            }

            // Check timestamp (must be after previous block)
            if current.timestamp <= previous.timestamp {
                return Ok(false);
            }

            // Validate merkle root
            if !self.validate_merkle_root(current) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn save_to_disk(&self, path: &str) {
        if let Ok(json) = serde_json::to_string(&self.chain) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load_from_disk(&mut self, path: &str) {
        if Path::new(path).exists() {
            if let Ok(mut file) = File::open(path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(chain) = serde_json::from_str::<Vec<Block>>(&contents) {
                        self.chain = chain;
                    }
                }
            }
        }
    }

    // UTXO Management
    pub fn rebuild_utxo_set(&mut self) {
        self.utxo_set.clear();
        self.total_supply = 0;

        for block in &self.chain {
            self.update_utxo_set(block);
        }
    }

    pub fn update_utxo_set(&mut self, block: &Block) {
        for tx in &block.transactions {
            // Add outputs (credits)
            if tx.sender == "MINING_REWARD" || tx.sender == "GENESIS" {
                *self.utxo_set.entry(tx.recipient.clone()).or_insert(0) += tx.amount;
                self.total_supply += tx.amount;
            } else {
                // Regular transaction
                *self.utxo_set.entry(tx.recipient.clone()).or_insert(0) += tx.amount;
                
                // Subtract inputs (debits) including fees
                if let Some(sender_balance) = self.utxo_set.get_mut(&tx.sender) {
                    *sender_balance = sender_balance.saturating_sub(tx.amount + tx.fee);
                }
            }
        }
    }

    // Transaction Validation
    pub fn validate_transaction_against_utxo(&self, transaction: &Transaction) -> Result<bool, BlockchainError> {
        // System transactions are always valid
        if transaction.sender == "GENESIS" || transaction.sender == "MINING_REWARD" || transaction.sender == "SYSTEM" {
            return Ok(true);
        }

        // Check if sender has sufficient balance
        let sender_balance = self.get_balance(&transaction.sender);
        let total_cost = transaction.amount + transaction.fee;

        if sender_balance < total_cost {
            return Err(BlockchainError::InsufficientBalance);
        }

        // Prevent double spending by checking transaction pool
        if !self.prevent_double_spending(transaction) {
            return Err(BlockchainError::DoubleSpending);
        }

        // Validate transaction signature and structure
        match transaction.is_valid(sender_balance) {
            Ok(true) => Ok(true),
            Ok(false) => Ok(false),
            Err(e) => Err(BlockchainError::InvalidTransaction(e)),
        }
    }

    pub fn prevent_double_spending(&self, transaction: &Transaction) -> bool {
        // Check if this transaction is already in the pool
        if self.transaction_pool.contains_key(&transaction.id) {
            return false;
        }

        // Check for conflicting transactions in the pool
        let sender_pending_amount: u64 = self.transaction_pool
            .values()
            .filter(|tx| tx.sender == transaction.sender)
            .map(|tx| tx.amount + tx.fee)
            .sum();

        let sender_balance = self.get_balance(&transaction.sender);
        let required_balance = sender_pending_amount + transaction.amount + transaction.fee;

        sender_balance >= required_balance
    }

    // Transaction Pool Management
    pub fn add_to_pool(&mut self, transaction: Transaction) -> Result<(), BlockchainError> {
        if self.transaction_pool.len() >= 10000 { // Max pool size
            return Err(BlockchainError::InvalidTransaction(TransactionError::FeeTooLow));
        }

        self.transaction_pool.insert(transaction.id.clone(), transaction.clone());
        self.pending_transactions.push(transaction);
        Ok(())
    }

    pub fn remove_from_pool(&mut self, transaction_id: &str) -> Option<Transaction> {
        if let Some(tx) = self.transaction_pool.remove(transaction_id) {
            self.pending_transactions.retain(|t| t.id != transaction_id);
            Some(tx)
        } else {
            None
        }
    }

    pub fn get_pending_transactions(&self) -> Vec<Transaction> {
        self.pending_transactions.clone()
    }

    // Mining and Difficulty
    pub fn mine_block(&self, block: &mut Block) -> Result<(), BlockchainError> {
        let target = "0".repeat(self.difficulty);
        
        loop {
            block.hash = block.calculate_hash();
            if block.hash.starts_with(&target) {
                break;
            }
            block.nonce += 1;
            
            // Prevent infinite loops in case of impossible difficulty
            if block.nonce > u64::MAX - 1000 {
                return Err(BlockchainError::DifficultyAdjustmentFailed);
            }
        }
        
        Ok(())
    }

    pub fn adjust_difficulty(&mut self) -> Result<(), BlockchainError> {
        let config = &self.difficulty_config;
        
        if self.chain.len() < config.adjustment_interval as usize {
            return Ok(()); // Not enough blocks yet
        }

        let current_block = &self.chain[self.chain.len() - 1];
        let target_block = &self.chain[self.chain.len() - config.adjustment_interval as usize];
        
        let actual_time = (current_block.timestamp - target_block.timestamp) / 1000; // Convert to seconds
        let expected_time = config.target_block_time * config.adjustment_interval;
        
        let adjustment_factor = actual_time as f64 / expected_time as f64;
        let clamped_factor = adjustment_factor.max(1.0 / config.max_adjustment_factor)
            .min(config.max_adjustment_factor);

        if clamped_factor > 1.1 {
            // Decrease difficulty (increase target)
            self.difficulty = self.difficulty.saturating_sub(1).max(1);
        } else if clamped_factor < 0.9 {
            // Increase difficulty (decrease target)
            self.difficulty = (self.difficulty + 1).min(32);
        }

        Ok(())
    }

    pub fn calculate_mining_reward(&self) -> u64 {
        let halvings = self.chain.len() as u64 / self.halving_interval;
        let base_reward = 50_000_000u64; // 50 QTC in satoshis
        
        if halvings >= 32 {
            return 0; // After 32 halvings, reward is effectively 0
        }
        
        base_reward >> halvings // Equivalent to dividing by 2^halvings
    }

    // Block Validation
    pub fn validate_block(&self, block: &Block) -> Result<bool, BlockchainError> {
        // Check block structure
        if block.transactions.is_empty() {
            return Ok(false);
        }

        // Check timestamp is reasonable (not too far in future)
        if !self.validate_block_timestamp(block) {
            return Ok(false);
        }

        // Validate all transactions in block
        for (i, tx) in block.transactions.iter().enumerate() {
            if i == 0 {
                // First transaction should be mining reward
                if tx.sender != "MINING_REWARD" {
                    return Ok(false);
                }
            } else {
                // Validate regular transactions
                if !self.validate_transaction_against_utxo(tx)? {
                    return Ok(false);
                }
            }
        }

        // Check proof of work
        let target = "0".repeat(self.difficulty);
        if !block.hash.starts_with(&target) {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn validate_merkle_root(&self, block: &Block) -> bool {
        let merkle_tree = MerkleTree::new(&block.transactions);
        // For now, just check if merkle tree can be built
        !merkle_tree.root.is_empty()
    }

    pub fn validate_block_timestamp(&self, block: &Block) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        // Block timestamp should not be more than 2 hours in the future
        let max_future = now + (2 * 60 * 60 * 1000); // 2 hours in milliseconds
        
        block.timestamp <= max_future
    }

    // Supply and Economics
    pub fn get_total_supply(&self) -> u64 {
        self.total_supply
    }

    pub fn get_max_supply(&self) -> u64 {
        self.max_supply
    }

    pub fn get_circulation_percentage(&self) -> f64 {
        if self.max_supply == 0 {
            return 0.0;
        }
        (self.total_supply as f64 / self.max_supply as f64) * 100.0
    }

    // Utility Methods
    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash == hash)
    }

    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn get_chain_height(&self) -> u64 {
        self.chain.len() as u64
    }

    pub fn get_difficulty(&self) -> usize {
        self.difficulty
    }

    pub fn get_pending_transaction_count(&self) -> usize {
        self.pending_transactions.len()
    }

    pub fn get_transaction_pool_size(&self) -> usize {
        self.transaction_pool.len()
    }

    // Network hashrate estimation
    pub fn estimate_network_hashrate(&self) -> f64 {
        if self.chain.len() < 2 {
            return 0.0;
        }

        let latest_block = &self.chain[self.chain.len() - 1];
        let prev_block = &self.chain[self.chain.len() - 2];
        
        let time_diff = (latest_block.timestamp - prev_block.timestamp) as f64 / 1000.0; // seconds
        let target = 2u64.pow(256 - (self.difficulty * 4) as u32) as f64;
        
        target / time_diff
    }
}

// Implement the missing calculate_hash method for Block
impl Block {
    pub fn calculate_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.nonce.to_be_bytes());
        
        // Add transaction hashes
        for tx in &self.transactions {
            hasher.update(tx.calculate_hash().as_bytes());
        }
        
        hex::encode(hasher.finalize())
    }
}