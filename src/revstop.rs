use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;

lazy_static! {
    static ref LOCKED: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

/// Is RevStop (lock) active for this address?
pub fn is_revstop_active(addr: &str) -> bool {
    LOCKED.lock().unwrap().contains(addr)
}

/// Human-readable RevStop status
pub fn get_revstop_status(addr: &str) -> String {
    if is_revstop_active(addr) {
        "🔒 ACTIVE".into()
    } else {
        "🔓 INACTIVE".into()
    }
}

/// Lock address (disable spending)
pub fn lock_address(addr: &str) {
    LOCKED.lock().unwrap().insert(addr.to_string());
}

/// Unlock address
pub fn unlock_address(addr: &str) {
    LOCKED.lock().unwrap().remove(addr);
}