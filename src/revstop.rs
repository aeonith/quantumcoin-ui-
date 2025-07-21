// src/revstop.rs

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const REVSTOP_FILE: &str = "revstop.lock";

pub struct RevStop {
    active: bool,
    password: String,
}

impl RevStop {
    // Initialize RevStop system (load or default)
    pub fn load() -> Self {
        if Path::new(REVSTOP_FILE).exists() {
            let mut file = File::open(REVSTOP_FILE).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let parts: Vec<&str> = contents.splitn(2, ':').collect();
            let status = parts[0] == "1";
            let password = parts.get(1).unwrap_or(&"default_password").to_string();
            RevStop {
                active: status,
                password,
            }
        } else {
            RevStop {
                active: false,
                password: "default_password".to_string(),
            }
        }
    }

    // Save current RevStop state to disk
    pub fn save(&self) {
        let status = if self.active { "1" } else { "0" };
        let data = format!("{}:{}", status, self.password);
        let mut file = File::create(REVSTOP_FILE).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }

    // Lock the RevStop system with password
    pub fn lock(&mut self, password: String) {
        self.active = true;
        self.password = password;
        self.save();
        println!("🔒 RevStop is now ENABLED.");
    }

    // Attempt to unlock using password
    pub fn unlock(&mut self, input_password: &str) -> bool {
        if input_password == self.password {
            self.active = false;
            self.save();
            println!("🔓 RevStop is now DISABLED.");
            true
        } else {
            println!("❌ Incorrect password. RevStop remains active.");
            false
        }
    }

    // Returns status of RevStop
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn status_message(&self) -> String {
        if self.active {
            "🛡️ RevStop: ACTIVE (Transaction protections enabled)".to_string()
        } else {
            "🟢 RevStop: DISABLED (Transactions unrestricted)".to_string()
        }
    }
}