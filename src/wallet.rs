use uuid::Uuid;
use crate::transaction::Transaction;
use crate::blockchain::Blockchain;

pub struct Wallet {
    pub address: String,
}

impl Wallet {
    /// Generate a brand-new wallet address
    pub fn generate() -> Self {
        Wallet {
            address: Uuid::new_v4().to_string()
        }
    }

    /// Derive balance by summing chain transactions
    pub fn balance(&self, chain: &Blockchain) -> u64 {
        chain.chain.iter()
            .flat_map(|b| b.transactions.iter())
            .filter_map(|tx| {
                if tx.to == self.address {
                    Some(tx.amount as i64)
                } else if tx.from == self.address {
                    Some(-(tx.amount as i64))
                } else {
                    None
                }
            })
            .sum::<i64>()
            .max(0) as u64
    }

    /// Create a new outgoing transaction
    pub fn create_transaction(&self, to: &str, amount: u64) -> Transaction {
        Transaction::new(&self.address, to, amount)
    }
}