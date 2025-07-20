use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Wallet {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
            balance: 0.0,
        }
    }

    pub fn save_to_files(&self) {
        let pub_path = "wallet_public.key";
        let priv_path = "wallet_private.key";

        let pub_encoded = encode(self.public_key.as_bytes());
        let priv_encoded = encode(self.secret_key.as_bytes());

        fs::write(pub_path, pub_encoded).expect("Failed to save public key");
        fs::write(priv_path, priv_encoded).expect("Failed to save private key");
    }

    pub fn load_from_files() -> Wallet {
        let pub_path = "wallet_public.key";
        let priv_path = "wallet_private.key";

        let pub_encoded = fs::read_to_string(pub_path).expect("Failed to read public key file");
        let priv_encoded = fs::read_to_string(priv_path).expect("Failed to read private key file");

        let pub_decoded = decode(pub_encoded.trim()).expect("Base64 decode failed");
        let priv_decoded = decode(priv_encoded.trim()).expect("Base64 decode failed");

        let public_key = PublicKey::from_bytes(&pub_decoded).expect("Invalid public key");
        let secret_key = SecretKey::from_bytes(&priv_decoded).expect("Invalid secret key");

        Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        }
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signature = sign_detached(message, &self.secret_key);
        signature.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        match pqcrypto_dilithium::dilithium2::DetachedSignature::from_bytes(signature) {
            Ok(sig) => verify_detached(&sig, message, &self.public_key).is_ok(),
            Err(_) => false,
        }
    }

    pub fn get_keys() -> (String, String) {
        let wallet = Wallet::load_from_files();
        let pub_str = encode(wallet.public_key.as_bytes());
        let priv_str = encode(wallet.secret_key.as_bytes());
        (pub_str, priv_str)
    }
}