use serde::{Deserialize, Serialize};
use std::{fs, error::Error};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RevStop {
    pub enabled: bool,
}

impl RevStop {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn default_and_save(path: &str) -> Result<Self, Box<dyn Error>> {
        let r = RevStop { enabled: false };
        fs::write(path, serde_json::to_string_pretty(&r)?)?;
        Ok(r)
    }

    pub fn save_status(&self, path: &str) -> Result<(), Box<dyn Error>> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn is_active(&self) -> bool { self.enabled }
}