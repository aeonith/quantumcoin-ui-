use pqcrypto_dilithium::dilithium3::{keypair, sign_detached, verify_detached, PublicKey, SecretKey, Signature};
use rand::{rngs::OsRng, RngCore};
use ring::{pbkdf2, aead::{Aad, LessSafeKey, UnboundKey, AES_256_GCM, Nonce}};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::num::NonZeroU32;
use base64::{encode, decode};

const SEED_FILE: &str = "wallet/seedphrase.txt";
const PRIVATE_FILE: &str = "wallet/private_encrypted.bin";
const PUBLIC_FILE: &str = "wallet/public.key";
const SALT: &[u8] = b"QuantumCoinSalt";
const ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(100_000) };

pub struct SecureWallet {
    pub public_key: PublicKey,
    secret_key: Option<SecretKey>,
}

impl SecureWallet {
    pub fn generate(password: &str) -> Self {
        let (pk, sk) = keypair();
        let seed_phrase = SecureWallet::generate_seed_phrase();
        fs::create_dir_all("wallet").unwrap();
        fs::write(SEED_FILE, &seed_phrase).unwrap();

        let encrypted_sk = encrypt_secret(&sk.as_bytes(), password);
        fs::write(PRIVATE_FILE, encrypted_sk).unwrap();
        fs::write(PUBLIC_FILE, pk.as_bytes()).unwrap();

        println!("🧠 New wallet created.");
        println!("🔐 Seed phrase saved: {}", seed_phrase);
        SecureWallet { public_key: pk, secret_key: Some(sk) }
    }

    pub fn load(password: &str) -> Option<Self> {
        let encrypted = fs::read(PRIVATE_FILE).ok()?;
        let decrypted = decrypt_secret(&encrypted, password)?;
        let sk = SecretKey::from_bytes(&decrypted).ok()?;

        let public_bytes = fs::read(PUBLIC_FILE).ok()?;
        let pk = PublicKey::from_bytes(&public_bytes).ok()?;

        Some(SecureWallet {
            public_key: pk,
            secret_key: Some(sk),
        })
    }

    pub fn sign_message(&self, message: &[u8]) -> Option<String> {
        self.secret_key.as_ref().map(|sk| {
            let sig = sign_detached(message, sk);
            encode(sig.as_bytes())
        })
    }

    pub fn verify_signature(&self, message: &[u8], signature_b64: &str) -> bool {
        if let Ok(sig_bytes) = decode(signature_b64) {
            if let Ok(sig) = Signature::from_bytes(&sig_bytes) {
                return verify_detached(&sig, message, &self.public_key).is_ok();
            }
        }
        false
    }

    pub fn get_address(&self) -> String {
        let hash = ring::digest::digest(&ring::digest::SHA256, self.public_key.as_bytes());
        encode(&hash.as_ref()[..20])
    }

    fn generate_seed_phrase() -> String {
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        hex::encode(entropy)
    }
}

fn encrypt_secret(secret: &[u8], password: &str) -> Vec<u8> {
    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        ITERATIONS,
        SALT,
        password.as_bytes(),
        &mut key,
    );

    let unbound_key = UnboundKey::new(&AES_256_GCM, &key).unwrap();
    let cipher = LessSafeKey::new(unbound_key);
    let nonce = Nonce::assume_unique_for_key([0u8; 12]);
    let mut in_out = secret.to_vec();
    cipher.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out).unwrap();
    in_out
}

fn decrypt_secret(encrypted: &[u8], password: &str) -> Option<Vec<u8>> {
    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        ITERATIONS,
        SALT,
        password.as_bytes(),
        &mut key,
    );

    let unbound_key = UnboundKey::new(&AES_256_GCM, &key).ok()?;
    let cipher = LessSafeKey::new(unbound_key);
    let nonce = Nonce::assume_unique_for_key([0u8; 12]);

    let mut in_out = encrypted.to_vec();
    cipher.open_in_place(nonce, Aad::empty(), &mut in_out).ok()?;
    let len = in_out.len() - 16;
    Some(in_out[..len].to_vec())
}