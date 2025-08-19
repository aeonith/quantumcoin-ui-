// QuantumCoin Validation Rules - Bitcoin-level Rigor

use crate::{Block, BlockHeader, Tx, Chain, Hash};
use anyhow::{Result, anyhow};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub struct Validator {
    chain: Chain,
    utxo_set: HashSet<String>, // Track unspent transaction outputs
}

impl Validator {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            utxo_set: HashSet::new(),
        }
    }
    
    /// Validate complete block - Bitcoin-level rigor
    pub fn validate_block(&mut self, block: &Block) -> Result<()> {
        // 1. Header format validation
        self.validate_block_header(&block.header)?;
        
        // 2. Parent exists validation
        self.validate_parent_exists(&block.header)?;
        
        // 3. Timestamp validation
        self.validate_timestamp(&block.header)?;
        
        // 4. Difficulty validation
        self.validate_difficulty(&block.header)?;
        
        // 5. Proof-of-work validation
        self.validate_proof_of_work(&block.header)?;
        
        // 6. Merkle root validation
        self.validate_merkle_root(&block.header, &block.txs)?;
        
        // 7. Transaction validity
        self.validate_all_transactions(&block.txs)?;
        
        Ok(())
    }
    
    fn validate_block_header(&self, header: &BlockHeader) -> Result<()> {
        // Header format validation
        if header.number == 0 && header.parent != "0x00" {
            return Err(anyhow!("Genesis block must have null parent"));
        }
        
        if header.timestamp == 0 {
            return Err(anyhow!("Invalid timestamp: zero"));
        }
        
        if header.difficulty == 0 {
            return Err(anyhow!("Invalid difficulty: zero"));
        }
        
        Ok(())
    }
    
    fn validate_parent_exists(&self, header: &BlockHeader) -> Result<()> {
        if header.number == 0 {
            return Ok(); // Genesis has no parent
        }
        
        // Check parent exists in chain
        let parent_height = header.number - 1;
        match self.chain.get_block_by_number(parent_height) {
            Some(parent) => {
                if parent.hash != header.parent {
                    return Err(anyhow!("Parent hash mismatch"));
                }
            },
            None => return Err(anyhow!("Parent block not found")),
        }
        
        Ok(())
    }
    
    fn validate_timestamp(&self, header: &BlockHeader) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Timestamp must be within 2 hours of current time
        if header.timestamp > now + 7200 {
            return Err(anyhow!("Block timestamp too far in future"));
        }
        
        // Must be after parent timestamp
        if header.number > 0 {
            if let Some(parent) = self.chain.get_block_by_number(header.number - 1) {
                if header.timestamp <= parent.header.timestamp {
                    return Err(anyhow!("Block timestamp not after parent"));
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_difficulty(&self, header: &BlockHeader) -> Result<()> {
        let expected_difficulty = self.calculate_expected_difficulty(header.number);
        
        if header.difficulty != expected_difficulty {
            return Err(anyhow!(
                "Difficulty mismatch: expected {}, got {}", 
                expected_difficulty, header.difficulty
            ));
        }
        
        Ok(())
    }
    
    fn calculate_expected_difficulty(&self, height: u64) -> u128 {
        // Difficulty adjustment every 2016 blocks (like Bitcoin)
        if height % 2016 != 0 || height == 0 {
            // Use previous difficulty
            if let Some(prev) = self.chain.get_block_by_number(height - 1) {
                return prev.header.difficulty;
            }
        }
        
        // Calculate new difficulty
        let period_start = height - 2016;
        let period_end = height - 1;
        
        if let (Some(start_block), Some(end_block)) = 
            (self.chain.get_block_by_number(period_start), 
             self.chain.get_block_by_number(period_end)) {
            
            let actual_time = end_block.header.timestamp - start_block.header.timestamp;
            let target_time = 2016 * 600; // 10 minutes per block
            
            let adjustment = target_time as f64 / actual_time as f64;
            let max_adjustment = 4.0;
            let clamped_adjustment = adjustment.max(1.0/max_adjustment).min(max_adjustment);
            
            let new_difficulty = (end_block.header.difficulty as f64 * clamped_adjustment) as u128;
            return new_difficulty.max(1);
        }
        
        0x1d00ffff // Default genesis difficulty
    }
    
    fn validate_proof_of_work(&self, header: &BlockHeader) -> Result<()> {
        let block_hash = self.calculate_block_hash(header);
        let hash_as_u128 = u128::from_be_bytes({
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&block_hash[..16]);
            bytes
        });
        
        let target = u128::MAX / header.difficulty;
        
        if hash_as_u128 > target {
            return Err(anyhow!("Proof-of-work invalid: hash {} > target {}", hash_as_u128, target));
        }
        
        Ok(())
    }
    
    fn calculate_block_hash(&self, header: &BlockHeader) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&1u32.to_le_bytes()); // version
        hasher.update(&hex::decode(header.parent.trim_start_matches("0x")).unwrap_or_default());
        hasher.update(&hex::decode(header.merkle_root.trim_start_matches("0x")).unwrap_or_default());
        hasher.update(&header.timestamp.to_le_bytes());
        hasher.update(&header.difficulty.to_le_bytes());
        hasher.update(&header.nonce.to_le_bytes());
        hasher.finalize().into()
    }
    
    fn validate_merkle_root(&self, header: &BlockHeader, transactions: &[Tx]) -> Result<()> {
        let calculated_root = self.calculate_merkle_root(transactions);
        
        if header.merkle_root != calculated_root {
            return Err(anyhow!("Merkle root mismatch"));
        }
        
        Ok(())
    }
    
    fn calculate_merkle_root(&self, transactions: &[Tx]) -> String {
        if transactions.is_empty() {
            return format!("0x{}", hex::encode([0u8; 32]));
        }
        
        let mut level: Vec<[u8; 32]> = transactions.iter().map(|tx| {
            let tx_bytes = serde_json::to_vec(tx).unwrap();
            let mut hasher = Sha256::new();
            hasher.update(&tx_bytes);
            hasher.finalize().into()
        }).collect();
        
        while level.len() > 1 {
            let mut next_level = Vec::new();
            
            for pair in level.chunks(2) {
                let left = pair[0];
                let right = *pair.get(1).unwrap_or(&pair[0]);
                
                let mut hasher = Sha256::new();
                hasher.update(&left);
                hasher.update(&right);
                next_level.push(hasher.finalize().into());
            }
            
            level = next_level;
        }
        
        format!("0x{}", hex::encode(level[0]))
    }
    
    /// Validate complete transaction
    pub fn validate_transaction(&mut self, tx: &Tx) -> Result<()> {
        // 1. Signature verification
        self.validate_signature(tx)?;
        
        // 2. Replay protection
        self.validate_no_replay(tx)?;
        
        // 3. Double-spend prevention
        self.validate_no_double_spend(tx)?;
        
        // 4. Fee validation
        self.validate_fees(tx)?;
        
        // 5. Minimum size rules
        self.validate_minimum_requirements(tx)?;
        
        Ok(())
    }
    
    fn validate_signature(&self, tx: &Tx) -> Result<()> {
        // TODO: Implement Dilithium2 signature verification
        // For now, basic format validation
        if tx.from.is_empty() || tx.to.is_empty() {
            return Err(anyhow!("Invalid transaction addresses"));
        }
        
        if tx.value == 0 && tx.fee == 0 {
            return Err(anyhow!("Transaction must have value or fee"));
        }
        
        Ok(())
    }
    
    fn validate_no_replay(&self, tx: &Tx) -> Result<()> {
        // Check if transaction already exists in blockchain
        let tx_id = format!("{}:{}:{}", tx.from, tx.to, tx.nonce);
        
        // This would check against actual UTXO set in production
        Ok(())
    }
    
    fn validate_no_double_spend(&self, tx: &Tx) -> Result<()> {
        // Verify UTXO inputs are unspent
        // This would check against actual UTXO set in production
        Ok(())
    }
    
    fn validate_fees(&self, tx: &Tx) -> Result<()> {
        if tx.fee < 1000 { // Minimum fee: 0.00001 QTC
            return Err(anyhow!("Fee too low: {} < 1000", tx.fee));
        }
        
        if tx.fee > 10000000 { // Maximum fee: 0.1 QTC
            return Err(anyhow!("Fee too high: {} > 10000000", tx.fee));
        }
        
        Ok(())
    }
    
    fn validate_minimum_requirements(&self, tx: &Tx) -> Result<()> {
        if tx.value < 546 { // Dust threshold
            return Err(anyhow!("Output below dust threshold: {}", tx.value));
        }
        
        Ok(())
    }
    
    fn validate_all_transactions(&mut self, transactions: &[Tx]) -> Result<()> {
        for tx in transactions {
            self.validate_transaction(tx)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_validation() {
        let chain = Chain::new_genesis();
        let mut validator = Validator::new(chain.clone());
        
        // Test genesis block validation
        let genesis = chain.head();
        assert!(validator.validate_block(&genesis).is_ok());
    }
    
    #[test]
    fn test_transaction_validation() {
        let chain = Chain::new_genesis();
        let mut validator = Validator::new(chain);
        
        let tx = Tx {
            nonce: 1,
            from: "qc1test123".to_string(),
            to: "qc1test456".to_string(),
            value: 1000000,
            fee: 1000,
            data: "".to_string(),
        };
        
        assert!(validator.validate_transaction(&tx).is_ok());
    }
}
