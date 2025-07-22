use pqcrypto_dilithium::dilithium2::*;
use base64::{encode, decode};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
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

    pub fn load_or_create() -> Self {
        if let Ok(mut file) = File::open("wallet.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap()
        } else {
            let wallet = Wallet::new();
            wallet.save_to_file();
            wallet
        }
    }

    pub fn save_to_file(&self) {
        create_dir_all(".").unwrap();
        let encoded = serde_json::to_string_pretty(self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("wallet.json")
            .unwrap();
        file.write_all(encoded.as_bytes()).unwrap();
    }

    pub fn sign(&self, message: &[u8]) -> String {
        let private_key_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&private_key_bytes).unwrap();
        let signature = sign_detached(message, &sk);
        encode(signature.as_bytes())
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn verify(&self, message: &[u8], signature: &str) -> bool {
        let public_key_bytes = decode(&self.public_key).unwrap();
        let sig_bytes = decode(signature).unwrap();

        let pk = PublicKey::from_bytes(&public_key_bytes).unwrap();
        let sig = Signature::from_bytes(&sig_bytes).unwrap();
        verify_detached(&sig, message, &pk).is_ok()
    }
}