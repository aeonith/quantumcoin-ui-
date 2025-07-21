use pqcrypto_dilithium::dilithium2::{keypair, sign, verify, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
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

    pub fn save_to_files(&self, pub_path: &str, sec_path: &str) {
        let encoded_pub = encode(self.public_key.as_bytes());
        let encoded_sec = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create(pub_path).expect("Failed to create public key file");
        pub_file.write_all(encoded_pub.as_bytes()).expect("Failed to write public key");

        let mut sec_file = File::create(sec_path).expect("Failed to create secret key file");
        sec_file.write_all(encoded_sec.as_bytes()).expect("Failed to write secret key");
    }

    pub fn load_from_files(pub_path: &str, sec_path: &str) -> Option<Self> {
        if !Path::new(pub_path).exists() || !Path::new(sec_path).exists() {
            return None;
        }

        let mut pub_contents = String::new();
        let mut pub_file = File::open(pub_path).ok()?;
        pub_file.read_to_string(&mut pub_contents).ok()?;
        let decoded_pub = decode(&pub_contents).ok()?;
        let public_key = PublicKey::from_bytes(&decoded_pub).ok()?;

        let mut sec_contents = String::new();
        let mut sec_file = File::open(sec_path).ok()?;
        sec_file.read_to_string(&mut sec_contents).ok()?;
        let decoded_sec = decode(&sec_contents).ok()?;
        let secret_key = SecretKey::from_bytes(&decoded_sec).ok()?;

        Some(Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        })
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_transaction(&self, transaction: &Transaction) -> Vec<u8> {
        let message = serde_json::to_vec(transaction).expect("Failed to serialize transaction");
        sign(&message, &self.secret_key)
    }

    pub fn verify_transaction(&self, transaction: &Transaction, signature: &[u8]) -> bool {
        let message = serde_json::to_vec(transaction).expect("Failed to serialize transaction");
        verify(&message, signature, &self.public_key).is_ok()
    }

    pub fn update_balance(&mut self, amount: f64) {
        self.balance += amount;
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }
}