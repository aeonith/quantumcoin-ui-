use pqcrypto_dilithium::dilithium2::{keypair, sign, DetachedSignature, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{File};
use std::io::{Read, Write};
use std::path::Path;

use crate::transaction::Transaction;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (pk, sk) = keypair();
        Ok(Wallet {
            public_key: pk,
            secret_key: sk,
        })
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> std::io::Result<()> {
        let pub_bytes = self.public_key.as_bytes();
        let priv_bytes = self.secret_key.as_bytes();

        let pub_encoded = general_purpose::STANDARD.encode(pub_bytes);
        let priv_encoded = general_purpose::STANDARD.encode(priv_bytes);

        std::fs::write(pub_path, pub_encoded)?;
        std::fs::write(priv_path, priv_encoded)?;
        Ok(())
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let pub_encoded = std::fs::read_to_string(pub_path).ok()?;
        let priv_encoded = std::fs::read_to_string(priv_path).ok()?;

        let pub_bytes = general_purpose::STANDARD.decode(pub_encoded).ok()?;
        let priv_bytes = general_purpose::STANDARD.decode(priv_encoded).ok()?;

        let public_key = PublicKey::from_bytes(&pub_bytes).ok()?;
        let secret_key = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Wallet {
            public_key,
            secret_key,
        })
    }

    pub fn get_address_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn create_transaction(&self, recipient: &str, amount: u64, sender: &str) -> Transaction {
        let message = format!("{}{}{}", sender, recipient, amount);
        let signature = sign(message.as_bytes(), &self.secret_key);

        Transaction {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            amount,
            signature: Some(general_purpose::STANDARD.encode(signature.as_bytes())),
        }
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = DetachedSignature::from_bytes(signature) {
            pqcrypto_dilithium::dilithium2::verify(message, &sig, &self.public_key).is_ok()
        } else {
            false
        }
    }
}