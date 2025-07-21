use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub timestamp: i64,
    pub tx_type: String,
}

impl Transaction {
    pub fn new(sender: &str, recipient: &str, amount: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            amount,
            timestamp: Utc::now().timestamp(),
            tx_type: "transfer".into(),
        }
    }
}