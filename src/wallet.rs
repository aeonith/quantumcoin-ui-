use pqcrypto_dilithium::dilithium2::{keypair, sign, verify, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use std::fs::{File};
use std::io::{Write, Read};
use std::path::Path;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Wallet {
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

        let mut pub_file = File::create(pub_path).expect("Failed to create public key file");
        pub_file.write_all(pub_encoded.as_bytes()).expect("Failed to write public key");

        let mut priv_file = File::create(priv_path).expect("Failed to create private key file");
        priv_file.write_all(priv_encoded.as_bytes()).expect("Failed to write private key");
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Wallet {
        let mut pub_file = File::open(pub_path).expect("Failed to open public key file");
        let mut pub_encoded = String::new();
        pub_file.read_to_string(&mut pub_encoded).expect("Failed to read public key");
        let pub_decoded = decode(&pub_encoded).expect("Failed to decode public key");
        let pub_key = PublicKey::from_bytes(&pub_decoded).expect("Invalid public key bytes");

        let mut priv_file = File::open(priv_path).expect("Failed to open private key file");
        let mut priv_encoded = String::new();
        priv_file.read_to_string(&mut priv_encoded).expect("Failed to read private key");
        let priv_decoded = decode(&priv_encoded).expect("Failed to decode private key");
        let priv_key = SecretKey::from_bytes(&priv_decoded).expect("Invalid private key bytes");

        Wallet {
            public_key: pub_key,
            secret_key: priv_key,
            balance: 0.0,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        sign(message, &self.secret_key)
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        verify(message, signature, &self.public_key).is_ok()
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn set_balance(&mut self, amount: f64) {
        self.balance = amount;
    }
}