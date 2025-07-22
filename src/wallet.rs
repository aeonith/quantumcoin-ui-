use serde::{Serialize, Deserialize};
use rand::Rng;
use sha2::{Sha256, Digest};
use std::{fs, path::Path};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address:       String,
    pub password_hash: String,
    // TODO: store Dilithium public / private keys here
}

impl Wallet {
    /// Load from disk or generate a new wallet
    pub fn load_or_generate() -> Self {
        let path = "wallet.json";
        if Path::new(path).exists() {
            let data = fs::read_to_string(path).expect("read wallet");
            serde_json::from_str(&data).expect("parse wallet")
        } else {
            let mut rng = rand::thread_rng();
            let address = Uuid::new_v4().to_string();
            let password = "default123"; // replace with user‐supplied
            let password_hash = Self::hash_password(password);
            let w = Wallet { address, password_hash };
            fs::write(path, serde_json::to_string_pretty(&w).unwrap()).unwrap();
            w
        }
    }

    fn hash_password(p: &str) -> String {
        let mut h = Sha256::new();
        h.update(p.as_bytes());
        format!("{:x}", h.finalize())
    }

    pub fn verify_password(&self, p: &str) -> bool {
        Self::hash_password(p) == self.password_hash
    }

    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn create_transaction(&self, recipient: &str, amount: u64) -> crate::transaction::Transaction {
        crate::transaction::Transaction {
            sender:    self.address.clone(),
            recipient: recipient.to_string(),
            amount,
            timestamp: chrono::Utc::now().timestamp_millis() as u128,
        }
    }
}