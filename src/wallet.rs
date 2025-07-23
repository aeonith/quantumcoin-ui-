use pqcrypto_dilithium::dilithium2::{keypair, sign, PublicKey, SecretKey, DetachedSignature, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait, SignedMessage as SignedMessageTrait};
use base64::{encode, decode};
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: encode(pk.as_bytes()),
            private_key: encode(sk.as_bytes()),
        }
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        std::fs::write(pub_path, &self.public_key).unwrap();
        std::fs::write(priv_path, &self.private_key).unwrap();
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Self {
        let public_key = std::fs::read_to_string(pub_path).unwrap_or_default();
        let private_key = std::fs::read_to_string(priv_path).unwrap_or_default();
        Wallet { public_key, private_key }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let sig: SignedMessage = sign(message, &sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let pub_bytes = decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pub_bytes).unwrap();
        match DetachedSignature::from_bytes(signature) {
            Ok(sig) => pk.verify_detached(&sig, message).is_ok(),
            Err(_) => false,
        }
    }
}