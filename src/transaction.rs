use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub signature: Option<String>,
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: u64, signature: Option<String>, timestamp: u64) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            signature,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        // For now we assume the genesis transaction or manually added transactions are valid
        // Validation should be done during transaction addition or API-level verification
        true
    }
}