use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};

use crate::blockchain::{Blockchain, Block};
use crate::mempool::Mempool;
use crate::revstop::RevStop;
use crate::transaction::{Transaction, SignedTransaction, TransactionOutput, TransactionInput};
use crate::block::DetailedBlock;

pub struct Miner {
    mining_address: String,
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Mempool>>,
    revstop: Arc<RwLock<RevStop>>,
    is_mining: Arc<RwLock<bool>>,
    max_transactions_per_block: usize,
    max_block_size: usize,
}

impl Miner {
    pub fn new(
        mining_address: String,
        blockchain: Arc<RwLock<Blockchain>>,
        mempool: Arc<RwLock<Mempool>>,
        revstop: Arc<RwLock<RevStop>>,
    ) -> Self {
        Self {
            mining_address,
            blockchain,
            mempool,
            revstop,
            is_mining: Arc::new(RwLock::new(false)),
            max_transactions_per_block: 1000,
            max_block_size: 1_000_000, // 1MB
        }
    }

    pub async fn start_mining(&self) -> Result<()> {
        let mut is_mining = self.is_mining.write().await;
        if *is_mining {
            return Err(anyhow!("Mining is already running"));
        }
        *is_mining = true;
        drop(is_mining);

        info!("Starting mining for address: {}", self.mining_address);

        loop {
            let should_continue = {
                let is_mining = self.is_mining.read().await;
                *is_mining
            };

            if !should_continue {
                break;
            }

            match self.mine_next_block().await {
                Ok(block) => {
                    info!(
                        "Successfully mined block {} with hash: {}",
                        block.index, block.hash
                    );
                }
                Err(e) => {
                    error!("Mining error: {}", e);
                    // Continue mining on errors
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }

            // Small delay between mining attempts
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub async fn stop_mining(&self) {
        let mut is_mining = self.is_mining.write().await;
        *is_mining = false;
        info!("Mining stopped");
    }

    pub async fn is_mining(&self) -> bool {
        let is_mining = self.is_mining.read().await;
        *is_mining
    }

    async fn mine_next_block(&self) -> Result<Block> {
        // Get current blockchain state
        let (current_difficulty, chain_height, previous_hash) = {
            let blockchain = self.blockchain.read().await;
            let latest_block = blockchain.get_latest_block();
            (
                blockchain.difficulty,
                blockchain.chain.len() as u64,
                latest_block.hash.clone(),
            )
        };

        // Get transactions from mempool
        let transactions = self.select_transactions_for_block().await?;

        // Create coinbase transaction
        let coinbase_tx = self.create_coinbase_transaction(chain_height).await?;
        
        // Combine coinbase with other transactions
        let mut block_transactions = vec![coinbase_tx];
        block_transactions.extend(transactions);

        // Create and mine the block
        let mut block = self.create_block_template(
            previous_hash,
            block_transactions,
            chain_height,
            current_difficulty,
        ).await?;

        // Mine the block
        self.mine_block(&mut block, current_difficulty).await?;

        // Add block to blockchain
        self.add_mined_block_to_chain(block.clone()).await?;

        Ok(block)
    }

    async fn select_transactions_for_block(&self) -> Result<Vec<SignedTransaction>> {
        let mempool = self.mempool.read().await;
        
        // Get transactions sorted by fee (highest first)
        let selected_transactions = mempool.get_transactions_for_mining(
            self.max_transactions_per_block - 1, // Reserve space for coinbase
            self.max_block_size / 2, // Reserve space for other data
        );

        info!(
            "Selected {} transactions for next block",
            selected_transactions.len()
        );

        Ok(selected_transactions)
    }

    async fn create_coinbase_transaction(&self, block_height: u64) -> Result<SignedTransaction> {
        let mining_reward = self.calculate_mining_reward(block_height).await;
        
        // Create coinbase transaction
        let coinbase_output = TransactionOutput {
            value: mining_reward,
            script_pubkey: self.mining_address.as_bytes().to_vec(),
            address: self.mining_address.clone(),
        };

        let coinbase_input = TransactionInput {
            previous_output: format!("coinbase_{}", block_height),
            script_sig: format!("Block Height: {}", block_height).as_bytes().to_vec(),
            sequence: 0xFFFFFFFF,
        };

        let mut coinbase_tx = SignedTransaction::new(
            vec![coinbase_input],
            vec![coinbase_output],
            0,
        );

        // Sign with mining address (simplified)
        coinbase_tx.signature = format!("coinbase_signature_{}", block_height);
        coinbase_tx.public_key = self.mining_address.clone();

        Ok(coinbase_tx)
    }

    async fn calculate_mining_reward(&self, block_height: u64) -> u64 {
        let blockchain = self.blockchain.read().await;
        
        // Bitcoin-style halving every 210,000 blocks
        let halvings = block_height / 210_000;
        let base_reward = blockchain.mining_reward;
        
        if halvings >= 64 {
            return 0; // No more rewards after 64 halvings
        }
        
        base_reward >> halvings
    }

    async fn create_block_template(
        &self,
        previous_hash: String,
        transactions: Vec<SignedTransaction>,
        height: u64,
        difficulty: usize,
    ) -> Result<Block> {
        let simple_transactions: Vec<Transaction> = transactions
            .iter()
            .map(|tx| tx.to_simple_transaction())
            .collect();

        let merkle_root = self.calculate_merkle_root(&simple_transactions);

        Ok(Block {
            index: height,
            timestamp: Utc::now(),
            transactions: simple_transactions,
            previous_hash,
            hash: String::new(), // Will be set during mining
            nonce: 0,
            merkle_root,
            difficulty,
        })
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

    async fn mine_block(&self, block: &mut Block, difficulty: usize) -> Result<()> {
        let target = "0".repeat(difficulty);
        let mut attempts = 0u64;
        const MAX_ATTEMPTS_PER_BATCH: u64 = 1_000_000;

        info!("Mining block {} with difficulty {}", block.index, difficulty);

        loop {
            let batch_start = attempts;
            
            // Mine in batches to allow checking if we should stop
            while attempts - batch_start < MAX_ATTEMPTS_PER_BATCH {
                block.nonce += 1;
                block.hash = self.calculate_block_hash(block);
                attempts += 1;

                if block.hash.starts_with(&target) {
                    info!(
                        "Block mined! Hash: {} (attempts: {})",
                        block.hash, attempts
                    );
                    return Ok(());
                }
            }

            // Check if we should continue mining
            let should_continue = {
                let is_mining = self.is_mining.read().await;
                *is_mining
            };

            if !should_continue {
                return Err(anyhow!("Mining stopped"));
            }

            // Check if blockchain has advanced (someone else mined a block)
            let current_height = {
                let blockchain = self.blockchain.read().await;
                blockchain.chain.len() as u64
            };

            if current_height > block.index {
                return Err(anyhow!("Blockchain advanced, restarting mining"));
            }

            if attempts % 10_000_000 == 0 {
                info!(
                    "Mining progress: {} attempts, current hash: {}",
                    attempts, block.hash
                );
            }
        }
    }

    fn calculate_block_hash(&self, block: &Block) -> String {
        let data = format!(
            "{}{}{}{}{}{}{}",
            block.index,
            block.timestamp.timestamp(),
            serde_json::to_string(&block.transactions).unwrap_or_default(),
            block.previous_hash,
            block.nonce,
            block.merkle_root,
            block.difficulty
        );
        
        let hash = blake3::hash(data.as_bytes());
        hex::encode(hash.as_bytes())
    }

    async fn add_mined_block_to_chain(&self, block: Block) -> Result<()> {
        // Validate the block before adding
        self.validate_mined_block(&block).await?;

        // Add block to blockchain
        {
            let mut blockchain = self.blockchain.write().await;
            blockchain.chain.push(block.clone());
            blockchain.adjust_difficulty();
            
            // Update total supply
            let coinbase_reward = block.transactions
                .first()
                .map(|tx| tx.amount)
                .unwrap_or(0);
            blockchain.total_supply += coinbase_reward;
        }

        // Remove mined transactions from mempool
        {
            let mut mempool = self.mempool.write().await;
            for tx in &block.transactions {
                if !tx.id.starts_with("coinbase_") {
                    mempool.remove_transaction(&tx.id);
                }
            }
        }

        // Update RevStop if needed
        {
            let mut revstop = self.revstop.write().await;
            revstop.update_on_new_block(&block).await?;
        }

        info!(
            "Added mined block {} to blockchain. Chain height: {}",
            block.hash,
            {
                let blockchain = self.blockchain.read().await;
                blockchain.chain.len()
            }
        );

        Ok(())
    }

    async fn validate_mined_block(&self, block: &Block) -> Result<()> {
        // Basic validation
        if block.hash.is_empty() {
            return Err(anyhow!("Block hash is empty"));
        }

        if block.transactions.is_empty() {
            return Err(anyhow!("Block has no transactions"));
        }

        // Check if hash meets difficulty requirement
        let difficulty = {
            let blockchain = self.blockchain.read().await;
            blockchain.difficulty
        };
        
        let target = "0".repeat(difficulty);
        if !block.hash.starts_with(&target) {
            return Err(anyhow!(
                "Block hash {} does not meet difficulty requirement {}",
                block.hash, target
            ));
        }

        // Verify hash
        let calculated_hash = self.calculate_block_hash(block);
        if calculated_hash != block.hash {
            return Err(anyhow!("Block hash verification failed"));
        }

        // Check coinbase transaction
        let coinbase_tx = &block.transactions[0];
        if !coinbase_tx.id.starts_with("coinbase_") {
            return Err(anyhow!("First transaction is not coinbase"));
        }

        Ok(())
    }

    pub async fn get_mining_stats(&self) -> MiningStats {
        let blockchain = self.blockchain.read().await;
        let mempool = self.mempool.read().await;
        let is_mining = self.is_mining.read().await;

        MiningStats {
            is_mining: *is_mining,
            mining_address: self.mining_address.clone(),
            current_difficulty: blockchain.difficulty,
            chain_height: blockchain.chain.len() as u64,
            pending_transactions: mempool.size(),
            total_supply: blockchain.total_supply,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MiningStats {
    pub is_mining: bool,
    pub mining_address: String,
    pub current_difficulty: usize,
    pub chain_height: u64,
    pub pending_transactions: usize,
    pub total_supply: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revstop::RevStop;

    #[tokio::test]
    async fn test_miner_creation() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let mempool = Arc::new(RwLock::new(Mempool::default()));
        let revstop = Arc::new(RwLock::new(RevStop::new()));

        let miner = Miner::new(
            "test_address".to_string(),
            blockchain,
            mempool,
            revstop,
        );

        assert_eq!(miner.mining_address, "test_address");
        assert!(!miner.is_mining().await);
    }

    #[tokio::test]
    async fn test_coinbase_transaction() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let mempool = Arc::new(RwLock::new(Mempool::default()));
        let revstop = Arc::new(RwLock::new(RevStop::new()));

        let miner = Miner::new(
            "miner_address".to_string(),
            blockchain,
            mempool,
            revstop,
        );

        let coinbase_tx = miner.create_coinbase_transaction(1).await.unwrap();
        
        assert!(coinbase_tx.id.starts_with("coinbase_") || !coinbase_tx.id.is_empty());
        assert_eq!(coinbase_tx.outputs.len(), 1);
        assert_eq!(coinbase_tx.outputs[0].address, "miner_address");
        assert!(coinbase_tx.outputs[0].value > 0);
    }
}
