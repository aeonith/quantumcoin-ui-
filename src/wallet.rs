use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use serde::{Serialize, Deserialize};
use std::fs::{File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub secret_key: String,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        let public_key = encode(pk.as_bytes());
        let secret_key = encode(sk.as_bytes());

        Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        }
    }

    pub fn save_to_file(&self, filename: &str) -> bool {
        if let Ok(mut file) = File::create(filename) {
            if let Ok(json) = serde_json::to_string(self) {
                return file.write_all(json.as_bytes()).is_ok();
            }
        }
        false
    }

    pub fn load_from_file(filename: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if let Ok(mut file) = File::open(filename) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return serde_json::from_str(&contents).ok();
            }
        }
        None
    }

    pub fn get_public_key_bytes(&self) -> Option<PublicKey> {
        decode(&self.public_key)
            .ok()
            .and_then(|bytes| PublicKey::from_bytes(&bytes).ok())
    }

    pub fn get_secret_key_bytes(&self) -> Option<SecretKey> {
        decode(&self.secret_key)
            .ok()
            .and_then(|bytes| SecretKey::from_bytes(&bytes).ok())
    }

    pub fn sign_message(&self, message: &[u8]) -> Option<Vec<u8>> {
        self.get_secret_key_bytes().map(|sk| sign_detached(message, &sk).as_bytes().to_vec())
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Some(pk) = self.get_public_key_bytes() {
            return verify_detached(signature, message, &pk).is_ok();
        }
        false
    }
}