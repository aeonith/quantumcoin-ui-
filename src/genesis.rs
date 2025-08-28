use crate::block::Block;
use crate::transaction::Transaction;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisConfig {
    pub total_supply: u64,
    pub initial_difficulty: u64,
    pub block_reward: u64,
    pub halving_interval: u64,
    pub target_block_time: u64,
    pub genesis_message: String,
    pub genesis_timestamp: DateTime<Utc>,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            total_supply: 22_000_000_000_000_000, // 22 million QTC max supply (NOT premined)
            initial_difficulty: 1000000,
            block_reward: 5_000_000_000, // 50 QTC initial reward
            halving_interval: 105_120,    // Halving every ~2 years (105,120 blocks)
            target_block_time: 600,      // 10 minutes like Bitcoin
            genesis_message: "QuantumCoin Genesis - Fair Launch, No Premine".to_string(),
            genesis_timestamp: Utc::now(),
        }
    }
}

pub struct GenesisBuilder {
    config: GenesisConfig,
}

impl GenesisBuilder {
    pub fn new() -> Self {
        Self {
            config: GenesisConfig::default(),
        }
    }

    pub fn with_config(config: GenesisConfig) -> Self {
        Self { config }
    }

    pub fn create_genesis_block(&self) -> Block {
        let genesis_transaction = self.create_genesis_transaction();
        
        let mut block = Block::new(
            0, // Genesis block height
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(), // Previous hash
            vec![genesis_transaction],
            self.config.initial_difficulty,
        );

        // Set genesis timestamp
        block.timestamp = self.config.genesis_timestamp;
        
        // Mine the genesis block
        self.mine_genesis_block(&mut block);
        
        block
    }

    fn create_genesis_transaction(&self) -> Transaction {
        // Create empty coinbase transaction for genesis block
        // NO PREMINE - all coins must be mined through Proof-of-Work
        let mut genesis_tx = Transaction::new(
            "0000000000000000000000000000000000000000".to_string(), // Coinbase address
            "0000000000000000000000000000000000000000".to_string(), // No recipient - fair launch
            0, // NO PREMINE - zero coins allocated
            0, // No fee for genesis
            0, // Nonce 0
        );

        // Set genesis message in transaction data
        genesis_tx.id = "genesis_coinbase_transaction".to_string();
        genesis_tx.timestamp = self.config.genesis_timestamp;
        
        genesis_tx
    }

    fn mine_genesis_block(&self, block: &mut Block) {
        let target = self.calculate_target(self.config.initial_difficulty);
        let mut nonce = 0u64;

        loop {
            block.nonce = nonce;
            let hash = self.calculate_block_hash(block);
            
            if self.hash_meets_target(&hash, &target) {
                block.hash = hash;
                break;
            }
            
            nonce += 1;
            
            if nonce % 100000 == 0 {
                println!("Genesis mining progress: {} attempts", nonce);
            }
        }

        println!("Genesis block mined! Hash: {}", block.hash);
        println!("Nonce: {}", block.nonce);
    }

    fn calculate_target(&self, difficulty: u64) -> Vec<u8> {
        let max_target = vec![0xFF; 32];
        let target_value = u128::from_be_bytes([0xFF; 16]) / difficulty as u128;
        
        let mut target = vec![0u8; 32];
        let target_bytes = target_value.to_be_bytes();
        
        for (i, &byte) in target_bytes.iter().enumerate() {
            if i < 32 {
                target[i] = byte;
            }
        }
        
        target
    }

    fn calculate_block_hash(&self, block: &Block) -> String {
        let block_string = format!(
            "{}{}{}{}{}{}",
            block.index,
            block.previous_hash,
            block.timestamp.timestamp(),
            block.merkle_root,
            block.nonce,
            block.difficulty
        );
        
        let mut hasher = Sha256::new();
        hasher.update(block_string.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn hash_meets_target(&self, hash: &str, target: &[u8]) -> bool {
        let hash_bytes = hex::decode(hash).unwrap_or_default();
        if hash_bytes.len() != 32 {
            return false;
        }
        
        for (i, (&hash_byte, &target_byte)) in hash_bytes.iter().zip(target.iter()).enumerate() {
            if hash_byte < target_byte {
                return true;
            } else if hash_byte > target_byte {
                return false;
            }
        }
        
        true
    }
}

pub fn create_mainnet_genesis() -> Block {
    let config = GenesisConfig {
        total_supply: 21_000_000_000_000_000, // 21 million QTC
        initial_difficulty: 1000000,
        block_reward: 5_000_000_000, // 50 QTC
        halving_interval: 210_000,
        target_block_time: 600,
        genesis_message: "QuantumCoin Mainnet Genesis - The Future of Quantum-Safe Money".to_string(),
        genesis_timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
    };

    GenesisBuilder::with_config(config).create_genesis_block()
}

pub fn create_testnet_genesis() -> Block {
    let config = GenesisConfig {
        total_supply: 21_000_000_000_000_000,
        initial_difficulty: 1000, // Lower difficulty for testing
        block_reward: 5_000_000_000,
        halving_interval: 150, // Faster halving for testing
        target_block_time: 60, // 1 minute blocks for testing
        genesis_message: "QuantumCoin Testnet Genesis - Testing Quantum-Safe Future".to_string(),
        genesis_timestamp: Utc::now(),
    };

    GenesisBuilder::with_config(config).create_genesis_block()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_creation() {
        let genesis = GenesisBuilder::new().create_genesis_block();
        
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0000000000000000000000000000000000000000000000000000000000000000");
        assert!(!genesis.transactions.is_empty());
        assert!(!genesis.hash.is_empty());
    }

    #[test]
    fn test_mainnet_genesis() {
        let genesis = create_mainnet_genesis();
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.transactions[0].amount, 21_000_000_000_000_000);
    }
}
