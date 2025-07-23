use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub signature: Option<String>, // base64 string of the signature
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: u64, signature: Option<String>) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            signature,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.sender == "GENESIS" || self.sender == "COINBASE" {
            return true;
        }

        // Ensure presence of signature
        if self.signature.is_none() {
            return false;
        }

        // Validation logic placeholder (signature verification occurs in routes.rs/backend)
        true
    }

    pub fn to_message(&self) -> String {
        format!("{}{}{}", self.sender, self.recipient, self.amount)
    }
}