use crate::{Blockchain, Transaction, Block};
use crate::mempool::Mempool;
use crate::revstop::RevStop;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Utc;
use rand::Rng;
use sha2::{Sha256, Digest};
use anyhow::Result;
use tracing::{info, warn, error};

pub struct Miner {
    pub miner_address: String,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub mempool: Arc<RwLock<Mempool>>,
    pub is_mining: Arc<RwLock<bool>>,
    pub revstop: Arc<RwLock<RevStop>>,
}

impl Miner {
    pub fn new(
        miner_address: String,
        blockchain: Arc<RwLock<Blockchain>>,
        mempool: Arc<RwLock<Mempool>>,
        revstop: Arc<RwLock<RevStop>>,
    ) -> Self {
        Self {
            miner_address,
            blockchain,
            mempool,
            is_mining: Arc::new(RwLock::new(false)),
            revstop,
        }
    }
    
    pub async fn start_mining(&self) -> Result<()> {
        {
            let mut is_mining = self.is_mining.write().await;
            if *is_mining {
                return Ok(());
            }
            *is_mining = true;
        }
        
        info!("Starting mining on address: {}", self.miner_address);
        
        loop {
            // Check if we should stop mining
            {
                let is_mining = self.is_mining.read().await;
                if !*is_mining {
                    break;
                }
            }
            
            // Check RevStop
            {
                let revstop = self.revstop.read().await;
                if revstop.is_enabled() {
                    warn!("🚫 Mining is currently locked by RevStop");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            }
            
            // Try to mine a block
            if let Ok(block) = self.mine_block().await {
                // Add block to blockchain
                {
                    let mut blockchain = self.blockchain.write().await;
                    match blockchain.add_block(block.clone()) {
                        Ok(_) => {
                            info!("✅ Successfully mined block: {}", block.hash);
                            
                            // Remove mined transactions from mempool
                            let mempool = self.mempool.write().await;
                            for tx in &block.transactions {
                                if tx.sender != "NETWORK" {
                                    mempool.remove_transaction(&tx.id);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to add mined block: {}", e);
                        }
                    }
                }
            }
            
            // Small delay to prevent excessive CPU usage
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        info!("Mining stopped");
        Ok(())
    }
    
    pub async fn stop_mining(&self) {
        let mut is_mining = self.is_mining.write().await;
        *is_mining = false;
    }
    
    async fn mine_block(&self) -> Result<Block> {
        // Get transactions from mempool
        let transactions = {
            let mempool = self.mempool.read().await;
            mempool.get_transactions_for_mining(1000, 1_000_000) // max 1000 txs, 1MB
        };
        
        // Create coinbase transaction
        let mining_reward = {
            let blockchain = self.blockchain.read().await;
            blockchain.get_current_mining_reward()
        };
        
        let coinbase_tx = Transaction {
            id: format!("coinbase_{}", Utc::now().timestamp()),
            sender: "NETWORK".to_string(),
            recipient: self.miner_address.clone(),
            amount: mining_reward,
            fee: 0,
            timestamp: Utc::now(),
            signature: None,
            public_key: None,
            nonce: 0,
        };
        
        // Calculate total fees
        let total_fees: u64 = transactions.iter().map(|tx| tx.fee).sum();
        let mut coinbase_with_fees = coinbase_tx.clone();
        coinbase_with_fees.amount += total_fees;
        
        // Combine coinbase with other transactions
        let mut all_transactions = vec![coinbase_with_fees];
        all_transactions.extend(transactions);
        
        // Get previous block hash
        let previous_hash = {
            let blockchain = self.blockchain.read().await;
            blockchain.get_latest_block_hash()
        };
        
        // Create block
        let mut block = Block {
            index: 0, // Will be set properly
            timestamp: Utc::now(),
            previous_hash,
            transactions: all_transactions,
            merkle_root: String::new(), // Will be calculated
            nonce: 0,
            hash: String::new(),
        };
        
        // Set index
        {
            let blockchain = self.blockchain.read().await;
            block.index = blockchain.chain.len() as u64;
        }
        
        // Calculate merkle root
        block.merkle_root = self.calculate_merkle_root(&block.transactions);
        
        // Mine the block (find valid nonce)
        let difficulty = {
            let blockchain = self.blockchain.read().await;
            blockchain.difficulty
        };
        
        let target = "0".repeat(difficulty);
        let mut attempts = 0;
        const MAX_ATTEMPTS_PER_ROUND: u64 = 100_000;
        
        loop {
            // Check if we should stop mining
            {
                let is_mining = self.is_mining.read().await;
                if !*is_mining {
                    return Err(anyhow::anyhow!("Mining stopped"));
                }
            }
            
            block.nonce = rand::thread_rng().gen();
            let hash = self.calculate_block_hash(&block);
            
            if hash.starts_with(&target) {
                block.hash = hash;
                info!("Found valid hash after {} attempts: {}", attempts, block.hash);
                return Ok(block);
            }
            
            attempts += 1;
            
            // Periodically update block with new transactions and timestamp
            if attempts % MAX_ATTEMPTS_PER_ROUND == 0 {
                // Update timestamp
                block.timestamp = Utc::now();
                
                // Get fresh transactions from mempool
                let fresh_transactions = {
                    let mempool = self.mempool.read().await;
                    mempool.get_transactions_for_mining(1000, 1_000_000)
                };
                
                // Update transactions if new ones are available
                if fresh_transactions.len() > block.transactions.len() - 1 {
                    let total_fees: u64 = fresh_transactions.iter().map(|tx| tx.fee).sum();
                    let mut updated_coinbase = coinbase_tx.clone();
                    updated_coinbase.amount += total_fees;
                    
                    block.transactions = vec![updated_coinbase];
                    block.transactions.extend(fresh_transactions);
                    block.merkle_root = self.calculate_merkle_root(&block.transactions);
                }
                
                attempts = 0;
                info!("Mining... difficulty: {}, transactions: {}", difficulty, block.transactions.len());
            }
        }
    }
    
    fn calculate_block_hash(&self, block: &Block) -> String {
        let mut hasher = Sha256::new();
        let data = format!(
            "{}{}{}{}{}{}",
            block.index,
            block.timestamp.timestamp(),
            block.previous_hash,
            block.merkle_root,
            block.nonce,
            block.transactions.len()
        );
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return String::new();
        }
        
        let mut hashes: Vec<String> = transactions
            .iter()
            .map(|tx| {
                let mut hasher = Sha256::new();
                hasher.update(format!("{}{}{}{}", tx.id, tx.sender, tx.recipient, tx.amount));
                format!("{:x}", hasher.finalize())
            })
            .collect();
        
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    format!("{}{}", chunk[0], chunk[0])
                };
                
                let mut hasher = Sha256::new();
                hasher.update(combined);
                next_level.push(format!("{:x}", hasher.finalize()));
            }
            
            hashes = next_level;
        }
        
        hashes.into_iter().next().unwrap_or_default()
    }
    
    pub async fn is_mining(&self) -> bool {
        *self.is_mining.read().await
    }
}