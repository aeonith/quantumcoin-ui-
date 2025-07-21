use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

const REVSTOP_FILE: &str = "revstop.lock";

pub struct RevStop {
    active: bool,
    password: Option<String>,
}

impl RevStop {
    pub fn new() -> Self {
        if Path::new(REVSTOP_FILE).exists() {
            let mut file = File::open(REVSTOP_FILE).expect("Unable to open revstop file");
            let mut content = String::new();
            file.read_to_string(&mut content).expect("Failed to read revstop file");
            if content.starts_with("locked:") {
                let password = content["locked:".len()..].trim().to_string();
                return RevStop {
                    active: true,
                    password: Some(password),
                };
            }
        }
        RevStop {
            active: false,
            password: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn lock(&mut self, password: &str) {
        let mut file = File::create(REVSTOP_FILE).expect("Unable to create revstop file");
        file.write_all(format!("locked:{}", password).as_bytes())
            .expect("Failed to write revstop status");
        self.active = true;
        self.password = Some(password.to_string());
    }

    pub fn unlock(&mut self, input_password: &str) -> bool {
        if let Some(ref actual_password) = self.password {
            if input_password == actual_password {
                std::fs::remove_file(REVSTOP_FILE).expect("Failed to remove revstop file");
                self.active = false;
                self.password = None;
                return true;
            }
        }
        false
    }

    pub fn get_status_message(&self) -> String {
        if self.active {
            "🔒 RevStop is ACTIVE".to_string()
        } else {
            "🔓 RevStop is INACTIVE".to_string()
        }
    }
}