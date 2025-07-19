use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, sign_detached, verify_detached};
use base64::{engine::general_purpose, Engine};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    public_key: PublicKey,
    secret_key: SecretKey,
    balance: f64,
}

impl Wallet {
    pub fn new() -> Wallet {
        let (pk, sk) = keypair();
        Wallet { public_key: pk, secret_key: sk, balance: 0.0 }
    }

    pub fn get_address(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        let pk = general_purpose::STANDARD.encode(self.public_key.as_bytes());
        let sk = general_purpose::STANDARD.encode(self.secret_key.as_bytes());
        File::create(pub_path).unwrap().write_all(pk.as_bytes()).unwrap();
        File::create(priv_path).unwrap().write_all(sk.as_bytes()).unwrap();
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Wallet> {
        let mut pk_buf = String::new();
        let mut sk_buf = String::new();
        File::open(pub_path).ok()?.read_to_string(&mut pk_buf).ok()?;
        File::open(priv_path).ok()?.read_to_string(&mut sk_buf).ok()?;
        let pk_bytes = general_purpose::STANDARD.decode(pk_buf).ok()?;
        let sk_bytes = general_purpose::STANDARD.decode(sk_buf).ok()?;
        let public_key = PublicKey::from_bytes(&pk_bytes).ok()?;
        let secret_key = SecretKey::from_bytes(&sk_bytes).ok()?;
        Some(Wallet { public_key, secret_key, balance: 0.0 })
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let sig = sign_detached(msg, &self.secret_key);
        sig.as_bytes().to_vec()
    }

    pub fn verify(msg: &[u8], sig: &[u8], pk: &PublicKey) -> bool {
        let s = pqcrypto_traits::sign::Signature::from_bytes(sig).ok()?;
        verify_detached(&s, msg, pk).is_ok()
    }
}