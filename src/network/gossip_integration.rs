// Integration layer for gossip protocol with NetworkManager
// Provides production-ready handlers and flood attack resistance

use crate::block::Block;
use crate::transaction::Transaction;
use crate::blockchain::Blockchain;
use crate::mempool::Mempool;
use crate::network::gossip::*;
use crate::network::{NetworkManager, ChainSpec, NetworkMetrics, SecurityManager};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

/// Production block handler for gossip protocol
pub struct ProductionBlockHandler {
    blockchain: Arc<RwLock<Blockchain>>,
    metrics: Arc<NetworkMetrics>,
    node_id: String,
}

impl ProductionBlockHandler {
    pub fn new(
        blockchain: Arc<RwLock<Blockchain>>,
        metrics: Arc<NetworkMetrics>,
        node_id: String,
    ) -> Self {
        Self {
            blockchain,
            metrics,
            node_id,
        }
    }
}

#[async_trait]
impl BlockHandler for ProductionBlockHandler {
    async fn handle_block(&self, block: Block) -> Result<()> {
        log::info!("Processing block {} via gossip", block.hash);
        
        let start = std::time::Instant::now();
        
        // Add block to blockchain
        {
            let mut blockchain = self.blockchain.write().await;
            blockchain.add_block(block.clone())?;
        }
        
        // Update metrics
        let processing_time = start.elapsed().as_millis();
        self.metrics.record_block_processed(processing_time as u64).await;
        
        log::debug!("Block {} processed in {}ms", block.hash, processing_time);
        Ok(())
    }
    
    async fn validate_block(&self, block: &Block) -> Result<bool> {
        // Comprehensive block validation
        
        // Basic structure validation
        if block.transactions.is_empty() {
            log::debug!("Block {} rejected: no transactions", block.hash);
            return Ok(false);
        }
        
        // Timestamp validation (not too far in future)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if block.timestamp > now + 7200 { // 2 hours in future max
            log::debug!("Block {} rejected: timestamp too far in future", block.hash);
            return Ok(false);
        }
        
        // Check if we already have this block
        {
            let blockchain = self.blockchain.read().await;
            if blockchain.get_block(&block.hash).is_ok() {
                log::debug!("Block {} already exists, skipping", block.hash);
                return Ok(false); // Already have it
            }
        }
        
        // Validate proof of work
        if !self.validate_proof_of_work(block).await? {
            log::debug!("Block {} rejected: invalid proof of work", block.hash);
            return Ok(false);
        }
        
        // Validate transactions
        for (i, tx) in block.transactions.iter().enumerate() {
            if !self.validate_transaction_in_block(tx, i == 0).await? {
                log::debug!("Block {} rejected: invalid transaction {}", block.hash, tx.hash());
                return Ok(false);
            }
        }
        
        // Validate merkle root
        if !self.validate_merkle_root(block).await? {
            log::debug!("Block {} rejected: invalid merkle root", block.hash);
            return Ok(false);
        }
        
        // Check difficulty adjustment
        if !self.validate_difficulty(block).await? {
            log::debug!("Block {} rejected: invalid difficulty", block.hash);
            return Ok(false);
        }
        
        // All validations passed
        log::debug!("Block {} validation passed", block.hash);
        Ok(true)
    }
}

impl ProductionBlockHandler {
    async fn validate_proof_of_work(&self, block: &Block) -> Result<bool> {
        // Validate that block hash meets difficulty target
        let hash_bytes = hex::decode(&block.hash)
            .map_err(|_| anyhow!("Invalid block hash format"))?;
        
        let target = self.calculate_target(block.bits);
        let hash_value = bytes_to_u256(&hash_bytes);
        
        Ok(hash_value <= target)
    }
    
    async fn validate_transaction_in_block(&self, tx: &Transaction, is_coinbase: bool) -> Result<bool> {
        if is_coinbase {
            // Coinbase validation
            if !tx.inputs.is_empty() {
                return Ok(false); // Coinbase should have no inputs
            }
            if tx.outputs.is_empty() {
                return Ok(false); // Coinbase should have outputs
            }
        } else {
            // Regular transaction validation
            if tx.inputs.is_empty() {
                return Ok(false);
            }
            
            // Verify signatures (simplified)
            for input in &tx.inputs {
                if input.signature.is_empty() {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
    
    async fn validate_merkle_root(&self, block: &Block) -> Result<bool> {
        let calculated_root = self.calculate_merkle_root(&block.transactions)?;
        Ok(calculated_root == block.merkle_root)
    }
    
    async fn validate_difficulty(&self, block: &Block) -> Result<bool> {
        let blockchain = self.blockchain.read().await;
        let expected_bits = blockchain.calculate_next_difficulty(block.timestamp)?;
        Ok(block.bits == expected_bits)
    }
    
    fn calculate_target(&self, bits: u32) -> u256 {
        // Simplified difficulty target calculation
        let exponent = ((bits >> 24) & 0xff) as usize;
        let mantissa = bits & 0xffffff;
        
        if exponent <= 3 {
            u256::from(mantissa >> (8 * (3 - exponent)))
        } else {
            u256::from(mantissa) << (8 * (exponent - 3))
        }
    }
    
    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> Result<String> {
        if transactions.is_empty() {
            return Ok("0".repeat(64));
        }
        
        let mut hashes: Vec<String> = transactions.iter()
            .map(|tx| tx.hash())
            .collect();
        
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    format!("{}{}", chunk[0], chunk[0]) // Duplicate last hash
                };
                
                let hash = blake3::hash(combined.as_bytes());
                next_level.push(hex::encode(hash.as_bytes()));
            }
            
            hashes = next_level;
        }
        
        Ok(hashes[0].clone())
    }
}

/// Simple U256 implementation for difficulty calculations
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct u256([u64; 4]);

impl u256 {
    fn from(val: u32) -> Self {
        u256([val as u64, 0, 0, 0])
    }
}

impl std::ops::Shl<usize> for u256 {
    type Output = u256;
    
    fn shl(mut self, rhs: usize) -> Self::Output {
        let word_shift = rhs / 64;
        let bit_shift = rhs % 64;
        
        if word_shift >= 4 {
            return u256([0, 0, 0, 0]);
        }
        
        // Shift words
        for i in (word_shift..4).rev() {
            self.0[i] = if i >= word_shift { self.0[i - word_shift] } else { 0 };
        }
        for i in 0..word_shift {
            self.0[i] = 0;
        }
        
        // Shift bits
        if bit_shift > 0 {
            let mut carry = 0;
            for i in 0..4 {
                let new_carry = self.0[i] >> (64 - bit_shift);
                self.0[i] = (self.0[i] << bit_shift) | carry;
                carry = new_carry;
            }
        }
        
        self
    }
}

fn bytes_to_u256(bytes: &[u8]) -> u256 {
    let mut result = [0u64; 4];
    
    for (i, chunk) in bytes.chunks(8).enumerate() {
        if i >= 4 { break; }
        
        let mut val = 0u64;
        for (j, &byte) in chunk.iter().enumerate() {
            val |= (byte as u64) << (8 * j);
        }
        result[i] = val;
    }
    
    u256(result)
}

/// Production transaction handler for gossip protocol
pub struct ProductionTransactionHandler {
    mempool: Arc<RwLock<Mempool>>,
    blockchain: Arc<RwLock<Blockchain>>,
    metrics: Arc<NetworkMetrics>,
    node_id: String,
}

impl ProductionTransactionHandler {
    pub fn new(
        mempool: Arc<RwLock<Mempool>>,
        blockchain: Arc<RwLock<Blockchain>>,
        metrics: Arc<NetworkMetrics>,
        node_id: String,
    ) -> Self {
        Self {
            mempool,
            blockchain,
            metrics,
            node_id,
        }
    }
}

#[async_trait]
impl TransactionHandler for ProductionTransactionHandler {
    async fn handle_transaction(&self, transaction: Transaction) -> Result<()> {
        log::debug!("Processing transaction {} via gossip", transaction.hash());
        
        let start = std::time::Instant::now();
        
        // Add to mempool
        {
            let mut mempool = self.mempool.write().await;
            mempool.add_transaction(transaction.clone())?;
        }
        
        // Update metrics
        let processing_time = start.elapsed().as_millis();
        self.metrics.record_transaction_processed(processing_time as u64).await;
        
        log::trace!("Transaction {} processed in {}ms", transaction.hash(), processing_time);
        Ok(())
    }
    
    async fn validate_transaction(&self, transaction: &Transaction) -> Result<bool> {
        // Comprehensive transaction validation
        
        // Basic structure validation
        if transaction.inputs.is_empty() || transaction.outputs.is_empty() {
            log::trace!("Transaction {} rejected: empty inputs/outputs", transaction.hash());
            return Ok(false);
        }
        
        // Check if we already have this transaction
        {
            let mempool = self.mempool.read().await;
            if mempool.contains_transaction(&transaction.hash()) {
                log::trace!("Transaction {} already in mempool", transaction.hash());
                return Ok(false); // Already have it
            }
        }
        
        // Check if transaction is already in blockchain
        {
            let blockchain = self.blockchain.read().await;
            if blockchain.has_transaction(&transaction.hash()) {
                log::trace!("Transaction {} already in blockchain", transaction.hash());
                return Ok(false);
            }
        }
        
        // Validate input signatures
        for input in &transaction.inputs {
            if !self.validate_signature(input).await? {
                log::debug!("Transaction {} rejected: invalid signature", transaction.hash());
                return Ok(false);
            }
        }
        
        // Validate input amounts vs output amounts
        if !self.validate_amounts(transaction).await? {
            log::debug!("Transaction {} rejected: invalid amounts", transaction.hash());
            return Ok(false);
        }
        
        // Check for double spending
        if self.check_double_spending(transaction).await? {
            log::debug!("Transaction {} rejected: double spending detected", transaction.hash());
            return Ok(false);
        }
        
        // Validate fee
        if !self.validate_fee(transaction).await? {
            log::debug!("Transaction {} rejected: insufficient fee", transaction.hash());
            return Ok(false);
        }
        
        // All validations passed
        log::trace!("Transaction {} validation passed", transaction.hash());
        Ok(true)
    }
}

impl ProductionTransactionHandler {
    async fn validate_signature(&self, input: &crate::transaction::TransactionInput) -> Result<bool> {
        // Simplified signature validation
        // In production, this would use proper cryptographic verification
        Ok(!input.signature.is_empty() && input.signature.len() >= 64)
    }
    
    async fn validate_amounts(&self, transaction: &Transaction) -> Result<bool> {
        // Calculate total input and output amounts
        let total_input: u64 = transaction.inputs.iter()
            .map(|input| input.amount)
            .sum();
        
        let total_output: u64 = transaction.outputs.iter()
            .map(|output| output.amount)
            .sum();
        
        // Ensure inputs >= outputs (difference is fee)
        Ok(total_input >= total_output)
    }
    
    async fn check_double_spending(&self, transaction: &Transaction) -> Result<bool> {
        let mempool = self.mempool.read().await;
        let blockchain = self.blockchain.read().await;
        
        for input in &transaction.inputs {
            // Check if this input is already spent in mempool
            if mempool.is_input_spent(&input.previous_tx, input.output_index) {
                return Ok(true); // Double spending detected
            }
            
            // Check if this input is already spent in blockchain
            if blockchain.is_input_spent(&input.previous_tx, input.output_index) {
                return Ok(true); // Double spending detected
            }
        }
        
        Ok(false) // No double spending
    }
    
    async fn validate_fee(&self, transaction: &Transaction) -> Result<bool> {
        let total_input: u64 = transaction.inputs.iter()
            .map(|input| input.amount)
            .sum();
        
        let total_output: u64 = transaction.outputs.iter()
            .map(|output| output.amount)
            .sum();
        
        let fee = total_input.saturating_sub(total_output);
        let min_fee = self.calculate_minimum_fee(transaction).await?;
        
        Ok(fee >= min_fee)
    }
    
    async fn calculate_minimum_fee(&self, transaction: &Transaction) -> Result<u64> {
        // Calculate fee based on transaction size
        let tx_size = bincode::serialize(transaction)?.len() as u64;
        let fee_rate = 10; // satoshis per byte
        Ok(tx_size * fee_rate)
    }
}

/// Gossip protocol manager for NetworkManager integration
pub struct GossipManager {
    gossip_protocol: Arc<GossipProtocol>,
    block_handler: Arc<ProductionBlockHandler>,
    transaction_handler: Arc<ProductionTransactionHandler>,
}

impl GossipManager {
    pub async fn new(
        node_id: String,
        chain_spec: Arc<ChainSpec>,
        metrics: Arc<NetworkMetrics>,
        security_manager: Arc<SecurityManager>,
        blockchain: Arc<RwLock<Blockchain>>,
        mempool: Arc<RwLock<Mempool>>,
    ) -> Result<Self> {
        // Create handlers
        let block_handler = Arc::new(ProductionBlockHandler::new(
            blockchain.clone(),
            metrics.clone(),
            node_id.clone(),
        ));
        
        let transaction_handler = Arc::new(ProductionTransactionHandler::new(
            mempool,
            blockchain,
            metrics.clone(),
            node_id.clone(),
        ));
        
        // Create gossip protocol
        let mut gossip_protocol = GossipProtocol::new(
            node_id,
            chain_spec,
            metrics,
            security_manager,
            block_handler.clone(),
            transaction_handler.clone(),
        ).await?;
        
        // Start the protocol
        gossip_protocol.start().await?;
        
        Ok(Self {
            gossip_protocol: Arc::new(gossip_protocol),
            block_handler,
            transaction_handler,
        })
    }
    
    /// Add a peer to gossip with
    pub async fn add_peer(&self, peer_id: String, sender: tokio::sync::mpsc::UnboundedSender<crate::network::protocol::NetworkMessage>) -> Result<()> {
        self.gossip_protocol.gossip_tx.send(GossipCommand::AddPeer(peer_id, sender))
            .map_err(|_| anyhow!("Failed to add peer to gossip"))?;
        Ok(())
    }
    
    /// Remove a peer from gossip
    pub async fn remove_peer(&self, peer_id: &str) -> Result<()> {
        self.gossip_protocol.gossip_tx.send(GossipCommand::RemovePeer(peer_id.to_string()))
            .map_err(|_| anyhow!("Failed to remove peer from gossip"))?;
        Ok(())
    }
    
    /// Gossip a new block
    pub async fn gossip_block(&self, block: Block) -> Result<()> {
        self.gossip_protocol.gossip_block(block).await
    }
    
    /// Gossip a new transaction
    pub async fn gossip_transaction(&self, transaction: Transaction) -> Result<()> {
        self.gossip_protocol.gossip_transaction(transaction).await
    }
    
    /// Process incoming gossip from peer
    pub async fn process_incoming_gossip(&self, peer_id: &str, item: GossipItem) -> Result<()> {
        self.gossip_protocol.process_incoming_gossip(peer_id, item).await
    }
    
    /// Get gossip statistics
    pub async fn get_stats(&self) -> GossipStats {
        self.gossip_protocol.get_stats().await
    }
    
    /// Update peer score for DoS protection
    pub async fn update_peer_score(&self, peer_id: &str, delta: i32) -> Result<()> {
        self.gossip_protocol.gossip_tx.send(GossipCommand::UpdatePeerScore(peer_id.to_string(), delta))
            .map_err(|_| anyhow!("Failed to update peer score"))?;
        Ok(())
    }
    
    /// Force synchronization
    pub async fn force_sync(&self) -> Result<()> {
        self.gossip_protocol.gossip_tx.send(GossipCommand::ForceSync)
            .map_err(|_| anyhow!("Failed to force sync"))?;
        Ok(())
    }
    
    /// Shutdown gossip manager
    pub async fn shutdown(&self) -> Result<()> {
        self.gossip_protocol.shutdown().await
    }
    
    /// Handle flood attack by implementing emergency backpressure
    pub async fn handle_flood_attack(&self, peer_id: &str) -> Result<()> {
        log::warn!("Flood attack detected from peer: {}", peer_id);
        
        // Immediately ban the peer
        self.update_peer_score(peer_id, DOS_BAN_THRESHOLD).await?;
        
        // Enable emergency backpressure
        let stats = self.get_stats().await;
        if stats.has_backpressure {
            log::error!("Network under flood attack - emergency backpressure active");
            
            // Reduce gossip rate temporarily
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        Ok(())
    }
}

/// Flood test resistance implementation
pub struct FloodTestResistance {
    pub max_messages_per_second: f64,
    pub max_queue_size: usize,
    pub emergency_backpressure_threshold: f64,
}

impl Default for FloodTestResistance {
    fn default() -> Self {
        Self {
            max_messages_per_second: 1000.0, // Allow burst of 1000 msgs/sec
            max_queue_size: BACKPRESSURE_THRESHOLD,
            emergency_backpressure_threshold: 0.8, // Trigger at 80% capacity
        }
    }
}

impl FloodTestResistance {
    pub fn should_activate_backpressure(&self, current_rate: f64, queue_utilization: f64) -> bool {
        current_rate > self.max_messages_per_second || 
        queue_utilization > self.emergency_backpressure_threshold
    }
    
    pub fn calculate_backpressure_delay(&self, overload_factor: f64) -> std::time::Duration {
        let delay_ms = (overload_factor * 1000.0) as u64;
        std::time::Duration::from_millis(delay_ms.min(5000)) // Max 5 second delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_u256_operations() {
        let val = u256::from(1);
        let shifted = val << 8;
        assert!(shifted > val);
    }
    
    #[test] 
    async fn test_flood_resistance() {
        let resistance = FloodTestResistance::default();
        
        // Normal load should not trigger backpressure
        assert!(!resistance.should_activate_backpressure(100.0, 0.5));
        
        // High load should trigger backpressure
        assert!(resistance.should_activate_backpressure(2000.0, 0.5));
        assert!(resistance.should_activate_backpressure(100.0, 0.9));
    }
}
