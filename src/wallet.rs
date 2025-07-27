use pqcrypto_dilithium::dilithium2::{keypair, sign, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use base64::{encode, decode};

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (pk, sk) = keypair();
        Ok(Self {
            public_key: pk,
            secret_key: sk,
        })
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let mut pub_file = File::open(pub_path).ok()?;
        let mut priv_file = File::open(priv_path).ok()?;

        let mut pub_buf = String::new();
        let mut priv_buf = String::new();
        pub_file.read_to_string(&mut pub_buf).ok()?;
        priv_file.read_to_string(&mut priv_buf).ok()?;

        let pub_bytes = decode(pub_buf).ok()?;
        let priv_bytes = decode(priv_buf).ok()?;

        let pub_key = PublicKey::from_bytes(&pub_bytes).ok()?;
        let priv_key = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Self {
            public_key: pub_key,
            secret_key: priv_key,
        })
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> std::io::Result<()> {
        fs::write(pub_path, encode(self.public_key.as_bytes()))?;
        fs::write(priv_path, encode(self.secret_key.as_bytes()))?;
        Ok(())
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signature = sign(message, &self.secret_key);
        signature.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = DetachedSignature::from_bytes(signature) {
            sig.verify_detached(message, &self.public_key).is_ok()
        } else {
            false
        }
    }
}