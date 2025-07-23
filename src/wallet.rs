use base64::{engine::general_purpose, Engine};
use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, verify_detached_signature, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, DetachedSignature as _};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, create_dir_all, File},
    io::{Read, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub public_key: String,
    pub secret_key: String,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if let Ok(mut f) = File::open("wallet.json") {
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            serde_json::from_str(&s).unwrap_or_else(|_| Self::generate_and_save())
        } else {
            Self::generate_and_save()
        }
    }

    fn generate_and_save() -> Self {
        create_dir_all("wallet").unwrap();
        let (pk, sk) = keypair();
        let pk_b64 = general_purpose::STANDARD.encode(pk.as_bytes());
        let sk_b64 = general_purpose::STANDARD.encode(sk.as_bytes());
        let w = Wallet { public_key: pk_b64, secret_key: sk_b64 };
        fs::write("wallet.json", serde_json::to_string_pretty(&w).unwrap()).unwrap();
        w
    }

    pub fn sign_message(&self, data: &[u8]) -> Vec<u8> {
        let sk_bytes = general_purpose::STANDARD.decode(&self.secret_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let sig: DetachedSignature = detached_sign(data, &sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, data: &[u8], sig_bytes: &[u8]) -> bool {
        let pk_bytes = general_purpose::STANDARD.decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        let sig = DetachedSignature::from_bytes(sig_bytes).unwrap();
        verify_detached_signature(&sig, data, &pk).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }
}