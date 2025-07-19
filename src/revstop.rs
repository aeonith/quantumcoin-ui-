use std::fs::{File};
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

const REVSTOP_FILE: &str = "revstop_status.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct RevStop {
    pub is_active: bool,
    pub password_hash: String,
}

impl RevStop {
    pub fn new(password: &str) -> Self {
        Self {
            is_active: true,
            password_hash: hash_password(password),
        }
    }

    pub fn load() -> Option<Self> {
        if Path::new(REVSTOP_FILE).exists() {
            let mut file = File::open(REVSTOP_FILE).ok()?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).ok()?;
            serde_json::from_str(&contents).ok()
        } else {
            None
        }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = File::create(REVSTOP_FILE).and_then(|mut f| f.write_all(json.as_bytes()));
        }
    }

    pub fn lock(&mut self) {
        self.is_active = true;
        self.save();
    }

    pub fn unlock(&mut self, password: &str) -> bool {
        if verify_password(&self.password_hash, password) {
            self.is_active = false;
            self.save();
            true
        } else {
            false
        }
    }

    pub fn status(&self) -> bool {
        self.is_active
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verify_password(hash: &str, password: &str) -> bool {
    hash == hash_password(password)
}