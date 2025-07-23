use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_dilithium::dilithium2::verify_detached_signature;
use pqcrypto_traits::sign::{PublicKey as PKTrait, SecretKey as SKTrait, DetachedSignature as SigTrait};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        let public_key = general_purpose::STANDARD.encode(pk.as_bytes());
        let private_key = general_purpose::STANDARD.encode(sk.as_bytes());
        Self { public_key, private_key }
    }

    pub fn save_to_files(&self) {
        fs::write("wallet_public.key", &self.public_key).expect("Unable to save public key");
        fs::write("wallet_private.key", &self.private_key).expect("Unable to save private key");
    }

    pub fn load_from_files() -> Self {
        let public_key = fs::read_to_string("wallet_public.key").unwrap_or_default();
        let private_key = fs::read_to_string("wallet_private.key").unwrap_or_default();
        Self { public_key, private_key }
    }

    pub fn sign_message(&self, message: &[u8]) -> Option<Vec<u8>> {
        let sk_bytes = general_purpose::STANDARD.decode(&self.private_key).ok()?;
        let sk = SecretKey::from_bytes(&sk_bytes).ok()?;
        let signature = detached_sign(message, &sk);
        Some(signature.as_bytes().to_vec())
    }

    pub fn verify_signature(&self, message: &[u8], signature_bytes: &[u8]) -> bool {
        let pk_bytes = general_purpose::STANDARD.decode(&self.public_key).ok()?;
        let pk = PublicKey::from_bytes(&pk_bytes).ok()?;
        let signature = DetachedSignature::from_bytes(signature_bytes).ok()?;
        verify_detached_signature(&signature, message, &pk).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn export_with_2fa(&self, password: &str) -> Result<(), String> {
        let combined = format!("{}:{}", self.public_key, self.private_key);
        let encrypted = xor_encrypt(&combined, password);
        fs::write("wallet_backup.qtc", encrypted).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn import_with_2fa(password: &str) -> Result<Self, String> {
        let encrypted = fs::read("wallet_backup.qtc").map_err(|e| e.to_string())?;
        let decrypted = xor_decrypt(&encrypted, password);
        let parts: Vec<&str> = decrypted.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid backup format".to_string());
        }
        Ok(Self {
            public_key: parts[0].to_string(),
            private_key: parts[1].to_string(),
        })
    }
}

// Simple XOR encryption for 2FA export
fn xor_encrypt(data: &str, key: &str) -> Vec<u8> {
    data.bytes().zip(key.bytes().cycle()).map(|(a, b)| a ^ b).collect()
}

fn xor_decrypt(data: &[u8], key: &str) -> String {
    String::from_utf8(data.iter().zip(key.bytes().cycle()).map(|(a, b)| a ^ b).collect()).unwrap_or_default()
}