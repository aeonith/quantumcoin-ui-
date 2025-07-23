use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
use base64::{engine::general_purpose, Engine as _};
use serde::{Serialize, Deserialize};
use std::fs::{File};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if let Ok(mut file) = File::open("wallet_key.json") {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet {
                public_key: general_purpose::STANDARD.encode(pk.as_bytes()),
                private_key: general_purpose::STANDARD.encode(sk.as_bytes()),
            };
            wallet.save_to_files();
            wallet
        }
    }

    pub fn sign_message(&self, msg: &[u8]) -> DetachedSignature {
        let sk_bytes = general_purpose::STANDARD.decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        detached_sign(msg, &sk)
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn save_to_files(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        let mut file = File::create("wallet_key.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    pub fn load_from_files() -> Self {
        let mut file = File::open("wallet_key.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        serde_json::from_str(&data).unwrap()
    }
}