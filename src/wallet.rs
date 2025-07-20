use pqcrypto_dilithium::dilithium2::{keypair, sign, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use std::fs::File;
use std::io::{Read, Write};

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
}

impl Wallet {
    /// Generate a fresh keypair
    pub fn new() -> Self {
        let (public_key, secret_key) = keypair();
        Wallet { public_key, secret_key, balance: 0.0 }
    }

    /// Saves both keys (base64‐encoded) to disk
    pub fn save_to_files(&self, pub_path: &str, sec_path: &str) {
        let pub_encoded = encode(self.public_key.as_bytes());
        let sec_encoded = encode(self.secret_key.as_bytes());
        File::create(pub_path)
            .and_then(|mut f| f.write_all(pub_encoded.as_bytes()))
            .expect("Failed to write public key");
        File::create(sec_path)
            .and_then(|mut f| f.write_all(sec_encoded.as_bytes()))
            .expect("Failed to write secret key");
    }

    /// Loads keys back from disk
    pub fn load_from_files(pub_path: &str, sec_path: &str) -> Self {
        let mut pub_s = String::new();
        let mut sec_s = String::new();
        File::open(pub_path)
            .and_then(|mut f| f.read_to_string(&mut pub_s))
            .expect("Failed to read public key");
        File::open(sec_path)
            .and_then(|mut f| f.read_to_string(&mut sec_s))
            .expect("Failed to read secret key");

        let pub_bytes = decode(&pub_s).expect("Invalid base64 public key");
        let sec_bytes = decode(&sec_s).expect("Invalid base64 secret key");
        let public_key  = PublicKey::from_bytes(&pub_bytes).expect("Bad public key bytes");
        let secret_key  = SecretKey::from_bytes(&sec_bytes).expect("Bad secret key bytes");

        Wallet { public_key, secret_key, balance: 0.0 }
    }

    /// Sign a message, returning the raw `SignedMessage` blob
    pub fn sign_message(&self, msg: &[u8]) -> SignedMessage {
        sign(msg, &self.secret_key)
    }

    /// Returns your address as base64(pub_key)
    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    /// **NEW** helper so `main.rs` can grab both keys at once
    pub fn get_keys(&self) -> (PublicKey, SecretKey) {
        (self.public_key.clone(), self.secret_key.clone())
    }
}