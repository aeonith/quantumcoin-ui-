use pqcrypto_dilithium::dilithium2::{
    keypair, sign_detached, PublicKey, SecretKey, DetachedSignature,
};
use base64::{encode, decode};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if let Ok(data) = fs::read_to_string("wallet_key.json") {
            if let Ok(wallet) = serde_json::from_str(&data) {
                return wallet;
            }
        }

        let (pk, sk) = keypair();
        let wallet = Wallet {
            public_key: encode(pk.as_bytes()),
            private_key: encode(sk.as_bytes()),
        };

        let json = serde_json::to_string_pretty(&wallet).unwrap();
        fs::write("wallet_key.json", json).unwrap();
        wallet
    }

    pub fn load_from_files() -> Option<Self> {
        let data = fs::read_to_string("wallet_key.json").ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn new() -> Self {
        Self::load_or_generate()
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let sig = sign_detached(message, &sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let pub_bytes = decode(&self.public_key).unwrap();
        let sig = DetachedSignature::from_bytes(signature).unwrap();
        let pk = PublicKey::from_bytes(&pub_bytes).unwrap();
        pk.verify_detached(&sig, message).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }
}