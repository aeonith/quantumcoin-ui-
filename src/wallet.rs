use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, sign_detached, verify_detached, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use std::fs::{File};
use std::io::{Write, Read};
use std::path::Path;
use crate::transaction::Transaction;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
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
        let decoded_sig = match <SignedMessage as pqcrypto_traits::sign::SignedMessage>::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        verify_detached(&decoded_sig, message, &self.public_key).is_ok()
    }

    pub fn save_to_files(&self) -> std::io::Result<()> {
        let pub_encoded = encode(self.public_key.as_bytes());
        let priv_encoded = encode(self.secret_key.as_bytes());
        let mut pub_file = File::create("wallet_public.key")?;
        let mut priv_file = File::create("wallet_private.key")?;
        pub_file.write_all(pub_encoded.as_bytes())?;
        priv_file.write_all(priv_encoded.as_bytes())?;
        Ok(())
    }

    pub fn load_from_files() -> std::io::Result<Self> {
        let mut pub_encoded = String::new();
        let mut priv_encoded = String::new();
        File::open("wallet_public.key")?.read_to_string(&mut pub_encoded)?;
        File::open("wallet_private.key")?.read_to_string(&mut priv_encoded)?;
        let pub_bytes = decode(&pub_encoded).expect("Invalid public key");
        let priv_bytes = decode(&priv_encoded).expect("Invalid private key");

        Ok(Wallet {
            public_key: PublicKey::from_bytes(&pub_bytes).expect("Invalid public key bytes"),
            secret_key: SecretKey::from_bytes(&priv_bytes).expect("Invalid private key bytes"),
            balance: 0.0,
        })
    }

    pub fn create_transaction(&self, to: &str, amount: f64) -> Transaction {
        let message = format!("{}:{}:{}", self.get_address(), to, amount);
        let signature = self.sign_message(message.as_bytes());
        Transaction {
            sender: self.get_address(),
            recipient: to.to_string(),
            amount,
            signature: encode(&signature),
        }
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn export_with_2fa(&self, password: &str) -> std::io::Result<()> {
        let encrypted = format!(
            "ADDRESS:{}\nPUB:{}\nPRIV:{}\nPASS:{}",
            self.get_address(),
            encode(self.public_key.as_bytes()),
            encode(self.secret_key.as_bytes()),
            password
        );
        let mut file = File::create("wallet_backup.qtc")?;
        file.write_all(encrypted.as_bytes())?;
        Ok(())
    }
}