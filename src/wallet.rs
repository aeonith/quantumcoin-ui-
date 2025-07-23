use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::*;
use base64::{encode, decode};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::Path;
use std::sync::Mutex;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn load_or_create() -> Self {
        if Path::new("wallet/keys").exists() {
            Self::load_from_files().expect("Failed to load keys")
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet { public_key: pk, secret_key: sk };
            wallet.save_to_files().expect("Failed to save keys");
            wallet
        }
    }

    pub fn address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_message(&self, msg: &[u8]) -> Vec<u8> {
        let signed = sign(msg, &self.secret_key);
        signed.as_bytes().to_vec()
    }

    pub fn verify_message(&self, msg: &[u8], signature: &[u8]) -> bool {
        let sig = match SignedMessage::from_bytes(signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        verify(&sig, &self.public_key).map_or(false, |recovered| recovered.as_bytes() == msg)
    }

    fn save_to_files(&self) -> std::io::Result<()> {
        create_dir_all("wallet")?;
        write("wallet/public.key", encode(self.public_key.as_bytes()))?;
        write("wallet/secret.key", encode(self.secret_key.as_bytes()))?;
        Ok(())
    }

    fn load_from_files() -> Result<Self, Box<dyn std::error::Error>> {
        let pub_b64 = read_to_string("wallet/public.key")?;
        let sec_b64 = read_to_string("wallet/secret.key")?;
        let pk = PublicKey::from_bytes(&decode(pub_b64.trim())?)?;
        let sk = SecretKey::from_bytes(&decode(sec_b64.trim())?)?;
        Ok(Wallet { public_key: pk, secret_key: sk })
    }
}