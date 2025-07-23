use pqcrypto_dilithium::dilithium2::{
    keypair, sign, verify, PublicKey, SecretKey, DetachedSignature,
};
use pqcrypto_traits::sign::{
    SecretKey as SecretKeyTrait,
    PublicKey as PublicKeyTrait,
    DetachedSignature as DetachedSignatureTrait,
};
use base64::{encode, decode};
use std::fs::{self};
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: encode(pk.as_bytes()),
            private_key: encode(sk.as_bytes()),
        }
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        fs::write(pub_path, &self.public_key).expect("Unable to write public key file");
        fs::write(priv_path, &self.private_key).expect("Unable to write private key file");
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Self {
        let public_key = fs::read_to_string(pub_path).expect("Unable to read public key file");
        let private_key = fs::read_to_string(priv_path).expect("Unable to read private key file");
        Wallet { public_key, private_key }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let sig = sign(message, &sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let pub_bytes = decode(&self.public_key).unwrap();
        let sig = DetachedSignature::from_bytes(signature).unwrap();
        let pk = PublicKey::from_bytes(&pub_bytes).unwrap();
        verify(message, &sig, &pk).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }
}