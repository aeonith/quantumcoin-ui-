use pqcrypto_dilithium::dilithium2::{
    keypair, sign, PublicKey, SecretKey, DetachedSignature, SignedMessage,
};
use pqcrypto_traits::sign::{
    SecretKey as SecretKeyTrait,
    PublicKey as PublicKeyTrait,
    DetachedSignature as DetachedSigTrait,
    SignedMessage as SignedMsgTrait,
};
use base64::{encode, decode};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: encode(pk.as_bytes()),
            private_key: encode(sk.as_bytes()),
        }
    }

    pub fn save_to_files(&self, pub_path: &str, priv_path: &str) {
        let mut pub_file = File::create(pub_path).unwrap();
        let mut priv_file = File::create(priv_path).unwrap();
        pub_file.write_all(self.public_key.as_bytes()).unwrap();
        priv_file.write_all(self.private_key.as_bytes()).unwrap();
    }

    pub fn load_from_files(pub_path: &str, priv_path: &str) -> Self {
        let mut pub_file = File::open(pub_path).unwrap();
        let mut priv_file = File::open(priv_path).unwrap();
        let mut pub_key = String::new();
        let mut priv_key = String::new();
        pub_file.read_to_string(&mut pub_key).unwrap();
        priv_file.read_to_string(&mut priv_key).unwrap();
        Wallet {
            public_key: pub_key,
            private_key: priv_key,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        let signed = sign(message, &sk);
        signed.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let pk_bytes = decode(&self.public_key).unwrap();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();

        match DetachedSignature::from_bytes(signature) {
            Ok(sig) => pk.verify_detached(&sig, message).is_ok(),
            Err(_) => false,
        }
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }
}