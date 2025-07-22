use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature, Signatures, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
use serde::{Serialize, Deserialize};
use std::fs;
use base64::{encode, decode};
use crate::transaction::Transaction;

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    pub password: String,
}

impl Wallet {
    /// Load existing or generate new wallet
    pub fn load_or_generate() -> Self {
        let path = "wallet_key.json";
        if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap()
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet {
                public_key: encode(pk.as_bytes()),
                private_key: encode(sk.as_bytes()),
                password: "quantum_secure_password".to_string(), // Default password
            };
            wallet.save();
            wallet
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write("wallet_key.json", json).unwrap();
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn verify_password(&self, password: &str) -> bool {
        self.password == password
    }

    pub fn create_transaction(&self, recipient: &str, amount: u64) -> Transaction {
        Transaction {
            sender: self.get_address(),
            recipient: recipient.to_string(),
            amount,
            timestamp: now(),
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> DetachedSignature {
        let sk_bytes = decode(&self.private_key).expect("Invalid private key base64");
        let sk = SecretKey::from_bytes(&sk_bytes).expect("Invalid private key");
        sign_detached(message, &sk)
    }

    pub fn verify_signature(&self, message: &[u8], sig: &DetachedSignature) -> bool {
        let pk_bytes = decode(&self.public_key).expect("Invalid public key base64");
        let pk = PublicKey::from_bytes(&pk_bytes).expect("Invalid public key");
        pk.verify_detached(sig, message).is_ok()
    }
}

fn now() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}