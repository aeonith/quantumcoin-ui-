use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn new(
        sender: String,
        recipient: String,
        amount: u64,
        timestamp: u64,
        signature: Option<String>,
    ) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            timestamp,
            signature,
        }
    }
}