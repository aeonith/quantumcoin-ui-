use serde::{Serialize, Deserialize};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: String,
    pub signature: Option<String>,
    pub tx_type: TransactionType,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, tx_type: TransactionType) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();
        Transaction {
            id,
            from,
            to,
            amount,
            timestamp,
            signature: None,
            tx_type,
        }
    }
}