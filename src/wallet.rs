use base64::{engine::general_purpose, Engine};
use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, verify_detached_signature, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, DetachedSignature as _};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub secret_key: String,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if let Ok(mut f) = File::open("wallet.json") {
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            serde_json::from_str(&s).unwrap_or_else(|_| Self::generate())
        } else {
            Self::generate()
        }
    }

    fn generate() -> Self {
        let (pk, sk) = keypair();
        let pub_b64 = general_purpose::STANDARD.encode(pk.as_bytes());
        let sec_b64 = general_purpose::STANDARD.encode(sk.as_bytes());
        let w = Wallet {
            public_key: pub_b64,
            secret_key: sec_b64,
        };
        fs::write("wallet.json", serde_json::to_string_pretty(&w).unwrap()).unwrap();
        w
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let sk_bytes = general_purpose::STANDARD.decode(&self.secret_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let sig: DetachedSignature = detached_sign(data, &sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify(&self, data: &[u8], sig: &[u8]) -> bool {
        let pk_bytes = general_purpose::STANDARD.decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        let ds = DetachedSignature::from_bytes(sig).unwrap();
        verify_detached_signature(&ds, data, &pk).is_ok()
    }

    pub fn address(&self) -> String {
        self.public_key.clone()
    }
}