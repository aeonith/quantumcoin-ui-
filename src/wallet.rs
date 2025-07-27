use pqcrypto_dilithium::dilithium2::{keypair, sign, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _};
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::io::{Read, Write};

#[derive(Clone)]
pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    /// Creates a new wallet (new keypair).
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (pk, sk) = keypair();
        Ok(Self {
            public_key: pk,
            secret_key: sk,
        })
    }

    /// Load an existing wallet from files (base64 encoded).
    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Option<Self> {
        let mut pub_buf = String::new();
        let mut priv_buf = String::new();

        let _ = fs::File::open(pub_path).and_then(|mut f| f.read_to_string(&mut pub_buf));
        let _ = fs::File::open(priv_path).and_then(|mut f| f.read_to_string(&mut priv_buf));

        if pub_buf.is_empty() || priv_buf.is_empty() {
            return None;
        }

        let pub_bytes = general_purpose::STANDARD.decode(pub_buf).ok()?;
        let priv_bytes = general_purpose::STANDARD.decode(priv_buf).ok()?;

        let pk = PublicKey::from_bytes(&pub_bytes).ok()?;
        let sk = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Self {
            public_key: pk,
            secret_key: sk,
        })
    }

    /// Save wallet keys to files (base64 encoded).
    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(pub_path, general_purpose::STANDARD.encode(self.public_key.as_bytes()))?;
        fs::write(priv_path, general_purpose::STANDARD.encode(self.secret_key.as_bytes()))?;
        Ok(())
    }

    /// Sign a message.
    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signature: SignedMessage = sign(message, &self.secret_key);
        signature.as_bytes().to_vec()
    }

    /// Verify a message signature.
    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        pqcrypto_dilithium::dilithium2::verify(signature, message, &self.public_key).is_ok()
    }

    /// Return public key as Base64 string.
    pub fn get_public_key(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }

    /// Return private key as Base64 string.
    pub fn get_private_key(&self) -> String {
        general_purpose::STANDARD.encode(self.secret_key.as_bytes())
    }
}