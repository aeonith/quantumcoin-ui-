use serde::{Serialize, Deserialize};
use std::{fs, path::Path};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub email: String,
    pub password_hash: String,
    pub wallet_file: String,
}

impl User {
    pub fn register(email: &str, password: &str) -> Option<Self> {
        let hash = hash_password(password);
        let wallet_file = format!("wallets/{}.json", base64::encode(email));
        let user = User { email: email.into(), password_hash: hash, wallet_file };
        save_user(&user);
        Some(user)
    }

    pub fn login(email: &str, password: &str) -> bool {
        if let Some(u) = load_user(email) {
            return u.password_hash == hash_password(password);
        }
        false
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn save_user(user: &User) {
    let path = format!("users/{}.json", base64::encode(&user.email));
    let data = serde_json::to_string_pretty(user).unwrap();
    fs::write(path, data).unwrap();
}

pub fn load_user(email: &str) -> Option<User> {
    let path = format!("users/{}.json", base64::encode(email));
    if !Path::new(&path).exists() { return None; }
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}