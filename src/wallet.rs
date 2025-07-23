use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, Signer, Verifier};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use base64::{engine::general_purpose, Engine};

#[derive(Serialize, Deserialize)]
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
            let public_key = general_purpose::STANDARD.encode(pk.as_bytes());
            let private_key = general_purpose::STANDARD.encode(sk.as_bytes());
            let wallet = Wallet { public_key, private_key };
            wallet.save_to_file();
            wallet
        }
    }

    pub fn save_to_file(&self) {
        let data = serde_json::to_string_pretty(&self).unwrap();
        fs::write("wallet_key.json", data).unwrap();
    }

    pub fn address(&self) -> String {
        self.public_key.clone()
    }

    pub fn sign_message(&self, message: &str) -> String {
        let sk_bytes = general_purpose::STANDARD.decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let signature = sk.sign_detached(message.as_bytes());
        general_purpose::STANDARD.encode(signature.as_bytes())
    }

    pub fn verify_signature(&self, message: &str, signature_b64: &str) -> bool {
        let pk_bytes = general_purpose::STANDARD.decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        let sig_bytes = general_purpose::STANDARD.decode(signature_b64).unwrap();
        let sig = DetachedSignature::from_bytes(&sig_bytes).unwrap();
        pk.verify_detached(&sig, message.as_bytes()).is_ok()
    }
}