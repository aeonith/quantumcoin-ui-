use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as TraitPubKey, SecretKey as TraitSecKey}; // for as_bytes and from_bytes
use base64::{engine::general_purpose, Engine as _};
use std::fs::{read_to_string, write};
use std::path::Path;

pub struct Wallet {
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (pk, sk) = keypair();
        Ok(Self {
            public_key: pk,
            private_key: sk,
        })
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        write(pub_path, general_purpose::STANDARD.encode(self.public_key.as_bytes()))?;
        write(priv_path, general_purpose::STANDARD.encode(self.private_key.as_bytes()))?;
        Ok(())
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        if !Path::new(pub_path).exists() || !Path::new(priv_path).exists() {
            return None;
        }

        let pub_key_encoded = read_to_string(pub_path).ok()?;
        let priv_key_encoded = read_to_string(priv_path).ok()?;

        let pub_key_bytes = general_purpose::STANDARD.decode(pub_key_encoded).ok()?;
        let priv_key_bytes = general_purpose::STANDARD.decode(priv_key_encoded).ok()?;

        let public_key = PublicKey::from_bytes(&pub_key_bytes).ok()?;
        let private_key = SecretKey::from_bytes(&priv_key_bytes).ok()?;

        Some(Self {
            public_key,
            private_key,
        })
    }

    pub fn get_address(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn get_private_key_string(&self) -> String {
        general_purpose::STANDARD.encode(self.private_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        sign_detached(message, &self.private_key).as_bytes().to_vec()
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        verify_detached(
            pqcrypto_dilithium::dilithium2::DetachedSignature::from_bytes(signature).unwrap(),
            message,
            &self.public_key,
        ).is_ok()
    }
}