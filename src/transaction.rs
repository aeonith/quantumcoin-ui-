use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: u64) -> Self {
        Transaction {
            id: Uuid::new_v4(),
            from: from.to_string(),
            to: to.to_string(),
            amount,
            timestamp: Utc::now(),
        }
    }
}