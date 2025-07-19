use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64) -> Transaction {
        Transaction { id: Uuid::new_v4().to_string(), from, to, amount }
    }
}