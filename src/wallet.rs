use serde::{Serialize, Deserialize};
use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{DetachedSignature, Signature, Signer, Verifier};
use base64::{encode, decode};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
    #[serde(skip_serializing)]
    pub password: Option<String>, // Used for login verification (simple placeholder)
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: encode(pk.as_bytes()),
            private_key: encode(sk.as_bytes()),
            password: None,
        }
    }

    pub fn generate_with_password(password: &str) -> Self {
        let mut wallet = Wallet::generate();
        wallet.password = Some(password.to_string());
        wallet
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn sign_detached(&self, message: &[u8]) -> DetachedSignature {
        let sk_bytes = decode(&self.private_key).expect("Invalid private key");
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        sign_detached(message, &sk)
    }

    pub fn verify_signature(&self, message: &[u8], sig: &DetachedSignature) -> bool {
        let pk_bytes = decode(&self.public_key).expect("Invalid public key");
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        verify_detached(sig, message, &pk).is_ok()
    }

    pub fn save_to_file(&self, path: &str, _password: &str) {
        // TODO: encryption with password (future)
        let data = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, data).unwrap();
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        if !Path::new(path).exists() {
            return None;
        }
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn verify_password(&self, input: &str) -> bool {
        match &self.password {
            Some(p) => p == input,
            None => false,
        }
    }

    pub fn create_transaction(&self, recipient: &str, amount: u64) -> crate::transaction::Transaction {
        crate::transaction::Transaction {
            sender: self.get_address(),
            recipient: recipient.to_string(),
            amount,
            timestamp: crate::blockchain::now(),
        }
    }
}