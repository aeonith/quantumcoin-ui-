use std::fs;
use std::collections::HashMap;

const REVSTOP_FILE: &str = "revstop_status.json";

pub fn is_revstop_active(public_key: &str) -> bool {
    let data = fs::read_to_string(REVSTOP_FILE).unwrap_or_else(|_| "{}".to_string());
    let map: HashMap<String, bool> = serde_json::from_str(&data).unwrap_or_default();
    map.get(public_key).copied().unwrap_or(false)
}

pub fn get_revstop_status(public_key: &str) -> String {
    if is_revstop_active(public_key) {
        "🔒 RevStop is ACTIVE".to_string()
    } else {
        "🔓 RevStop is NOT active".to_string()
    }
}