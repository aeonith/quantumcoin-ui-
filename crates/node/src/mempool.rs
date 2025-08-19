// QuantumCoin Mempool - Bitcoin-level Transaction Pool Management

use crate::{Tx, Validator, Chain};
use anyhow::{Result, anyhow};
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Mempool {
    transactions: HashMap<String, MempoolTx>,
    by_fee_rate: BTreeMap<u64, Vec<String>>, // Fee rate -> tx ids
    by_arrival: BTreeMap<u64, String>,        // Timestamp -> tx id
    validator: Validator,
    max_size: usize,
    max_tx_count: usize,
}

#[derive(Clone, Debug)]
pub struct MempoolTx {
    pub tx: Tx,
    pub arrival_time: u64,
    pub fee_rate: u64,        // Satoshis per virtual byte
    pub size: usize,
    pub ancestors: Vec<String>, // Dependencies
    pub descendants: Vec<String>, // Dependents
}

impl Mempool {
    pub fn new(chain: Chain) -> Self {
        Self {
            transactions: HashMap::new(),
            by_fee_rate: BTreeMap::new(),
            by_arrival: BTreeMap::new(),
            validator: Validator::new(chain),
            max_size: 300_000_000, // 300MB like Bitcoin
            max_tx_count: 100_000,  // 100k transactions max
        }
    }
    
    /// Add transaction to mempool with Bitcoin-level validation
    pub fn add_transaction(&mut self, tx: Tx) -> Result<String> {
        // 1. Basic validation
        self.validator.validate_transaction(&tx)?;
        
        // 2. Check if already in mempool
        let txid = self.calculate_txid(&tx);
        if self.transactions.contains_key(&txid) {
            return Err(anyhow!("Transaction already in mempool"));
        }
        
        // 3. Check mempool limits
        self.enforce_size_limits()?;
        
        // 4. Calculate fee rate
        let size = self.estimate_tx_size(&tx);
        let fee_rate = tx.fee / size as u64;
        
        // 5. Replace-by-fee (RBF) logic
        self.handle_replace_by_fee(&tx, &txid, fee_rate)?;
        
        // 6. Add to mempool
        let mempool_tx = MempoolTx {
            tx: tx.clone(),
            arrival_time: self.current_time(),
            fee_rate,
            size,
            ancestors: Vec::new(),
            descendants: Vec::new(),
        };
        
        self.transactions.insert(txid.clone(), mempool_tx);
        
        // Index by fee rate for mining prioritization
        self.by_fee_rate.entry(fee_rate).or_insert_with(Vec::new).push(txid.clone());
        self.by_arrival.insert(self.current_time(), txid.clone());
        
        println!("✅ Transaction added to mempool: {} (fee rate: {} sat/vB)", 
                 &txid[..16], fee_rate);
        
        Ok(txid)
    }
    
    /// Get transactions for block template (highest fee first)
    pub fn get_block_template(&self, max_block_size: usize) -> Vec<Tx> {
        let mut selected = Vec::new();
        let mut total_size = 0;
        
        // Select transactions by fee rate (highest first)
        for (_fee_rate, txids) in self.by_fee_rate.iter().rev() {
            for txid in txids {
                if let Some(mempool_tx) = self.transactions.get(txid) {
                    if total_size + mempool_tx.size <= max_block_size {
                        selected.push(mempool_tx.tx.clone());
                        total_size += mempool_tx.size;
                    }
                }
            }
        }
        
        selected
    }
    
    /// Remove transactions (after block confirmation)
    pub fn remove_transactions(&mut self, txids: &[String]) {
        for txid in txids {
            if let Some(mempool_tx) = self.transactions.remove(txid) {
                // Remove from indexes
                if let Some(txids) = self.by_fee_rate.get_mut(&mempool_tx.fee_rate) {
                    txids.retain(|id| id != txid);
                    if txids.is_empty() {
                        self.by_fee_rate.remove(&mempool_tx.fee_rate);
                    }
                }
                self.by_arrival.retain(|_time, id| id != txid);
            }
        }
    }
    
    /// Evict old/low-fee transactions
    pub fn cleanup_expired(&mut self) {
        let now = self.current_time();
        let max_age = 86400; // 24 hours
        
        let expired: Vec<String> = self.by_arrival.iter()
            .filter(|(time, _)| now - **time > max_age)
            .map(|(_, txid)| txid.clone())
            .collect();
        
        self.remove_transactions(&expired);
        
        if !expired.is_empty() {
            println!("🧹 Cleaned {} expired transactions from mempool", expired.len());
        }
    }
    
    /// Mempool statistics for monitoring
    pub fn get_stats(&self) -> MempoolStats {
        let total_size: usize = self.transactions.values().map(|tx| tx.size).sum();
        let avg_fee_rate = if !self.transactions.is_empty() {
            self.transactions.values().map(|tx| tx.fee_rate).sum::<u64>() / self.transactions.len() as u64
        } else {
            0
        };
        
        MempoolStats {
            tx_count: self.transactions.len(),
            total_size_bytes: total_size,
            avg_fee_rate,
            min_fee_rate: self.by_fee_rate.keys().next().copied().unwrap_or(0),
            max_fee_rate: self.by_fee_rate.keys().last().copied().unwrap_or(0),
        }
    }
    
    // Helper methods
    fn calculate_txid(&self, tx: &Tx) -> String {
        let tx_bytes = serde_json::to_vec(tx).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&tx_bytes);
        hex::encode(hasher.finalize())
    }
    
    fn estimate_tx_size(&self, tx: &Tx) -> usize {
        // Simplified size estimation
        250 + tx.data.len() // Base size + data
    }
    
    fn current_time(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
    
    fn enforce_size_limits(&mut self) -> Result<()> {
        while self.transactions.len() > self.max_tx_count {
            // Evict lowest fee rate transaction
            if let Some((_, txids)) = self.by_fee_rate.iter().next() {
                if let Some(txid) = txids.first() {
                    let txid = txid.clone();
                    self.remove_transactions(&[txid]);
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_replace_by_fee(&mut self, new_tx: &Tx, txid: &str, fee_rate: u64) -> Result<()> {
        // Check for existing transaction from same sender with same nonce
        for (existing_txid, existing_tx) in &self.transactions {
            if existing_tx.tx.from == new_tx.from && existing_tx.tx.nonce == new_tx.nonce {
                // RBF: new transaction must have higher fee rate
                if fee_rate > existing_tx.fee_rate {
                    println!("🔄 Replacing transaction {} with higher fee", &existing_txid[..16]);
                    self.remove_transactions(&[existing_txid.clone()]);
                    return Ok(());
                } else {
                    return Err(anyhow!("RBF requires higher fee rate"));
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub tx_count: usize,
    pub total_size_bytes: usize,
    pub avg_fee_rate: u64,
    pub min_fee_rate: u64,
    pub max_fee_rate: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mempool_add_transaction() {
        let chain = Chain::new_genesis();
        let mut mempool = Mempool::new(chain);
        
        let tx = Tx {
            nonce: 1,
            from: "qc1test123".to_string(),
            to: "qc1test456".to_string(),
            value: 1000000,
            fee: 1000,
            data: "".to_string(),
        };
        
        let result = mempool.add_transaction(tx);
        assert!(result.is_ok());
        assert_eq!(mempool.transactions.len(), 1);
    }
    
    #[test]
    fn test_fee_prioritization() {
        let chain = Chain::new_genesis();
        let mut mempool = Mempool::new(chain);
        
        // Add low fee transaction
        let low_fee_tx = Tx {
            nonce: 1,
            from: "qc1test123".to_string(),
            to: "qc1test456".to_string(),
            value: 1000000,
            fee: 1000,
            data: "".to_string(),
        };
        
        // Add high fee transaction
        let high_fee_tx = Tx {
            nonce: 2,
            from: "qc1test789".to_string(),
            to: "qc1test012".to_string(),
            value: 1000000,
            fee: 10000,
            data: "".to_string(),
        };
        
        mempool.add_transaction(low_fee_tx).unwrap();
        mempool.add_transaction(high_fee_tx).unwrap();
        
        let template = mempool.get_block_template(1000000);
        // High fee transaction should be first
        assert_eq!(template[0].fee, 10000);
    }
}
