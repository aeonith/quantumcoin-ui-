use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// The only two transaction kinds we need right now.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TxType {
    Transfer,   // user → user
    Coinbase,   // miner reward
}

/// A single movement of value on-chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: i64,
    pub tx_type: TxType,
}

impl Transaction {
    /// Creates a normal user-to-user transfer.
    pub fn new(from: &str, to: &str, amount: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            amount,
            timestamp: Utc::now().timestamp(),
            tx_type: TxType::Transfer,
        }
    }

    /// Special coinbase tx that mints the block reward to the miner.
    pub fn coinbase(recipient: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: "GENESIS".to_string(),
            to: recipient.to_string(),
            amount: 50,
            timestamp: Utc::now().timestamp(),
            tx_type: TxType::Coinbase,
        }
    }
}