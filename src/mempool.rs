// src/mempool.rs

use crate::transaction::Transaction;

#[derive(Default)]
pub struct Mempool {
    pub transactions: Vec<Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Mempool {
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }

    pub fn get_all(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn clear(&mut self) {
        self.transactions.clear();
    }

    pub fn has_pending(&self) -> bool {
        !self.transactions.is_empty()
    }
}