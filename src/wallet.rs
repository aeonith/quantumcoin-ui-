use base64::{engine::general_purpose, Engine as _};
use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    pub balance: u64,
    pub recent_tx: Vec<String>,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        let path = "wallet_key.json";

        if let Ok(json) = fs::read_to_string(path) {
            if let Ok(wallet) = serde_json::from_str(&json) {
                return wallet;
            }
        }

        let (pk, sk) = keypair();
        let wallet = Wallet {
            public_key: general_purpose::STANDARD.encode(pk.as_bytes()),
            private_key: general_purpose::STANDARD.encode(sk.as_bytes()),
            balance: 0,
            recent_tx: Vec::new(),
        };

        let json = serde_json::to_string_pretty(&wallet).unwrap();
        fs::write(path, json).unwrap();
        wallet
    }

    pub fn sign_message(&self, msg: &[u8]) -> DetachedSignature {
        let sk_bytes = general_purpose::STANDARD.decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        sign_detached(msg, &sk)
    }

    pub fn verify_signature(&self, msg: &[u8], sig: &DetachedSignature) -> bool {
        let pk_bytes = general_purpose::STANDARD.decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        verify_detached(msg, sig, &pk).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn add_transaction(&mut self, tx: String) {
        self.recent_tx.push(tx);
        if self.recent_tx.len() > 5 {
            self.recent_tx.remove(0);
        }
    }

    pub fn show_last_transactions(&self) {
        println!("Last 5 Transactions:");
        for tx in &self.recent_tx {
            println!("- {}", tx);
        }
    }

    pub fn save_to_file(&self) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        fs::write("wallet_key.json", json).unwrap();
    }

    pub fn load_from_file() -> Option<Self> {
        if let Ok(json) = fs::read_to_string("wallet_key.json") {
            serde_json::from_str(&json).ok()
        } else {
            None
        }
    }
}