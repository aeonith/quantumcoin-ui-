use serde::{Serialize, Deserialize};

/// A simple transaction: in future you’ll add Dilithium signatures here.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub sender:    String,
    pub recipient: String,
    pub amount:    u64,
    pub timestamp: u128,
}