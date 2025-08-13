use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevMode { Usb, Password }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevStopStatus {
    pub active: bool,
    pub mode: Option<RevMode>,
    pub last_changed: DateTime<Utc>,
}

impl Default for RevStopStatus {
    fn default() -> Self {
        Self { active: false, mode: None, last_changed: Utc::now() }
    }
}

pub struct RevStop {
    status_path: PathBuf,
    status: RevStopStatus,
}

impl RevStop {
    pub fn load(status_path: PathBuf) -> Self {
        use std::path::Path;
        let status = if Path::new(&status_path).exists() {
            crate::utils::fs::read_json_verified::<RevStopStatus>(&status_path).unwrap_or_default()
        } else { RevStopStatus::default() };
        Self { status_path, status }
    }
    pub fn is_active(&self) -> bool { self.status.active }
    pub fn mode(&self) -> Option<RevMode> { self.status.mode.clone() }
    pub fn lock_password(&mut self) {
        self.status.active = true; self.status.mode = Some(RevMode::Password);
        self.status.last_changed = Utc::now();
        let _ = crate::utils::fs::atomic_write_json(&self.status_path, &self.status);
    }
    pub fn lock_usb(&mut self) {
        self.status.active = true; self.status.mode = Some(RevMode::Usb);
        self.status.last_changed = Utc::now();
        let _ = crate::utils::fs::atomic_write_json(&self.status_path, &self.status);
    }
    pub fn unlock(&mut self) {
        self.status.active = false; self.status.mode = None;
        self.status.last_changed = Utc::now();
        let _ = crate::utils::fs::atomic_write_json(&self.status_path, &self.status);
    }
}
