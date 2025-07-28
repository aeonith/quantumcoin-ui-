use pqcrypto_dilithium::dilithium2::{keypair, sign, DetachedSignature, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _, SecretKey as _};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{self, File};
use std::io::{Read, Write};

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (pk, sk) = keypair();
        Ok(Wallet {
            public_key: pk,
            secret_key: sk,
        })
    }

    pub fn get_address_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn get_public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    pub fn get_private_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.secret_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sig = sign::detached_sign(message, &self.secret_key);
        sig.as_bytes().to_vec()
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = <DetachedSignature as pqcrypto_traits::sign::DetachedSignature>::from_bytes(signature) {
            pqcrypto_dilithium::dilithium2::verify_detached(&sig, message, &self.public_key).is_ok()
        } else {
            false
        }
    }

    pub fn save_to_files(
        &self,
        pub_path: &str,
        priv_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(pub_path, self.public_key.as_bytes())?;
        fs::write(priv_path, self.secret_key.as_bytes())?;
        Ok(())
    }

    pub fn load_from_files(
        pub_path: &str,
        priv_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut pub_bytes = Vec::new();
        let mut priv_bytes = Vec::new();

        File::open(pub_path)?.read_to_end(&mut pub_bytes)?;
        File::open(priv_path)?.read_to_end(&mut priv_bytes)?;

        let pub_key = PublicKey::from_bytes(&pub_bytes)?;
        let priv_key = SecretKey::from_bytes(&priv_bytes)?;

        Ok(Wallet {
            public_key: pub_key,
            secret_key: priv_key,
        })
    }
}