use pqcrypto_dilithium::dilithium2::{keypair, sign, PublicKey, SecretKey, DetachedSignature, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, DetachedSignature as _, SignedMessage as _};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{read_to_string, write};

pub struct Wallet {
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (pk, sk) = keypair();
        Ok(Wallet {
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
        let pub_key_base64 = read_to_string(pub_path).ok()?;
        let priv_key_base64 = read_to_string(priv_path).ok()?;

        let pub_key_bytes = general_purpose::STANDARD.decode(pub_key_base64).ok()?;
        let priv_key_bytes = general_purpose::STANDARD.decode(priv_key_base64).ok()?;

        let public_key = PublicKey::from_bytes(&pub_key_bytes).ok()?;
        let private_key = SecretKey::from_bytes(&priv_key_bytes).ok()?;

        Some(Wallet {
            public_key,
            private_key,
        })
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signature: SignedMessage = sign(message, &self.private_key);
        signature.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = DetachedSignature::from_bytes(signature) {
            pqcrypto_dilithium::dilithium2::verify_detached(&sig, message, &self.public_key).is_ok()
        } else {
            false
        }
    }

    pub fn get_public_key(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn get_private_key(&self) -> String {
        general_purpose::STANDARD.encode(self.private_key.as_bytes())
    }
}