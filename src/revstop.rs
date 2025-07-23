use std::fs::{self, File};
use std::io::{Read, Write};

pub struct RevStop;

impl RevStop {
    pub fn lock(password: &str) {
        fs::write("revstop.lock", password).unwrap();
    }

    pub fn unlock(input_password: &str) -> bool {
        if let Ok(mut file) = File::open("revstop.lock") {
            let mut stored = String::new();
            file.read_to_string(&mut stored).unwrap();
            stored.trim() == input_password.trim()
        } else {
            false
        }
    }

    pub fn is_active() -> bool {
        File::open("revstop.lock").is_ok()
    }

    pub fn status() -> String {
        if Self::is_active() {
            "🔒 RevStop Protection Active".to_string()
        } else {
            "🔓 RevStop Protection Disabled".to_string()
        }
    }

    pub fn clear() {
        let _ = fs::remove_file("revstop.lock");
    }
}