use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use serde::{Serialize, Deserialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub public_key: String,
    pub secret_key: String,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: base64::encode(pk.as_bytes()),
            secret_key: base64::encode(sk.as_bytes()),
        }
    }

    pub fn save_to_file(&self, path: &str) {
        let data = serde_json::to_string_pretty(&self).expect("Failed to serialize wallet");
        let mut file = File::create(path).expect("Failed to create wallet file");
        file.write_all(data.as_bytes()).expect("Failed to write wallet");
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        if let Ok(mut file) = File::open(path) {
            let mut data = String::new();
            file.read_to_string(&mut data).ok()?;
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }

    pub fn init_wallet() -> Self {
        let path = "wallet_key.json";
        if let Some(wallet) = Wallet::load_from_file(path) {
            println!("🔐 Loaded existing wallet.");
            wallet
        } else {
            println!("🔐 No wallet found. Generating new one...");
            let wallet = Wallet::generate();
            wallet.save_to_file(path);
            println!("✅ New wallet saved to '{}'", path);
            wallet
        }
    }

    pub fn get_address(&self) -> &str {
        &self.public_key
    }
}