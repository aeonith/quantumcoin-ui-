use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn get_balance(&self, address: &str) -> u64 {
        self.transactions
            .iter()
            .fold(0i64, |acc, tx| {
                if tx.to == address {
                    acc + tx.amount as i64
                } else if tx.from == address {
                    acc - tx.amount as i64
                } else {
                    acc
                }
            })
            .max(0) as u64
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }
}