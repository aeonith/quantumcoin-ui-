use pqcrypto_dilithium::dilithium2::{keypair, sign, DetachedSignature, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, DetachedSignature as _};
use base64::{engine::general_purpose, Engine as _};
use std::fs;
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
        fs::write(pub_path, general_purpose::STANDARD.encode(self.public_key.as_bytes()))?;
        fs::write(priv_path, general_purpose::STANDARD.encode(self.secret_key.as_bytes()))?;
        Ok(())
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let pub_encoded = fs::read_to_string(pub_path).ok()?;
        let priv_encoded = fs::read_to_string(priv_path).ok()?;

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

    pub fn get_public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn get_private_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.secret_key.as_bytes())
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
            sig.verify_detached(message, &self.public_key).is_ok()
        } else {
            false
        }
    }
}