use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _};
use std::fs::{self, File};
use std::io::{Read, Write};
use base64::{encode, decode};

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: u64,
}

impl Wallet {
    pub fn new() -> Self {
        let (public_key, secret_key) = keypair();
        Wallet {
            public_key,
            secret_key,
            balance: 1_250_000,
        }
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn save_to_file(&self, filename: &str) {
        let pub_encoded = encode(self.public_key.as_bytes());
        let sec_encoded = encode(self.secret_key.as_bytes());

        let data = format!("{}\n{}\n{}", pub_encoded, sec_encoded, self.balance);
        fs::write(filename, data).expect("❌ Failed to save wallet");
    }

    pub fn load_from_file(filename: &str) -> Option<Self> {
        let mut file = File::open(filename).ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;

        let mut lines = content.lines();
        let pub_line = lines.next()?;
        let sec_line = lines.next()?;
        let balance_line = lines.next()?;

        let pub_bytes = decode(pub_line).ok()?;
        let sec_bytes = decode(sec_line).ok()?;
        let balance = balance_line.parse::<u64>().ok()?;

        Some(Wallet {
            public_key: PublicKey::from_bytes(&pub_bytes).ok()?,
            secret_key: SecretKey::from_bytes(&sec_bytes).ok()?,
            balance,
        })
    }
}