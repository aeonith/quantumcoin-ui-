use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::collections::HashMap;
use otpauth::TOTP;
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub email: String,
    pub password_hash: String,
    pub wallet_address: String,
    pub two_fa_secret: String,
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    format!("{:x}", hasher.finalize())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

pub fn generate_2fa_secret(email: &str) -> String {
    let totp = TOTP::new("SHA1", 6, 1, 30, email.as_bytes().to_vec());
    totp.get_url(email, "QuantumCoin")
}

pub fn verify_2fa(secret_url: &str, token: &str) -> bool {
    if let Ok(totp) = TOTP::from_url(secret_url) {
        return totp.verify(token);
    }
    false
}

pub fn load_users() -> HashMap<String, User> {
    if let Ok(mut file) = File::open("users.json") {
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        serde_json::from_str(&data).unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    }
}

pub fn save_users(users: &HashMap<String, User>) {
    if let Ok(json) = serde_json::to_string_pretty(users) {
        let mut file = File::create("users.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}