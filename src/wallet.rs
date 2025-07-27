use pqcrypto_dilithium::dilithium2::{
    keypair, sign_detached, verify_detached, PublicKey, SecretKey, DetachedSignature,
};
use pqcrypto_traits::sign::{
    PublicKey as PublicKeyTrait,
    SecretKey as SecretKeyTrait,
    Signature as SignatureTrait,
};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{write, read};
use std::path::Path;

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
        write(pub_path, general_purpose::STANDARD.encode(self.public_key.as_bytes()))?;
        write(priv_path, general_purpose::STANDARD.encode(self.private_key.as_bytes()))?;
        Ok(())
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        if !Path::new(pub_path).exists() || !Path::new(priv_path).exists() {
            return None;
        }

        let pub_key_encoded = read(pub_path).ok()?;
        let priv_key_encoded = read(priv_path).ok()?;

        let pub_key_bytes = general_purpose::STANDARD.decode(pub_key_encoded).ok()?;
        let priv_key_bytes = general_purpose::STANDARD.decode(priv_key_encoded).ok()?;

        let public_key = PublicKey::from_bytes(&pub_key_bytes).ok()?;
        let private_key = SecretKey::from_bytes(&priv_key_bytes).ok()?;

        Some(Wallet {
            public_key,
            private_key,
        })
    }

    pub fn new_and_save() -> Self {
        let wallet = Wallet::generate();
        let _ = wallet.save_to_files("public.key", "private.key");
        wallet
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        sign_detached(message, &self.private_key).as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = DetachedSignature::from_bytes(signature) {
            verify_detached(&sig, message, &self.public_key).is_ok()
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