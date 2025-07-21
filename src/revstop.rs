use serde::{Deserialize, Serialize};
use std::{fs, error::Error};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RevStop {
    pub enabled: bool,
}

impl RevStop {
    pub fn load_status(path: &str) -> Result<Self, Box<dyn Error>> {
        if let Ok(data) = fs::read_to_string(path) {
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save_status(&self, path: &str) -> Result<(), Box<dyn Error>> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}