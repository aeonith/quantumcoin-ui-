use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: Option<String>,
    pub timestamp: u128,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64, signature: Option<String>, timestamp: u128) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            signature,
            timestamp,
        }
    }
}