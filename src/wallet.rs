use pqcrypto_dilithium::dilithium2::{keypair, sign, verify, PublicKey, SecretKey};
use pqcrypto_traits::sign::{Signer, Verifier, PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use std::fs::{File};
use std::io::{Write, Read};
use std::path::Path;
use crate::transaction::Transaction;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
            balance: 0.0,
        }
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        sign(message, &self.secret_key)
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        verify(message, signature, &self.public_key).is_ok()
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        let pub_encoded = encode(self.public_key.as_bytes());
        let priv_encoded = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create(pub_path).expect("Failed to create public key file");
        pub_file.write_all(pub_encoded.as_bytes()).expect("Failed to write public key");

        let mut priv_file = File::create(priv_path).expect("Failed to create private key file");
        priv_file.write_all(priv_encoded.as_bytes()).expect("Failed to write private key");
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        if Path::new(pub_path).exists() && Path::new(priv_path).exists() {
            let mut pub_encoded = String::new();
            File::open(pub_path).ok()?.read_to_string(&mut pub_encoded).ok()?;
            let pub_bytes = decode(pub_encoded).ok()?;
            let public_key = PublicKey::from_bytes(&pub_bytes).ok()?;

            let mut priv_encoded = String::new();
            File::open(priv_path).ok()?.read_to_string(&mut priv_encoded).ok()?;
            let priv_bytes = decode(priv_encoded).ok()?;
            let secret_key = SecretKey::from_bytes(&priv_bytes).ok()?;

            Some(Wallet {
                public_key,
                secret_key,
                balance: 0.0,
            })
        } else {
            None
        }
    }

    pub fn create_transaction(&self, recipient: &str, amount: f64) -> Option<Transaction> {
        if self.balance < amount {
            return None;
        }

        let message = format!("{}:{}:{}", self.get_address(), recipient, amount);
        let signature = self.sign_message(message.as_bytes());
        Some(Transaction {
            sender: self.get_address(),
            recipient: recipient.to_string(),
            amount,
            signature: encode(&signature),
        })
    }
}