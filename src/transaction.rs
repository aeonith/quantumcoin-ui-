use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TxType { Transfer, Coinbase }

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: i64,
    pub tx_type: TxType,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: from.into(),
            to: to.into(),
            amount,
            timestamp: Utc::now().timestamp(),
            tx_type: TxType::Transfer,
        }
    }

    pub fn coinbase(recipient: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: "GENESIS".into(),
            to: recipient.into(),
            amount: 25,
            timestamp: Utc::now().timestamp(),
            tx_type: TxType::Coinbase,
        }
    }
}