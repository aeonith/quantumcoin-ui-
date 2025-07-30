use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::transaction::Transaction;
use crate::database::{DatabaseError, TransactionRecord, TransactionStatus};

#[derive(Clone)]
pub struct MockDatabase {
    balances: Arc<RwLock<HashMap<String, u64>>>,
    transactions: Arc<RwLock<Vec<TransactionRecord>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        let mut balances = HashMap::new();
        
        // Add some demo balances
        balances.insert("demo_wallet_1".to_string(), 100000);
        balances.insert("demo_wallet_2".to_string(), 50000);
        balances.insert("miner_address".to_string(), 1000000);
        
        Self {
            balances: Arc::new(RwLock::new(balances)),
            transactions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<u64, DatabaseError> {
        let balances = self.balances.read();
        Ok(*balances.get(address).unwrap_or(&0))
    }

    pub async fn add_transaction_batch(&self, transactions: &[Transaction]) -> Result<u64, DatabaseError> {
        let mut tx_list = self.transactions.write();
        let mut processed = 0u64;

        for transaction in transactions {
            let record = TransactionRecord {
                id: transaction.id.clone(),
                block_hash: None,
                block_height: None,
                sender: transaction.sender.clone(),
                recipient: transaction.recipient.clone(),
                amount: transaction.amount,
                fee: transaction.fee,
                status: TransactionStatus::Pending,
                timestamp: transaction.timestamp,
                confirmations: 0,
            };
            
            tx_list.push(record);
            processed += 1;
        }

        Ok(processed)
    }

    pub async fn validate_transaction_fast(&self, tx: &Transaction) -> Result<bool, DatabaseError> {
        let sender_balance = self.get_balance(&tx.sender).await?;
        Ok(sender_balance >= tx.total_cost())
    }

    pub async fn get_transaction_history(
        &self,
        address: &str,
        limit: u32,
        _offset: u32,
    ) -> Result<Vec<TransactionRecord>, DatabaseError> {
        let transactions = self.transactions.read();
        let filtered: Vec<TransactionRecord> = transactions
            .iter()
            .filter(|tx| tx.sender == address || tx.recipient == address)
            .take(limit as usize)
            .cloned()
            .collect();
        
        Ok(filtered)
    }

    pub async fn get_database_stats(&self) -> Result<HashMap<String, u64>, DatabaseError> {
        let mut stats = HashMap::new();
        stats.insert("total_transactions".to_string(), self.transactions.read().len() as u64);
        stats.insert("total_addresses".to_string(), self.balances.read().len() as u64);
        Ok(stats)
    }
}
