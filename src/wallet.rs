use pqcrypto_dilithium::dilithium2::{
    keypair, sign_detached, verify_detached, PublicKey, SecretKey, SignedMessage,
};
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
        sign_detached(message, &self.secret_key).as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = SignedMessage::from_bytes(signature) {
            verify_detached(&sig, message, &self.public_key).is_ok()
        } else {
            false
        }
    }

    pub fn save_to_files(&self) {
        let _ = File::create("public.key")
            .and_then(|mut f| f.write_all(encode(self.public_key.as_bytes()).as_bytes()));
        let _ = File::create("private.key")
            .and_then(|mut f| f.write_all(encode(self.secret_key.as_bytes()).as_bytes()));
    }

    pub fn load_from_files() -> Option<Self> {
        let pk_data = std::fs::read_to_string("public.key").ok()?;
        let sk_data = std::fs::read_to_string("private.key").ok()?;
        let pk_bytes = decode(&pk_data).ok()?;
        let sk_bytes = decode(&sk_data).ok()?;
        let pk = PublicKey::from_bytes(&pk_bytes).ok()?;
        let sk = SecretKey::from_bytes(&sk_bytes).ok()?;
        Some(Wallet {
            public_key: pk,
            secret_key: sk,
            balance: 0.0,
        })
    }
}