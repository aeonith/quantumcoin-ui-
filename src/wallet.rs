use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, sign_detached, verify_detached, DetachedSignature};
use base64::{encode, decode};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Self {
        let (public_key, secret_key) = keypair();
        Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        }
    }

    pub fn save_to_files(&self, pub_path: &str, sec_path: &str) {
        let pub_encoded = encode(self.public_key.as_bytes());
        let sec_encoded = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create(pub_path).expect("Failed to create public key file");
        let mut sec_file = File::create(sec_path).expect("Failed to create secret key file");

        pub_file.write_all(pub_encoded.as_bytes()).expect("Failed to write public key");
        sec_file.write_all(sec_encoded.as_bytes()).expect("Failed to write secret key");
    }

    pub fn load_from_files(pub_path: &str, sec_path: &str) -> Self {
        let mut pub_encoded = String::new();
        let mut sec_encoded = String::new();

        File::open(pub_path).expect("Public key file not found")
            .read_to_string(&mut pub_encoded).expect("Failed to read public key");

        File::open(sec_path).expect("Secret key file not found")
            .read_to_string(&mut sec_encoded).expect("Failed to read secret key");

        let pub_decoded = decode(&pub_encoded).expect("Failed to decode public key");
        let sec_decoded = decode(&sec_encoded).expect("Failed to decode secret key");

        let public_key = PublicKey::from_bytes(&pub_decoded).expect("Invalid public key bytes");
        let secret_key = SecretKey::from_bytes(&sec_decoded).expect("Invalid secret key bytes");

        Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sig = sign_detached(message, &self.secret_key);
        sig.as_bytes().to_vec()
    }

    pub fn verify_signature(message: &[u8], signature: &[u8], public_key: &PublicKey) -> bool {
        let detached = DetachedSignature::from_bytes(signature).expect("Invalid signature format");
        verify_detached(&detached, message, public_key).is_ok()
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    // ✅ Optional helper for main.rs if it needs both keys
    pub fn get_keys(&self) -> (PublicKey, SecretKey) {
        (self.public_key.clone(), self.secret_key.clone())
    }
}