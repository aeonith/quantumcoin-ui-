use pqcrypto_dilithium::dilithium2::{
    keypair, open, DetachedSignature, PublicKey, SecretKey, SignedMessage,
};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, Signer};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

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

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let mut pub_buf = Vec::new();
        let mut priv_buf = Vec::new();

        if File::open(pub_path).ok()?.read_to_end(&mut pub_buf).is_err()
            || File::open(priv_path).ok()?.read_to_end(&mut priv_buf).is_err()
        {
            return None;
        }

        let pub_bytes = general_purpose::STANDARD.decode(pub_buf).ok()?;
        let priv_bytes = general_purpose::STANDARD.decode(priv_buf).ok()?;

        let pub_key = PublicKey::from_bytes(&pub_bytes).ok()?;
        let priv_key = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Wallet {
            public_key: pub_key,
            secret_key: priv_key,
        })
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> std::io::Result<()> {
        fs::write(pub_path, general_purpose::STANDARD.encode(self.public_key.as_bytes()))?;
        fs::write(priv_path, general_purpose::STANDARD.encode(self.secret_key.as_bytes()))?;
        Ok(())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signed = self.secret_key.sign(message);
        signed.as_bytes().to_vec()
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        match open(signature, &self.public_key) {
            Ok(recovered) => recovered == message,
            Err(_) => false,
        }
    }

    pub fn export_public_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn export_private_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.secret_key.as_bytes())
    }

    pub fn export_with_2fa(&self, password: &str) -> Option<String> {
        // Extremely basic 2FA-like mechanism (you should replace with real encryption)
        let pub_encoded = self.export_public_base64();
        let priv_encoded = self.export_private_base64();
        let combined = format!("{}::{}::{}", password, pub_encoded, priv_encoded);
        Some(general_purpose::STANDARD.encode(combined))
    }
}