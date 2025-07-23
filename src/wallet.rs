use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, detached_verify, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{DetachedSignature as SigTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
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
        if let Ok(mut file) = File::open("wallet_key.json") {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet {
                public_key: encode(pk.as_bytes()),
                private_key: encode(sk.as_bytes()),
            };
            let json = serde_json::to_string_pretty(&wallet).unwrap();
            let mut file = File::create("wallet_key.json").unwrap();
            file.write_all(json.as_bytes()).unwrap();
            wallet
        }
    }

    pub fn sign_message(&self, msg: &[u8]) -> DetachedSignature {
        let sk_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        detached_sign(msg, &sk)
    }

    pub fn verify_signature(&self, msg: &[u8], sig: &DetachedSignature) -> bool {
        let pk_bytes = decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        detached_verify(msg, sig, &pk).is_ok()
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