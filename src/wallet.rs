use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature as TraitDetachedSignature, PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{engine::general_purpose, Engine as _};
use serde::{Serialize, Deserialize};
use std::fs::{File};
use std::io::{Read};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: general_purpose::STANDARD.encode(pk.as_bytes()),
            private_key: general_purpose::STANDARD.encode(sk.as_bytes()),
            balance: 0.0,
        }
    }

    pub fn save_to_files(&self) {
        let pub_file = File::create("wallet_key.json").unwrap();
        serde_json::to_writer(pub_file, &self).unwrap();
    }

    pub fn load_from_files() -> Option<Self> {
        if Path::new("wallet_key.json").exists() {
            let mut file = File::open("wallet_key.json").unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).ok()
        } else {
            None
        }
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn add_balance(&mut self, amount: f64) {
        self.balance += amount;
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn sign_message(&self, message: &[u8]) -> (Vec<u8>, String) {
        let priv_bytes = general_purpose::STANDARD.decode(&self.private_key).unwrap();
        let sk = TraitSecretKey::from_bytes(&priv_bytes).unwrap();
        let signature = detached_sign(message, &sk);
        (message.to_vec(), general_purpose::STANDARD.encode(signature.as_bytes()))
    }

    pub fn verify(&self, message: &[u8], signature_b64: &str) -> bool {
        let pub_bytes = general_purpose::STANDARD.decode(&self.public_key).unwrap();
        let sig_bytes = general_purpose::STANDARD.decode(signature_b64).unwrap();
        let sig = TraitDetachedSignature::from_bytes(&sig_bytes).unwrap();
        let pk = TraitPublicKey::from_bytes(&pub_bytes).unwrap();
        pk.verify_detached(&sig, message).is_ok()
    }
}