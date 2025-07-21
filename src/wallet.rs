use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, Signer, Verifier};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;
use base64::{encode, decode};

pub struct Wallet {
    pub public_key: dilithium2::PublicKey,
    pub secret_key: dilithium2::SecretKey,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = dilithium2::keypair();
        Wallet { public_key: pk, secret_key: sk }
    }

    pub fn save_to_files(&self, directory: &str) {
        create_dir_all(directory).unwrap();

        let pk_encoded = encode(self.public_key.as_bytes());
        let sk_encoded = encode(self.secret_key.as_bytes());

        let mut pk_file = File::create(format!("{}/public.key", directory)).unwrap();
        let mut sk_file = File::create(format!("{}/private.key", directory)).unwrap();

        pk_file.write_all(pk_encoded.as_bytes()).unwrap();
        sk_file.write_all(sk_encoded.as_bytes()).unwrap();
    }

    pub fn load_from_files(directory: &str) -> Option<Self> {
        let pk_path = format!("{}/public.key", directory);
        let sk_path = format!("{}/private.key", directory);

        if !Path::new(&pk_path).exists() || !Path::new(&sk_path).exists() {
            return None;
        }

        let mut pk_data = String::new();
        let mut sk_data = String::new();

        File::open(pk_path).unwrap().read_to_string(&mut pk_data).unwrap();
        File::open(sk_path).unwrap().read_to_string(&mut sk_data).unwrap();

        let pk_bytes = decode(pk_data.trim()).unwrap();
        let sk_bytes = decode(sk_data.trim()).unwrap();

        let public_key = dilithium2::PublicKey::from_bytes(&pk_bytes).unwrap();
        let secret_key = dilithium2::SecretKey::from_bytes(&sk_bytes).unwrap();

        Some(Wallet { public_key, secret_key })
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_message(&self, message: &[u8]) -> DetachedSignature {
        self.secret_key.sign_detached(message)
    }

    pub fn verify_signature(&self, message: &[u8], signature: &DetachedSignature) -> bool {
        self.public_key.verify_detached(message, signature).is_ok()
    }

    pub fn export_with_2fa(&self, password: &str) -> String {
        let encrypted = format!(
            "{}:{}",
            encode(self.public_key.as_bytes()),
            base64::encode(format!("{}::{}", password, encode(self.secret_key.as_bytes())))
        );
        encrypted
    }
}