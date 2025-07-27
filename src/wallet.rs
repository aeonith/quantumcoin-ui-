use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::io::{Read};
use std::path::Path;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Result<Self, &'static str> {
        let (pk, sk) = keypair();
        Ok(Self {
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
        if !Path::new(pub_path).exists() || !Path::new(priv_path).exists() {
            return None;
        }

        let pub_encoded = fs::read_to_string(pub_path).ok()?;
        let priv_encoded = fs::read_to_string(priv_path).ok()?;

        let pub_bytes = general_purpose::STANDARD.decode(pub_encoded).ok()?;
        let priv_bytes = general_purpose::STANDARD.decode(priv_encoded).ok()?;

        let pk = PublicKey::from_bytes(&pub_bytes).ok()?;
        let sk = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Self {
            public_key: pk,
            secret_key: sk,
        })
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sig = detached_sign(message, &self.secret_key);
        sig.as_bytes().to_vec()
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = DetachedSignature::from_bytes(signature) {
            sig.verify_detached(message, &self.public_key).is_ok()
        } else {
            false
        }
    }

    pub fn get_public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn get_private_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.secret_key.as_bytes())
    }
}