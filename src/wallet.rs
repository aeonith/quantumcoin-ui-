use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, verify_detached, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, DetachedSignature as TraitDetachedSignature};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

const WALLET_FILE: &str = "wallet_key.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    pub balance: u64,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        let public_key = general_purpose::STANDARD.encode(pk.as_bytes());
        let private_key = general_purpose::STANDARD.encode(sk.as_bytes());

        Wallet {
            public_key,
            private_key,
            balance: 0,
        }
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn adjust_balance(&mut self, delta: i64) {
        if delta.is_negative() {
            let amount = delta.wrapping_abs() as u64;
            self.balance = self.balance.saturating_sub(amount);
        } else {
            self.balance = self.balance.saturating_add(delta as u64);
        }
    }

    pub fn save_to_files(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(WALLET_FILE, json);
        }
    }

    pub fn load_from_files() -> Self {
        if let Ok(mut file) = File::open(WALLET_FILE) {
            let mut data = String::new();
            if file.read_to_string(&mut data).is_ok() {
                if let Ok(wallet) = serde_json::from_str::<Wallet>(&data) {
                    return wallet;
                }
            }
        }
        Wallet::generate()
    }

    pub fn sign_message(&self, message: &[u8]) -> Option<Vec<u8>> {
        let sk_bytes = general_purpose::STANDARD.decode(&self.private_key).ok()?;
        let sk = SecretKey::from_bytes(&sk_bytes).ok()?;
        let signature = detached_sign(message, &sk);
        Some(signature.as_bytes().to_vec())
    }

    pub fn verify_signature(&self, message: &[u8], signature_bytes: &[u8]) -> bool {
        let pk_bytes = match general_purpose::STANDARD.decode(&self.public_key) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let pk = match PublicKey::from_bytes(&pk_bytes) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let signature = match DetachedSignature::from_bytes(signature_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };
        verify_detached(&signature, message, &pk).is_ok()
    }

    pub fn export_with_2fa(&self, code: &str) -> Option<String> {
        if code.len() < 6 {
            return None;
        }
        let combined = format!("{}:{}:{}", self.public_key, self.private_key, code);
        Some(general_purpose::STANDARD.encode(combined.as_bytes()))
    }
}