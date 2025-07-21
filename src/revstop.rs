use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RevStop {
    pub enabled: bool,
}

impl RevStop {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn load_status() -> Option<Self> {
        fs::read("revstop_status.json").ok()
            .and_then(|b| serde_json::from_slice::<Self>(&b).ok())
    }

    pub fn save_status(&self) -> Result<(), Box<dyn Error>> {
        fs::write("revstop_status.json", serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

pub fn is_revstop_active(rev: &RevStop) -> bool {
    rev.enabled
}