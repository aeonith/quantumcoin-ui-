use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use crate::transaction::{Transaction, SignedTransaction};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolEntry {
    pub transaction: SignedTransaction,
    pub received_time: DateTime<Utc>,
    pub fee_per_byte: f64,
    pub priority: u64,
}

impl MempoolEntry {
    pub fn new(transaction: SignedTransaction) -> Self {
        let fee = transaction.calculate_fee(&HashMap::new()).unwrap_or(0);
        let size = bincode::serialize(&transaction).map(|data| data.len()).unwrap_or(1);
        let fee_per_byte = if size > 0 { fee as f64 / size as f64 } else { 0.0 };
        
        Self {
            transaction,
            received_time: Utc::now(),
            fee_per_byte,
            priority: fee,
        }
    }

    pub fn is_expired(&self, max_age: Duration) -> bool {
        Utc::now() - self.received_time > max_age
    }
}

pub struct Mempool {
    transactions: HashMap<String, MempoolEntry>,
    max_size: usize,
    max_transaction_age: Duration,
    min_fee_per_byte: f64,
}

impl Mempool {
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: HashMap::new(),
            max_size,
            max_transaction_age: Duration::hours(24),
            min_fee_per_byte: 0.0001, // Minimum fee per byte
        }
    }

    pub fn add_transaction(&mut self, transaction: SignedTransaction) -> Result<()> {
        // Check if transaction already exists
        if self.transactions.contains_key(&transaction.id) {
            return Err(anyhow!("Transaction already in mempool"));
        }

        let entry = MempoolEntry::new(transaction);

        // Check minimum fee
        if entry.fee_per_byte < self.min_fee_per_byte {
            return Err(anyhow!(
                "Transaction fee too low: {} < {}",
                entry.fee_per_byte,
                self.min_fee_per_byte
            ));
        }

        // If mempool is full, try to evict lowest fee transaction
        if self.transactions.len() >= self.max_size {
            self.evict_lowest_fee_transaction()?;
        }

        let tx_id = entry.transaction.id.clone();
        self.transactions.insert(tx_id, entry);
        
        Ok(())
    }

    pub fn remove_transaction(&mut self, tx_id: &str) -> Option<MempoolEntry> {
        self.transactions.remove(tx_id)
    }

    pub fn get_transaction(&self, tx_id: &str) -> Option<&MempoolEntry> {
        self.transactions.get(tx_id)
    }

    pub fn get_transactions_by_fee(&self, limit: usize) -> Vec<&MempoolEntry> {
        let mut entries: Vec<&MempoolEntry> = self.transactions.values().collect();
        entries.sort_by(|a, b| b.fee_per_byte.partial_cmp(&a.fee_per_byte).unwrap_or(std::cmp::Ordering::Equal));
        entries.into_iter().take(limit).collect()
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let expired_keys: Vec<String> = self.transactions
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.max_transaction_age))
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.transactions.remove(&key);
        }
        
        count
    }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn clear(&mut self) {
        self.transactions.clear();
    }

    pub fn contains(&self, tx_id: &str) -> bool {
        self.transactions.contains_key(tx_id)
    }

    pub fn get_all_transactions(&self) -> Vec<&SignedTransaction> {
        self.transactions.values().map(|entry| &entry.transaction).collect()
    }

    pub fn get_transactions_for_mining(&self, max_count: usize, max_size_bytes: usize) -> Vec<SignedTransaction> {
        let mut selected = Vec::new();
        let mut total_size = 0;
        
        // Get transactions sorted by fee per byte (highest first)
        let mut entries: Vec<&MempoolEntry> = self.transactions.values().collect();
        entries.sort_by(|a, b| b.fee_per_byte.partial_cmp(&a.fee_per_byte).unwrap_or(std::cmp::Ordering::Equal));
        
        for entry in entries {
            if selected.len() >= max_count {
                break;
            }
            
            let tx_size = bincode::serialize(&entry.transaction).map(|data| data.len()).unwrap_or(0);
            if total_size + tx_size > max_size_bytes {
                break;
            }
            
            selected.push(entry.transaction.clone());
            total_size += tx_size;
        }
        
        selected
    }

    fn evict_lowest_fee_transaction(&mut self) -> Result<()> {
        if self.transactions.is_empty() {
            return Err(anyhow!("Cannot evict from empty mempool"));
        }

        // Find transaction with lowest fee per byte
        let lowest_fee_tx = self.transactions
            .iter()
            .min_by(|a, b| a.1.fee_per_byte.partial_cmp(&b.1.fee_per_byte).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(key, _)| key.clone());

        if let Some(tx_id) = lowest_fee_tx {
            self.transactions.remove(&tx_id);
        }

        Ok(())
    }

    pub fn estimate_fee_for_priority(&self, target_confirmations: u32) -> f64 {
        if self.transactions.is_empty() {
            return self.min_fee_per_byte;
        }

        let mut fees: Vec<f64> = self.transactions.values().map(|entry| entry.fee_per_byte).collect();
        fees.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Estimate based on target confirmations
        let position = match target_confirmations {
            1 => 0.1,  // Top 10% for next block
            2..=3 => 0.25, // Top 25% for 2-3 blocks
            4..=6 => 0.5,  // Top 50% for 4-6 blocks
            _ => 0.75,     // Top 75% for longer confirmation
        };

        let index = ((fees.len() as f64 * position) as usize).min(fees.len().saturating_sub(1));
        fees.get(index).copied().unwrap_or(self.min_fee_per_byte)
    }

    pub fn get_mempool_stats(&self) -> MempoolStats {
        if self.transactions.is_empty() {
            return MempoolStats::default();
        }

        let fees: Vec<f64> = self.transactions.values().map(|entry| entry.fee_per_byte).collect();
        let total_fees: f64 = fees.iter().sum();
        let avg_fee = total_fees / fees.len() as f64;
        
        let mut sorted_fees = fees.clone();
        sorted_fees.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median_fee = if sorted_fees.is_empty() {
            0.0
        } else if sorted_fees.len() % 2 == 0 {
            let mid = sorted_fees.len() / 2;
            (sorted_fees[mid - 1] + sorted_fees[mid]) / 2.0
        } else {
            sorted_fees[sorted_fees.len() / 2]
        };

        MempoolStats {
            transaction_count: self.transactions.len(),
            avg_fee_per_byte: avg_fee,
            median_fee_per_byte: median_fee,
            min_fee_per_byte: sorted_fees.first().copied().unwrap_or(0.0),
            max_fee_per_byte: sorted_fees.last().copied().unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStats {
    pub transaction_count: usize,
    pub avg_fee_per_byte: f64,
    pub median_fee_per_byte: f64,
    pub min_fee_per_byte: f64,
    pub max_fee_per_byte: f64,
}

impl Default for MempoolStats {
    fn default() -> Self {
        Self {
            transaction_count: 0,
            avg_fee_per_byte: 0.0,
            median_fee_per_byte: 0.0,
            min_fee_per_byte: 0.0,
            max_fee_per_byte: 0.0,
        }
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(10000) // Default to 10k transactions
    }
}

// Thread-safe mempool wrapper
pub type SharedMempool = Arc<RwLock<Mempool>>;

pub fn create_shared_mempool(max_size: usize) -> SharedMempool {
    Arc::new(RwLock::new(Mempool::new(max_size)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TransactionInput, TransactionOutput};

    fn create_test_transaction(id: &str) -> SignedTransaction {
        SignedTransaction::new(
            vec![TransactionInput {
                previous_output: "test_input".to_string(),
                script_sig: vec![],
                sequence: 0,
            }],
            vec![TransactionOutput {
                value: 1000,
                script_pubkey: vec![],
                address: "test_address".to_string(),
            }],
            0,
        )
    }

    #[test]
    fn test_mempool_add_remove() {
        let mut mempool = Mempool::new(100);
        let tx = create_test_transaction("test_tx_1");
        let tx_id = tx.id.clone();

        assert!(mempool.add_transaction(tx).is_ok());
        assert_eq!(mempool.size(), 1);
        assert!(mempool.contains(&tx_id));

        let removed = mempool.remove_transaction(&tx_id);
        assert!(removed.is_some());
        assert_eq!(mempool.size(), 0);
        assert!(!mempool.contains(&tx_id));
    }

    #[test]
    fn test_mempool_cleanup_expired() {
        let mut mempool = Mempool::new(100);
        mempool.max_transaction_age = Duration::seconds(1);
        
        let tx = create_test_transaction("test_tx_2");
        mempool.add_transaction(tx).unwrap();
        
        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let expired_count = mempool.cleanup_expired();
        assert_eq!(expired_count, 1);
        assert_eq!(mempool.size(), 0);
    }
}
