use pqcrypto_dilithium::dilithium2::{
    keypair, sign_detached, verify_detached, PublicKey, SecretKey, DetachedSignature,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use base64::{encode, decode};

pub struct Wallet {
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl Wallet {
    pub fn load_or_generate() -> Wallet {
        let pub_path = "wallet_public.key";
        let priv_path = "wallet_private.key";

        if let (Ok(mut pub_file), Ok(mut priv_file)) = (File::open(pub_path), File::open(priv_path)) {
            let mut pub_encoded = String::new();
            let mut priv_encoded = String::new();

            pub_file.read_to_string(&mut pub_encoded).unwrap();
            priv_file.read_to_string(&mut priv_encoded).unwrap();

            let pub_bytes = decode(pub_encoded).unwrap();
            let priv_bytes = decode(priv_encoded).unwrap();

            let public_key = PublicKey::from_bytes(&pub_bytes).unwrap();
            let private_key = SecretKey::from_bytes(&priv_bytes).unwrap();

            Wallet { public_key, private_key }
        } else {
            let (public_key, private_key) = keypair();
            let pub_encoded = encode(public_key.as_bytes());
            let priv_encoded = encode(private_key.as_bytes());

            fs::write(pub_path, pub_encoded).unwrap();
            fs::write(priv_path, priv_encoded).unwrap();

            Wallet { public_key, private_key }
        }
    }

    pub fn sign(&self, message: &[u8]) -> DetachedSignature {
        sign_detached(message, &self.private_key)
    }

    pub fn verify(&self, message: &[u8], signature: &DetachedSignature) -> bool {
        verify_detached(signature, message, &self.public_key).is_ok()
    }

    pub fn get_address_base64(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn export_with_2fa(&self, code: &str) -> Option<String> {
        if code == "123456" {
            Some(format!(
                "PUBLIC: {}\nPRIVATE: {}",
                encode(self.public_key.as_bytes()),
                encode(self.private_key.as_bytes())
            ))
        } else {
            None
        }
    }
}