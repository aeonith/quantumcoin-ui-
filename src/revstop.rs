use std::collections::HashSet;
use std::sync::Mutex;
use std::fs::{read_to_string, write};

lazy_static::lazy_static! {
    static ref REVSTOP_DB: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

pub fn is_revstop_active(address: &str) -> bool {
    REVSTOP_DB.lock().unwrap().contains(address)
}

pub fn enable_revstop(address: &str) {
    REVSTOP_DB.lock().unwrap().insert(address.to_string());
}

pub fn disable_revstop(address: &str) {
    REVSTOP_DB.lock().unwrap().remove(address);
}

pub fn get_revstop_status(address: &str) -> String {
    if is_revstop_active(address) {
        "🔒 RevStop is ACTIVE".to_string()
    } else {
        "🔓 RevStop is INACTIVE".to_string()
    }
}