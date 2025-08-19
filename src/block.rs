pub use crate::blockchain::Block;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::transaction::{Transaction, SignedTransaction};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_block_hash: String,
    pub merkle_root: String,
    pub timestamp: DateTime<Utc>,
    pub difficulty_target: u32,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedBlock {
    pub header: BlockHeader,
    pub transactions: Vec<SignedTransaction>,
    pub hash: String,
    pub size: usize,
    pub height: u64,
}

impl DetailedBlock {
    pub fn new(
        previous_hash: String,
        transactions: Vec<SignedTransaction>,
        height: u64,
        difficulty: u32,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        let header = BlockHeader {
            version: 1,
            previous_block_hash: previous_hash,
            merkle_root,
            timestamp: Utc::now(),
            difficulty_target: difficulty,
            nonce: 0,
        };

        let mut block = Self {
            header,
            transactions,
            hash: String::new(),
            size: 0,
            height,
        };

        block.hash = block.calculate_hash();
        block.size = block.calculate_size();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let header_data = format!(
            "{}{}{}{}{}{}",
            self.header.version,
            self.header.previous_block_hash,
            self.header.merkle_root,
            self.header.timestamp.timestamp(),
            self.header.difficulty_target,
            self.header.nonce
        );
        
        let hash = blake3::hash(header_data.as_bytes());
        hex::encode(hash.as_bytes())
    }

    pub fn calculate_merkle_root(transactions: &[SignedTransaction]) -> String {
        if transactions.is_empty() {
            return "0".to_string();
        }

        let mut tx_hashes: Vec<String> = transactions
            .iter()
            .map(|tx| tx.id.clone())
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

    pub fn calculate_size(&self) -> usize {
        bincode::serialize(self).map(|data| data.len()).unwrap_or(0)
    }

    pub fn mine(&mut self, target_difficulty: usize) -> Result<()> {
        let target = "0".repeat(target_difficulty);
        let mut attempts = 0u64;
        const MAX_ATTEMPTS: u64 = u64::MAX;

        while !self.hash.starts_with(&target) {
            if attempts >= MAX_ATTEMPTS {
                return Err(anyhow!("Mining timeout reached"));
            }
            
            self.header.nonce = self.header.nonce.wrapping_add(1);
            self.hash = self.calculate_hash();
            attempts += 1;
            
            if attempts % 1_000_000 == 0 {
                println!("Mining attempt: {}, current hash: {}", attempts, self.hash);
            }
        }

        println!("Block mined after {} attempts: {}", attempts, self.hash);
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        // Validate hash
        let calculated_hash = self.calculate_hash();
        if calculated_hash != self.hash {
            return Err(anyhow!("Invalid block hash"));
        }

        // Validate merkle root
        let calculated_merkle_root = Self::calculate_merkle_root(&self.transactions);
        if calculated_merkle_root != self.header.merkle_root {
            return Err(anyhow!("Invalid merkle root"));
        }

        // Validate timestamp (not too far in the future)
        let now = Utc::now();
        let max_future_time = now + chrono::Duration::hours(2);
        if self.header.timestamp > max_future_time {
            return Err(anyhow!("Block timestamp too far in the future"));
        }

        // Validate transactions
        for transaction in &self.transactions {
            if transaction.id.is_empty() {
                return Err(anyhow!("Transaction has empty ID"));
            }
            
            if transaction.outputs.is_empty() {
                return Err(anyhow!("Transaction has no outputs"));
            }
            
            for output in &transaction.outputs {
                if output.value == 0 {
                    return Err(anyhow!("Transaction output has zero value"));
                }
            }
        }

        Ok(())
    }

    pub fn to_simple_block(&self) -> Block {
        let simple_transactions: Vec<Transaction> = self.transactions
            .iter()
            .map(|tx| tx.to_simple_transaction())
            .collect();

        Block {
            index: self.height,
            timestamp: self.header.timestamp,
            transactions: simple_transactions,
            previous_hash: self.header.previous_block_hash.clone(),
            hash: self.hash.clone(),
            nonce: self.header.nonce,
            merkle_root: self.header.merkle_root.clone(),
            difficulty: self.header.difficulty_target as usize,
        }
    }
}

pub struct BlockValidator {
    max_block_size: usize,
    max_transactions_per_block: usize,
}

impl BlockValidator {
    pub fn new(max_block_size: usize, max_transactions_per_block: usize) -> Self {
        Self {
            max_block_size,
            max_transactions_per_block,
        }
    }

    pub fn validate_block(&self, block: &DetailedBlock) -> Result<()> {
        // Basic block validation
        block.validate()?;

        // Size validation
        if block.size > self.max_block_size {
            return Err(anyhow!(
                "Block size {} exceeds maximum {}", 
                block.size, 
                self.max_block_size
            ));
        }

        // Transaction count validation
        if block.transactions.len() > self.max_transactions_per_block {
            return Err(anyhow!(
                "Block has {} transactions, maximum is {}", 
                block.transactions.len(), 
                self.max_transactions_per_block
            ));
        }

        Ok(())
    }
}

impl Default for BlockValidator {
    fn default() -> Self {
        Self::new(1_000_000, 10_000) // 1MB blocks, 10k transactions max
    }
}
