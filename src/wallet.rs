use pqcrypto_dilithium::dilithium2::{
    keypair, detached_sign, verify_detached_signature, PublicKey, SecretKey, DetachedSignature,
};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _, SecretKey as _};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;

#[derive(Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn new() -> Self {
        let pub_path = "wallet_public.key";
        let priv_path = "wallet_private.key";

        if Path::new(pub_path).exists() && Path::new(priv_path).exists() {
            let public_key = read_to_string(pub_path).unwrap_or_default();
            let private_key = read_to_string(priv_path).unwrap_or_default();
            return Wallet { public_key, private_key };
        }

        let (pk, sk) = keypair();
        let public_key = general_purpose::STANDARD.encode(pk.as_bytes());
        let private_key = general_purpose::STANDARD.encode(sk.as_bytes());

        let _ = File::create(pub_path).and_then(|mut f| f.write_all(public_key.as_bytes()));
        let _ = File::create(priv_path).and_then(|mut f| f.write_all(private_key.as_bytes()));

        Wallet { public_key, private_key }
    }

    pub fn new_and_save() -> Self {
        let wallet = Self::new();
        wallet.save_to_files("public.key", "private.key");
        wallet
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let pub_key = std::fs::read_to_string(pub_path).ok()?;
        let priv_key = std::fs::read_to_string(priv_path).ok()?;
        Some(Wallet {
            public_key: pub_key,
            private_key: priv_key,
        })
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        let _ = File::create(pub_path).and_then(|mut f| f.write_all(self.public_key.as_bytes()));
        let _ = File::create(priv_path).and_then(|mut f| f.write_all(self.private_key.as_bytes()));
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = general_purpose::STANDARD.decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let sig = detached_sign(message, &sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let pk_bytes = general_purpose::STANDARD.decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        let sig = DetachedSignature::from_bytes(signature).unwrap();
        verify_detached_signature(&sig, message, &pk).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn get_public_key(&self) -> String {
        self.public_key.clone()
    }

    pub fn get_private_key(&self) -> String {
        self.private_key.clone()
    }
} 