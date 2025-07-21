use pqcrypto_dilithium::dilithium2::{
    keypair, detached_sign, verify_detached, PublicKey, SecretKey, DetachedSignature,
};
use std::fs::{File};
use std::io::{Read, Write};
use crate::transaction::Transaction;
use serde_json;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
        }
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> std::io::Result<()> {
        let mut pub_file = File::create(pub_path)?;
        let mut priv_file = File::create(priv_path)?;
        pub_file.write_all(self.public_key.as_bytes())?;
        priv_file.write_all(self.secret_key.as_bytes())?;
        Ok(())
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> std::io::Result<Self> {
        let mut pub_bytes = Vec::new();
        let mut priv_bytes = Vec::new();
        File::open(pub_path)?.read_to_end(&mut pub_bytes)?;
        File::open(priv_path)?.read_to_end(&mut priv_bytes)?;

        let public_key = PublicKey::from_bytes(&pub_bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid public key"))?;
        let secret_key = SecretKey::from_bytes(&priv_bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid secret key"))?;

        Ok(Wallet {
            public_key,
            secret_key,
        })
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signature = detached_sign(message, &self.secret_key);
        signature.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = DetachedSignature::from_bytes(signature) {
            verify_detached(&sig, message, &self.public_key).is_ok()
        } else {
            false
        }
    }

    pub fn sign_transaction(&self, tx: &Transaction) -> Vec<u8> {
        let serialized = serde_json::to_vec(tx).expect("Transaction serialization failed");
        self.sign_message(&serialized)
    }
}