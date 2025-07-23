use pqcrypto_dilithium::dilithium2::{keypair, detached_sign, verify_detached_signature, PublicKey, SecretKey, DetachedSignature};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, DetachedSignature as _};
use base64::{engine::general_purpose, Engine as _};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn load_from_files() -> Self {
        let mut pk_file = File::open("wallet/public.key").expect("Missing public key file");
        let mut sk_file = File::open("wallet/secret.key").expect("Missing secret key file");

        let mut pk_base64 = String::new();
        let mut sk_base64 = String::new();

        pk_file.read_to_string(&mut pk_base64).unwrap();
        sk_file.read_to_string(&mut sk_base64).unwrap();

        let pk_bytes = general_purpose::STANDARD.decode(pk_base64.trim()).unwrap();
        let sk_bytes = general_purpose::STANDARD.decode(sk_base64.trim()).unwrap();

        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();

        Wallet { public_key: pk, secret_key: sk }
    }

    pub fn generate_and_save() -> Self {
        let (pk, sk) = keypair();
        create_dir_all("wallet").unwrap();

        let pk_base64 = general_purpose::STANDARD.encode(pk.as_bytes());
        let sk_base64 = general_purpose::STANDARD.encode(sk.as_bytes());

        File::create("wallet/public.key").unwrap().write_all(pk_base64.as_bytes()).unwrap();
        File::create("wallet/secret.key").unwrap().write_all(sk_base64.as_bytes()).unwrap();

        Wallet { public_key: pk, secret_key: sk }
    }

    pub fn sign(&self, message: &[u8]) -> DetachedSignature {
        detached_sign(message, &self.secret_key)
    }

    pub fn verify(&self, message: &[u8], signature: &DetachedSignature) -> bool {
        verify_detached_signature(message, signature, &self.public_key).is_ok()
    }

    pub fn get_address(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }
}