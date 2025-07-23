use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, PublicKey, SecretKey};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, Signature, Sign};
use base64::{encode, decode};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if let Ok(mut file) = File::open("wallet_key.json") {
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            serde_json::from_str(&data).unwrap()
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet {
                public_key: encode(pk.as_bytes()),
                private_key: encode(sk.as_bytes()),
            };
            wallet.save_to_file();
            wallet
        }
    }

    fn save_to_file(&self) {
        let serialized = serde_json::to_string_pretty(self).unwrap();
        let _ = create_dir_all(".");
        let mut file = File::create("wallet_key.json").unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let sk_bytes = decode(&self.private_key).unwrap();
        let sk = TraitSecretKey::from_bytes(&sk_bytes).unwrap();
        detached_sign(message, &sk).as_bytes().to_vec()
    }

    pub fn verify_signature(&self, message: &[u8], sig: &[u8]) -> bool {
        let pub_bytes = decode(&self.public_key).unwrap();
        let sig = Signature::from_bytes(sig).unwrap();
        let pk: PublicKey = TraitPublicKey::from_bytes(&pub_bytes).unwrap();
        pk.verify_detached(&sig, message).is_ok()
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }
}