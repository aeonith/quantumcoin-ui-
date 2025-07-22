use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as TraitPub, SecretKey as TraitSec};
use base64::{encode, decode};
use otpauth::TOTP;
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public_key: String,
    private_key: String,
    pub password: String,
    pub otp_secret: Option<String>,
    pub otp_enabled: bool,
}

impl Wallet {
    pub fn load_or_generate() -> Self {
        if let Ok(data) = fs::read_to_string("wallet_key.json") {
            serde_json::from_str(&data).unwrap()
        } else {
            let (pk, sk) = keypair();
            let wallet = Wallet {
                public_key: encode(pk.as_bytes()),
                private_key: encode(sk.as_bytes()),
                password: "defaultpassword".to_string(),
                otp_secret: None,
                otp_enabled: false,
            };
            wallet.save();
            wallet
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write("wallet_key.json", json).unwrap();
    }

    pub fn verify_password(&self, input: &str) -> bool {
        self.password == input
    }

    pub fn sign_message(&self, msg: &[u8]) -> DetachedSignature {
        let sk_bytes = decode(&self.private_key).unwrap();
        let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        sign_detached(msg, &sk)
    }

    pub fn get_address(&self) -> String {
        self.public_key.clone()
    }

    pub fn enable_2fa(&mut self, secret: String) {
        self.otp_secret = Some(secret);
        self.otp_enabled = true;
        self.save();
    }

    pub fn verify_2fa(&self, code: &str) -> bool {
        if let Some(secret) = &self.otp_secret {
            let totp = TOTP::from_base32(secret).unwrap();
            totp.verify(code)
        } else {
            false
        }
    }

    pub fn is_2fa_enabled(&self) -> bool {
        self.otp_enabled
    }

    pub fn get_2fa_qr_url(&self) -> Option<String> {
        self.otp_secret.as_ref().map(|secret| {
            let totp = TOTP::from_base32(secret).unwrap();
            totp.get_url("QuantumCoin", "user@quantumcoin.com")
        })
    }
}