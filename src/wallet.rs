use pqcrypto_dilithium::dilithium2::{keypair, sign, verify, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature as TraitDetachedSignature, PublicKey as _, SecretKey as _};
use base64::{encode, decode};
use std::fs::{File};
use std::io::{Read, Write};
use std::path::Path;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn get_address_base64(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_public_key_base64(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_private_key_base64(&self) -> String {
        encode(self.secret_key.as_bytes())
    }

    pub fn create_transaction(&self, recipient: &str, amount: u64, sender: &str) -> crate::transaction::Transaction {
        let message = format!("{}{}{}", sender, recipient, amount);
        let signature = sign(message.as_bytes(), &self.secret_key);
        crate::transaction::Transaction {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            amount,
            signature: Some(encode(signature.as_bytes())),
        }
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = pqcrypto_dilithium::dilithium2::DetachedSignature::from_bytes(signature) {
            verify(message, &sig, &self.public_key).is_ok()
        } else {
            false
        }
    }
}