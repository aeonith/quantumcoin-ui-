use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub signature: Option<String>, // base64-encoded Dilithium signature
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
        if self.sender == "GENESIS" || self.sender == "DEVFUND" {
            return true;
        }

        // Ensure presence of signature
        if self.signature.is_none() {
            return false;
        }

        // TODO: Signature validation logic (handled externally in wallet.rs usually)
        true
    }

    pub fn to_message(&self) -> String {
        format!("{}{}{}", self.sender, self.recipient, self.amount)
    }
}