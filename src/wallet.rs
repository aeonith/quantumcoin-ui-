use pqcrypto_dilithium::dilithium2::{
    keypair, sign_detached, verify_detached, PublicKey, SecretKey, SignedMessage,
};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use serde::{Serialize, Deserialize};
use std::fs::{File};
use std::io::{Write, Read};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: encode(pk.as_bytes()),
            private_key: encode(sk.as_bytes()),
            balance: 0.0,
        }
    }

    pub fn load_from_file(file_path: &str) -> Option<Self> {
        if let Ok(mut file) = File::open(file_path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                serde_json::from_str(&contents).ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn save_to_file(&self, file_path: &str) {
        if let Ok(serialized) = serde_json::to_string(self) {
            let _ = File::create(file_path).and_then(|mut f| f.write_all(serialized.as_bytes()));
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Option<String> {
        let sk_bytes = decode(&self.private_key).ok()?;
        let sk = SecretKey::from_bytes(&sk_bytes).ok()?;
        let signed = sign_detached(message, &sk);
        Some(encode(signed.as_bytes()))
    }

    pub fn verify_message(&self, message: &[u8], signature_b64: &str) -> bool {
        let pk_bytes = decode(&self.public_key).ok()?;
        let signature_bytes = decode(signature_b64).ok()?;
        let pk = PublicKey::from_bytes(&pk_bytes).ok()?;
        let sig = SignedMessage::from_bytes(&signature_bytes).ok()?;
        verify_detached(&sig, message, &pk).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn update_balance(&mut self, amount: f64) {
        self.balance += amount;
    }

    pub fn create_transaction(&self, to: &str, amount: f64) -> Option<String> {
        let tx_data = format!("from:{}-to:{}-amount:{}", self.get_address(), to, amount);
        self.sign_message(tx_data.as_bytes())
    }

    pub fn export_with_2fa(&self, code: &str) -> Option<String> {
        let export = format!("wallet:{}:{}:{}", self.public_key, self.private_key, code);
        Some(encode(export.as_bytes()))
    }
}