use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode};

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

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }
}