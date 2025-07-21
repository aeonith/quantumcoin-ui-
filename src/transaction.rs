// src/transaction.rs

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: Option<String>, // Quantum-safe signature (base64)
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64, signature: Option<String>, timestamp: u64) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            signature,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        // Add real signature verification logic if needed.
        // This is a placeholder assuming signature presence = valid
        self.signature.is_some()
    }

    pub fn summary(&self) -> String {
        format!(
            "From: {}\nTo: {}\nAmount: {}\nTimestamp: {}\nSigned: {}\n",
            self.sender,
            self.recipient,
            self.amount,
            self.timestamp,
            self.signature.is_some()
        )
    }
}