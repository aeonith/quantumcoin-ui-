use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, sign_detached, verify_detached};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, Signature};
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

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sig: Signature = sign_detached(message, &self.secret_key);
        sig.as_bytes().to_vec()
    }

    pub fn verify_message(message: &[u8], signature: &[u8], public_key: &PublicKey) -> bool {
        let sig = Signature::from_bytes(signature).unwrap_or_else(|_| return false);
        verify_detached(&sig, message, public_key).is_ok()
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        let pub_encoded = encode(self.public_key.as_bytes());
        let priv_encoded = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create(pub_path).unwrap();
        pub_file.write_all(pub_encoded.as_bytes()).unwrap();

        let mut priv_file = File::create(priv_path).unwrap();
        priv_file.write_all(priv_encoded.as_bytes()).unwrap();
    }

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