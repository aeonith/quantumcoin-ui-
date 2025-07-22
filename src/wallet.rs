use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::{fs, path::Path};
use rand::Rng;

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    address: String,
    password_hash: String,
}

impl Wallet {
    pub fn load_or_create() -> Self {
        if Path::new("wallet.json").exists() {
            let data = fs::read_to_string("wallet.json").unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            let mut rng = rand::thread_rng();
            let address: String = (0..32).map(|_| rng.gen_range(0..10).to_string()).collect();
            let password = "default123";
            let password_hash = Self::hash_password(password);

            let wallet = Wallet { address, password_hash };
            let json = serde_json::to_string(&wallet).unwrap();
            fs::write("wallet.json", json).unwrap();
            wallet
        }
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password);
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_password(&self, password: &str) -> bool {
        Self::hash_password(password) == self.password_hash
    }

    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn create_transaction(&self, recipient: String, amount: u64) -> crate::models::Transaction {
        crate::models::Transaction {
            sender: self.address.clone(),
            recipient,
            amount,
        }
    }
}