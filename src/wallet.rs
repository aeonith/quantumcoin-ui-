use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, sign_detached, verify_detached, Signature};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, SignedMessage, DetachedSignature};
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
    /// Generates a new wallet with fresh Dilithium2 keypair
    pub fn new() -> Wallet {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
            balance: 0.0,
        }
    }

    /// Returns the base64 wallet address (public key)
    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    /// Signs a message with the wallet's secret key
    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sig: Signature = sign_detached(message, &self.secret_key);
        sig.as_bytes().to_vec()
    }

    /// Verifies a message signature using the wallet's public key
    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        let sig = Signature::from_bytes(signature).ok();
        if let Some(sig) = sig {
            verify_detached(&sig, message, &self.public_key).is_ok()
        } else {
            false
        }
    }

    /// Saves the public/private keys as base64 to disk
    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        let pub_encoded = encode(self.public_key.as_bytes());
        let priv_encoded = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create(pub_path).expect("Failed to create public key file");
        pub_file.write_all(pub_encoded.as_bytes()).expect("Failed to write public key");

        let mut priv_file = File::create(priv_path).expect("Failed to create private key file");
        priv_file.write_all(priv_encoded.as_bytes()).expect("Failed to write private key");
    }

    /// Loads wallet keys from base64-encoded files
    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Wallet> {
        if !Path::new(pub_path).exists() || !Path::new(priv_path).exists() {
            return None;
        }

        let mut pub_file = File::open(pub_path).ok()?;
        let mut priv_file = File::open(priv_path).ok()?;

        let mut pub_buf = String::new();
        let mut priv_buf = String::new();
        pub_file.read_to_string(&mut pub_buf).ok()?;
        priv_file.read_to_string(&mut priv_buf).ok()?;

        let pub_bytes = decode(pub_buf).ok()?;
        let priv_bytes = decode(priv_buf).ok()?;

        let public_key = PublicKey::from_bytes(&pub_bytes).ok()?;
        let secret_key = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        })
    }
}

/// Returns a fresh base64 public/private keypair — used by `/keys` endpoint
pub fn get_keys() -> (String, String) {
    let wallet = Wallet::new();
    let pub_key = encode(wallet.public_key.as_bytes());
    let priv_key = encode(wallet.secret_key.as_bytes());
    (pub_key, priv_key)
}