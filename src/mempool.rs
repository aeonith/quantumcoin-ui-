use crate::{Transaction, TransactionError};
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MempoolTransaction {
    pub transaction: Transaction,
    pub timestamp: u64,
    pub fee_per_byte: u64,
    pub dependencies: Vec<String>, // Transaction IDs this depends on
}

pub struct Mempool {
    transactions: Arc<RwLock<HashMap<String, MempoolTransaction>>>,
    fee_index: Arc<RwLock<BTreeMap<u64, Vec<String>>>>, // fee_per_byte -> tx_ids
    max_size: usize,
    max_age: u64, // seconds
}

impl Mempool {
    pub fn new(max_size: usize, max_age_hours: u64) -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            fee_index: Arc::new(RwLock::new(BTreeMap::new())),
            max_size,
            max_age: max_age_hours * 3600,
        }
    }
    
    pub fn add_transaction(&self, tx: Transaction) -> Result<(), TransactionError> {
        let tx_size = self.estimate_transaction_size(&tx);
        let fee_per_byte = tx.fee / tx_size;
        
        let mempool_tx = MempoolTransaction {
            transaction: tx.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            fee_per_byte,
            dependencies: self.find_dependencies(&tx),
        };
        
        {
            let mut transactions = self.transactions.write();
            let mut fee_index = self.fee_index.write();
            
            // Check if we're at capacity and this tx has lower fee than lowest
            if transactions.len() >= self.max_size {
                if let Some((lowest_fee, _)) = fee_index.iter().next() {
                    if fee_per_byte <= *lowest_fee {
                        return Err(TransactionError::FeeTooLow);
                    }
                    // Remove lowest fee transaction
                    self.remove_lowest_fee_transaction(&mut transactions, &mut fee_index);
                }
            }
            
            // Add new transaction
            transactions.insert(tx.id.clone(), mempool_tx);
            fee_index.entry(fee_per_byte).or_insert_with(Vec::new).push(tx.id.clone());
        }
        
        Ok(())
    }
    
    pub fn remove_transaction(&self, tx_id: &str) -> Option<Transaction> {
        let mut transactions = self.transactions.write();
        let mut fee_index = self.fee_index.write();
        
        if let Some(mempool_tx) = transactions.remove(tx_id) {
            // Remove from fee index
            if let Some(tx_list) = fee_index.get_mut(&mempool_tx.fee_per_byte) {
                tx_list.retain(|id| id != tx_id);
                if tx_list.is_empty() {
                    fee_index.remove(&mempool_tx.fee_per_byte);
                }
            }
            Some(mempool_tx.transaction)
        } else {
            None
        }
    }
    
    pub fn get_transactions_for_mining(&self, max_count: usize, max_size: usize) -> Vec<Transaction> {
        let transactions = self.transactions.read();
        let fee_index = self.fee_index.read();
        
        let mut selected = Vec::new();
        let mut total_size = 0;
        let mut selected_ids = std::collections::HashSet::new();
        
        // Start with highest fee transactions
        for (_, tx_ids) in fee_index.iter().rev() {
            for tx_id in tx_ids {
                if selected.len() >= max_count {
                    break;
                }
                
                if let Some(mempool_tx) = transactions.get(tx_id) {
                    // Check dependencies are satisfied
                    let deps_satisfied = mempool_tx.dependencies.iter()
                        .all(|dep_id| selected_ids.contains(dep_id));
                    
                    if deps_satisfied {
                        let tx_size = self.estimate_transaction_size(&mempool_tx.transaction);
                        if total_size + tx_size <= max_size {
                            selected.push(mempool_tx.transaction.clone());
                            selected_ids.insert(tx_id.clone());
                            total_size += tx_size;
                        }
                    }
                }
            }
            if selected.len() >= max_count {
                break;
            }
        }
        
        selected
    }
    
    pub fn cleanup_expired(&self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut transactions = self.transactions.write();
        let mut fee_index = self.fee_index.write();
        
        let expired_ids: Vec<String> = transactions
            .iter()
            .filter(|(_, mempool_tx)| now - mempool_tx.timestamp > self.max_age)
            .map(|(id, _)| id.clone())
            .collect();
        
        for tx_id in expired_ids {
            if let Some(mempool_tx) = transactions.remove(&tx_id) {
                if let Some(tx_list) = fee_index.get_mut(&mempool_tx.fee_per_byte) {
                    tx_list.retain(|id| id != &tx_id);
                    if tx_list.is_empty() {
                        fee_index.remove(&mempool_tx.fee_per_byte);
                    }
                }
            }
        }
    }
    
    pub fn contains(&self, tx_id: &str) -> bool {
        self.transactions.read().contains_key(tx_id)
    }
    
    pub fn get_transaction(&self, tx_id: &str) -> Option<Transaction> {
        self.transactions.read().get(tx_id).map(|mt| mt.transaction.clone())
    }
    
    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        self.transactions.read()
            .values()
            .map(|mt| mt.transaction.clone())
            .collect()
    }
    
    pub fn size(&self) -> usize {
        self.transactions.read().len()
    }
    
    pub fn get_fee_stats(&self) -> (u64, u64, u64) { // min, avg, max fee per byte
        let fee_index = self.fee_index.read();
        
        if fee_index.is_empty() {
            return (0, 0, 0);
        }
        
        let fees: Vec<u64> = fee_index.keys().copied().collect();
        let min_fee = *fees.first().unwrap();
        let max_fee = *fees.last().unwrap();
        let avg_fee = fees.iter().sum::<u64>() / fees.len() as u64;
        
        (min_fee, avg_fee, max_fee)
    }
    
    fn estimate_transaction_size(&self, tx: &Transaction) -> u64 {
        // Rough estimate: base size + signature size + public key size
        let base_size = 100; // Basic transaction data
        let sig_size = tx.signature.as_ref().map(|s| s.len()).unwrap_or(0) as u64;
        let pk_size = tx.public_key.as_ref().map(|pk| pk.len()).unwrap_or(0) as u64;
        
        base_size + sig_size + pk_size
    }
    
    fn find_dependencies(&self, tx: &Transaction) -> Vec<String> {
        // For UTXO-based systems, we'd check if any inputs depend on outputs
        // from other transactions in the mempool
        // For now, simplified implementation
        Vec::new()
    }
    
    fn remove_lowest_fee_transaction(
        &self,
        transactions: &mut HashMap<String, MempoolTransaction>,
        fee_index: &mut BTreeMap<u64, Vec<String>>,
    ) {
        if let Some((lowest_fee, tx_ids)) = fee_index.iter_mut().next() {
            if let Some(tx_id) = tx_ids.pop() {
                transactions.remove(&tx_id);
                if tx_ids.is_empty() {
                    let lowest_fee = *lowest_fee;
                    fee_index.remove(&lowest_fee);
                }
            }
        }
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(10000, 24) // 10k transactions, 24 hour max age
    }
}
