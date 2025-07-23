use base64::{engine::general_purpose, Engine as _};
use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as _, SecretKey as _};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub secret_key: String,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if let Ok(mut file) = File::open("wallet.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap()
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet {
                public_key: general_purpose::STANDARD.encode(pk.as_bytes()),
                secret_key: general_purpose::STANDARD.encode(sk.as_bytes()),
            };
            wallet.save_to_file();
            wallet
        }
    }

    fn save_to_file(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write("wallet.json", json).unwrap();
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = general_purpose::STANDARD.decode(&self.secret_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let signature = sign_detached(message, &sk);
        signature.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature_bytes: &[u8]) -> bool {
        let pk_bytes = general_purpose::STANDARD.decode(&self.public_key).ok()?;
        let pk = PublicKey::from_bytes(&pk_bytes).ok()?;
        let signature = DetachedSignature::from_bytes(signature_bytes).ok()?;
        verify_detached_signature(&signature, message, &pk).is_ok()
    }
}