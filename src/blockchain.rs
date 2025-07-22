use crate::models::Transaction;
use std::{fs, path::Path};

pub struct Blockchain {
    transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn load_or_create() -> Self {
        if Path::new("blockchain.json").exists() {
            let data = fs::read_to_string("blockchain.json").unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            Blockchain {
                transactions: vec![]
            }
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
        let json = serde_json::to_string(&self).unwrap();
        fs::write("blockchain.json", json).unwrap();
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        let mut balance = 0;
        for tx in &self.transactions {
            if tx.recipient == address {
                balance += tx.amount;
            } else if tx.sender == address {
                balance -= tx.amount;
            }
        }
        balance
    }
}

impl serde::Serialize for Blockchain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let mut state = serializer.serialize_struct("Blockchain", 1)?;
        state.serialize_field("transactions", &self.transactions)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Blockchain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(Deserialize)]
        struct BlockchainData {
            transactions: Vec<Transaction>,
        }

        let data = BlockchainData::deserialize(deserializer)?;
        Ok(Blockchain {
            transactions: data.transactions,
        })
    }
}