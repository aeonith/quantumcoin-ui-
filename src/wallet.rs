use pqcrypto_dilithium::dilithium2::{keypair, sign, verify, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::SignedMessage as _; // enables `.as_bytes()`
use base64::{encode, decode};
use std::fs::{read_to_string, write};
use std::error::Error;

pub struct Wallet {
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (pk, sk) = keypair();
        Ok(Wallet {
            public_key: pk,
            private_key: sk,
        })
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> Result<(), Box<dyn Error>> {
        write(pub_path, encode(self.public_key.as_bytes()))?;
        write(priv_path, encode(self.private_key.as_bytes()))?;
        Ok(())
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let pub_key_encoded = read_to_string(pub_path).ok()?;
        let priv_key_encoded = read_to_string(priv_path).ok()?;

        let pub_key_bytes = decode(pub_key_encoded).ok()?;
        let priv_key_bytes = decode(priv_key_encoded).ok()?;

        let public_key = PublicKey::from_bytes(&pub_key_bytes).ok()?;
        let private_key = SecretKey::from_bytes(&priv_key_bytes).ok()?;

        Some(Wallet {
            public_key,
            private_key,
        })
    }

    pub fn get_public_key(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_private_key(&self) -> String {
        encode(self.private_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        sign(message, &self.private_key).as_bytes().to_vec()
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        pqcrypto_dilithium::dilithium2::verify(message, signature, &self.public_key).is_ok()
    }
}