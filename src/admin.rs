use std::fs;
use sha2::{Sha256, Digest};
use std::sync::Mutex;
use std::path::Path;

const ADMIN_HASH_PATH: &str = "admin_hash.txt"; // stores hashed admin password

lazy_static::lazy_static! {
    static ref ADMIN_LOCKED: Mutex<bool> = Mutex::new(true); // initially locked
}

// Initialize admin password hash (only run once or securely elsewhere)
pub fn set_admin_password(password: &str) {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    fs::write(ADMIN_HASH_PATH, hex::encode(result)).expect("Failed to write admin password hash.");
}

// Check admin login
pub fn verify_admin(password: &str) -> bool {
    if !Path::new(ADMIN_HASH_PATH).exists() {
        return false;
    }

    let saved_hash = fs::read_to_string(ADMIN_HASH_PATH).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hex::encode(hasher.finalize());

    let is_valid = result == saved_hash;

    let mut locked = ADMIN_LOCKED.lock().unwrap();
    *locked = !is_valid;

    is_valid
}

// Check if admin is unlocked
pub fn is_admin_unlocked() -> bool {
    let locked = ADMIN_LOCKED.lock().unwrap();
    !*locked
}

// Lock admin
pub fn lock_admin() {
    let mut locked = ADMIN_LOCKED.lock().unwrap();
    *locked = true;
}

// Admin-only actions
pub fn admin_override_kyc() -> Result<&'static str, &'static str> {
    if is_admin_unlocked() {
        Ok("✅ Admin override of KYC successful.")
    } else {
        Err("⛔ Admin access is locked.")
    }
}

pub fn admin_pause_mining() -> Result<&'static str, &'static str> {
    if is_admin_unlocked() {
        Ok("🛑 Mining has been paused by admin.")
    } else {
        Err("⛔ Admin access is locked.")
    }
}