use crate::models::Transaction;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn load_or_create() -> Self {
        if Path::new("blockchain.json").exists() {
            let data = fs::read_to_string("blockchain.json").unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            Blockchain {
                transactions: vec![]
            }
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
        let json = serde_json::to_string(&self).unwrap();
        fs::write("blockchain.json", json).unwrap();
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0;
        for tx in &self.transactions {
            if tx.recipient == address {
                balance += tx.amount;
            } else if tx.sender == address {
                balance -= tx.amount;
            }
        }
        balance
    }
}