use pqcrypto_dilithium::dilithium2::{
    keypair, sign_detached, verify_detached, PublicKey, SecretKey, DetachedSignature,
};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use base64::{encode, decode};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    pub balance: f64,
}

impl Wallet {
    pub fn load_or_create() -> Self {
        if let Ok(mut file) = File::open("wallet_key.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap()
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet {
                public_key: encode(pk.as_bytes()),
                private_key: encode(sk.as_bytes()),
                balance: 0.0,
            };
            wallet.save_to_file();
            wallet
        }
    }

    pub fn save_to_file(&self) {
        let serialized = serde_json::to_string_pretty(self).unwrap();
        fs::write("wallet_key.json", serialized).unwrap();
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn add_balance(&mut self, amount: f64) {
        self.balance += amount;
        self.save_to_file();
    }

    pub fn create_transaction(&self, to: &str, amount: f64) -> (String, String) {
        let message = format!("{}:{}:{}", self.public_key, to, amount);
        let priv_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&priv_bytes).unwrap();
        let signature = sign_detached(message.as_bytes(), &sk);
        (message, encode(signature.as_bytes()))
    }

    pub fn verify(&self, message: &[u8], signature_b64: &str) -> bool {
        let pub_bytes = decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pub_bytes).unwrap();
        let sig_bytes = decode(signature_b64).unwrap();
        let sig = DetachedSignature::from_bytes(&sig_bytes).unwrap();
        verify_detached(message, &sig, &pk).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }
}