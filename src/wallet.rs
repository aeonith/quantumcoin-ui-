use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, DetachedSignature, PublicKey, SecretKey};
use pqcrypto_traits::sign::{Signer, Verifier, Signature};
use std::fs::{File};
use std::io::{Read, Write};
use std::path::Path;
use base64::{encode, decode};

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Wallet { public_key: pk, secret_key: sk }
    }

    pub fn save_to_files(&self, pub_path: &str, sec_path: &str) {
        let mut pub_file = File::create(pub_path).expect("Failed to create pub key file");
        pub_file.write_all(&self.public_key.as_bytes()).expect("Failed to write pub key");

        let mut sec_file = File::create(sec_path).expect("Failed to create sec key file");
        sec_file.write_all(&self.secret_key.as_bytes()).expect("Failed to write sec key");
    }

    pub fn load_from_files(pub_path: &str, sec_path: &str) -> Self {
        let mut pub_bytes = Vec::new();
        File::open(pub_path).unwrap().read_to_end(&mut pub_bytes).unwrap();
        let public_key = PublicKey::from_bytes(&pub_bytes).unwrap();

        let mut sec_bytes = Vec::new();
        File::open(sec_path).unwrap().read_to_end(&mut sec_bytes).unwrap();
        let secret_key = SecretKey::from_bytes(&sec_bytes).unwrap();

        Wallet { public_key, secret_key }
    }

    pub fn get_address_base64(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_public_key_base64(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_private_key_base64(&self) -> String {
        encode(self.secret_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> DetachedSignature {
        sign_detached(message, &self.secret_key)
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = DetachedSignature::from_bytes(signature) {
            sig.verify_detached(message, &self.public_key).is_ok()
        } else {
            false
        }
    }
}