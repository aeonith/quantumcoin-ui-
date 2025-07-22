use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey, DetachedSignature};
use base64::{encode, decode};
use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::Path;
use std::fmt;

#[derive(Clone)]
pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet { public_key: pk, secret_key: sk }
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign(&self, data: &[u8]) -> String {
        let sig = sign_detached(data, &self.secret_key);
        encode(sig.as_bytes())
    }

    pub fn verify(&self, data: &[u8], signature_b64: &str) -> bool {
        if let Ok(sig_bytes) = decode(signature_b64) {
            if let Ok(sig) = DetachedSignature::from_bytes(&sig_bytes) {
                return verify_detached(&sig, data, &self.public_key).is_ok();
            }
        }
        false
    }

    pub fn create_transaction(&self, recipient: &str, amount: f64) -> crate::blockchain::Transaction {
        let data = format!("{}{}{}", self.get_address(), recipient, amount);
        let signature = self.sign(data.as_bytes());

        crate::blockchain::Transaction {
            sender: self.get_address(),
            recipient: recipient.to_string(),
            amount,
            signature: Some(signature),
        }
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        fs::write(pub_path, encode(self.public_key.as_bytes())).unwrap();
        fs::write(priv_path, encode(self.secret_key.as_bytes())).unwrap();
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        if !Path::new(pub_path).exists() || !Path::new(priv_path).exists() {
            return None;
        }

        let mut pub_encoded = String::new();
        let mut priv_encoded = String::new();

        File::open(pub_path).unwrap().read_to_string(&mut pub_encoded).unwrap();
        File::open(priv_path).unwrap().read_to_string(&mut priv_encoded).unwrap();

        let pub_bytes = decode(pub_encoded.trim()).ok()?;
        let priv_bytes = decode(priv_encoded.trim()).ok()?;

        let public_key = PublicKey::from_bytes(&pub_bytes).ok()?;
        let secret_key = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Wallet { public_key, secret_key })
    }

    pub fn export_with_2fa(&self, code: &str) -> bool {
        if code.trim() == "123456" {
            let backup_dir = "wallet_backup";
            fs::create_dir_all(backup_dir).unwrap();

            let pub_path = format!("{}/pub.key", backup_dir);
            let priv_path = format!("{}/priv.key", backup_dir);
            self.save_to_files(&pub_path, &priv_path);
            true
        } else {
            false
        }
    }
}

impl fmt::Debug for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wallet {{ address: {} }}", self.get_address())
    }
}