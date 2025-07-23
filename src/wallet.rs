use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey, DetachedSignature};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if Path::new("wallet_public.key").exists() && Path::new("wallet_private.key").exists() {
            Self::load_from_files()
        } else {
            Self::generate()
        }
    }

    pub fn load_from_files() -> Self {
        let mut pub_file = File::open("wallet_public.key").expect("Failed to open public key file");
        let mut priv_file = File::open("wallet_private.key").expect("Failed to open private key file");

        let mut pub_key = String::new();
        let mut priv_key = String::new();

        pub_file.read_to_string(&mut pub_key).unwrap();
        priv_file.read_to_string(&mut priv_key).unwrap();

        Wallet {
            public_key: pub_key,
            private_key: priv_key,
        }
    }

    pub fn generate() -> Self {
        let (pk, sk) = keypair();

        let pk_base64 = general_purpose::STANDARD.encode(pk.as_bytes());
        let sk_base64 = general_purpose::STANDARD.encode(sk.as_bytes());

        fs::write("wallet_public.key", &pk_base64).expect("Failed to save public key");
        fs::write("wallet_private.key", &sk_base64).expect("Failed to save private key");

        Wallet {
            public_key: pk_base64,
            private_key: sk_base64,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = general_purpose::STANDARD
            .decode(&self.private_key)
            .expect("Invalid private key encoding");
        let sk = SecretKey::from_bytes(&sk_bytes).expect("Failed to parse secret key");

        let signature = sign_detached(message, &sk);
        signature.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature_bytes: &[u8]) -> bool {
        let pk_bytes = match general_purpose::STANDARD.decode(&self.public_key) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let pk = match PublicKey::from_bytes(&pk_bytes) {
            Ok(key) => key,
            Err(_) => return false,
        };

        let signature = match DetachedSignature::from_bytes(signature_bytes) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        verify_detached(&signature, message, &pk).is_ok()
    }
}