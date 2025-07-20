use pqcrypto_dilithium::dilithium2::{keypair, sign, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{DetachedSignature, Signature, Signer, Verifier};
use base64::{encode, decode};
use std::fs::{File};
use std::io::{Write, Read};
use std::path::Path;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
            balance: 0.0,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        sign(message, &self.secret_key).as_bytes().to_vec()
    }

    pub fn save_to_files(&self) -> std::io::Result<()> {
        let pub_key_str = encode(self.public_key.as_bytes());
        let sec_key_str = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create("wallet_public.key")?;
        pub_file.write_all(pub_key_str.as_bytes())?;

        let mut sec_file = File::create("wallet_private.key")?;
        sec_file.write_all(sec_key_str.as_bytes())?;

        Ok(())
    }

    pub fn load_from_files() -> std::io::Result<Self> {
        let mut pub_file = File::open("wallet_public.key")?;
        let mut sec_file = File::open("wallet_private.key")?;

        let mut pub_key_encoded = String::new();
        let mut sec_key_encoded = String::new();

        pub_file.read_to_string(&mut pub_key_encoded)?;
        sec_file.read_to_string(&mut sec_key_encoded)?;

        let pub_bytes = decode(pub_key_encoded.trim()).expect("Failed to decode public key");
        let sec_bytes = decode(sec_key_encoded.trim()).expect("Failed to decode private key");

        let public_key = PublicKey::from_bytes(&pub_bytes).expect("Invalid public key");
        let secret_key = SecretKey::from_bytes(&sec_bytes).expect("Invalid secret key");

        Ok(Wallet {
            public_key,
            secret_key,
            balance: 0.0,
        })
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }
}