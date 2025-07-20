use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, Signer, Verifier};
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

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        let pub_encoded = encode(self.public_key.as_bytes());
        let priv_encoded = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create(pub_path).expect("Unable to create public key file");
        pub_file.write_all(pub_encoded.as_bytes()).expect("Unable to write public key");

        let mut priv_file = File::create(priv_path).expect("Unable to create private key file");
        priv_file.write_all(priv_encoded.as_bytes()).expect("Unable to write private key");
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Self {
        let mut pub_encoded = String::new();
        File::open(pub_path).expect("Public key file not found")
            .read_to_string(&mut pub_encoded).expect("Unable to read public key file");
        let pub_bytes = decode(&pub_encoded).expect("Failed to decode public key");
        let public_key = PublicKey::from_bytes(&pub_bytes).expect("Failed to parse public key");

        let mut priv_encoded = String::new();
        File::open(priv_path).expect("Private key file not found")
            .read_to_string(&mut priv_encoded).expect("Unable to read private key file");
        let priv_bytes = decode(&priv_encoded).expect("Failed to decode private key");
        let secret_key = SecretKey::from_bytes(&priv_bytes).expect("Failed to parse secret key");

        Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        self.secret_key.sign(message)
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        self.public_key.verify(message, signature).is_ok()
    }

    pub fn create_transaction(&self, recipient: &str, amount: f64) -> Option<Transaction> {
        if self.balance < amount {
            println!("Insufficient balance.");
            return None;
        }

        let message = format!("{}:{}:{}", encode(self.public_key.as_bytes()), recipient, amount);
        let signature = self.sign_message(message.as_bytes());

        Some(Transaction {
            sender: encode(self.public_key.as_bytes()),
            recipient: recipient.to_string(),
            amount,
            signature: encode(&signature),
        })
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn update_balance(&mut self, amount: f64) {
        self.balance += amount;
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }
}