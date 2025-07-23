use std::fs::File;
use std::io::{Read, Write};

pub struct RevStop;

impl RevStop {
    pub fn lock(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(b"locked").unwrap();
    }

    pub fn unlock(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(b"unlocked").unwrap();
    }

    pub fn is_locked(&self, path: &str) -> bool {
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            contents.trim() == "locked"
        } else {
            false
        }
    }

    pub fn status() -> String {
        "RevStop active".to_string()
    }

    pub fn load_status(path: &str) -> Self {
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() && contents.trim() == "locked" {
                return RevStop::locked();
            }
        }
        RevStop::unlocked()
    }

    pub fn locked() -> Self {
        RevStop
    }

    pub fn unlocked() -> Self {
        RevStop
    }
}