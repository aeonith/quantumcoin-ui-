use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevStop {
    enabled: bool,
    activation_time: Option<u64>,
    reason: Option<String>,
    automatic_disable_time: Option<u64>,
}

impl RevStop {
    pub fn new() -> Self {
        Self {
            enabled: false,
            activation_time: None,
            reason: None,
            automatic_disable_time: None,
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        // Check if automatic disable time has passed
        if let Some(disable_time) = self.automatic_disable_time {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now >= disable_time {
                return false;
            }
        }
        
        self.enabled
    }
    
    pub fn activate(&mut self, reason: Option<String>, duration_seconds: Option<u64>) {
        self.enabled = true;
        self.activation_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        self.reason = reason.clone();
        
        if let Some(duration) = duration_seconds {
            self.automatic_disable_time = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + duration
            );
        }
        
        warn!(
            "🚫 RevStop activated. Reason: {}",
            reason.unwrap_or_else(|| "Manual activation".to_string())
        );
    }
    
    pub fn deactivate(&mut self) {
        self.enabled = false;
        self.activation_time = None;
        self.reason = None;
        self.automatic_disable_time = None;
        
        info!("✅ RevStop deactivated. Operations resumed.");
    }
    
    pub fn get_status(&self) -> RevStopStatus {
        RevStopStatus {
            enabled: self.is_enabled(),
            activation_time: self.activation_time,
            reason: self.reason.clone(),
            time_remaining: if let Some(disable_time) = self.automatic_disable_time {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if disable_time > now {
                    Some(disable_time - now)
                } else {
                    None
                }
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevStopStatus {
    pub enabled: bool,
    pub activation_time: Option<u64>,
    pub reason: Option<String>,
    pub time_remaining: Option<u64>,
}

impl Default for RevStop {
    fn default() -> Self {
        Self::new()
    }
}
