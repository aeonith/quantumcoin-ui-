use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub address: String,
}

impl Wallet {
    pub fn new() -> Self {
        let random_number: u64 = rand::thread_rng().gen();
        let address = format!("wallet-{}", random_number);
        Wallet { address }
    }

    // This must exist and be `pub`:
    pub fn load_or_generate() -> Self {
        let path = "wallet.json";
        if Path::new(path).exists() {
            let data = fs::read_to_string(path).expect("Failed to read wallet file");
            serde_json::from_str(&data).expect("Failed to parse wallet file")
        } else {
            let wallet = Wallet::new();
            let data = serde_json::to_string_pretty(&wallet).expect("Failed to serialize wallet");
            fs::write(path, data).expect("Failed to write wallet file");
            wallet
        }
    }
}