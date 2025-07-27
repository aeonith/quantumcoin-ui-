use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, detached_verify, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, DetachedSignature as _};
use std::fs::{write, read};
use base64::{encode, decode};

pub struct Wallet {
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            private_key: sk,
        }
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> std::io::Result<()> {
        write(pub_path, encode(self.public_key.as_bytes()))?;
        write(priv_path, encode(self.private_key.as_bytes()))?;
        Ok(())
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let pub_key_encoded = read(pub_path).ok()?;
        let priv_key_encoded = read(priv_path).ok()?;

        let pub_key_bytes = decode(pub_key_encoded).ok()?;
        let priv_key_bytes = decode(priv_key_encoded).ok()?;

        let public_key = PublicKey::from_bytes(&pub_key_bytes).ok()?;
        let private_key = SecretKey::from_bytes(&priv_key_bytes).ok()?;

        Some(Wallet {
            public_key,
            private_key,
        })
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        detached_sign(&self.private_key, message).as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let sig_obj = DetachedSignature::from_bytes(signature).unwrap();
        detached_verify(message, &sig_obj, &self.public_key).is_ok()
    }

    pub fn get_public_key_string(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_private_key_string(&self) -> String {
        encode(self.private_key.as_bytes())
    }

    // Optional: if routes.rs needs raw keys
    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn get_private_key(&self) -> &SecretKey {
        &self.private_key
    }
}