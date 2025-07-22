use crate::transaction::Transaction;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub address: String,
    pub password: String,
}

impl Wallet {
    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn verify_password(&self, input: &str) -> bool {
        self.password == input
    }

    pub fn create_transaction(&self, to: String, amount: u64) -> Transaction {
        Transaction {
            from: self.address.clone(),
            to,
            amount,
        }
    }
}