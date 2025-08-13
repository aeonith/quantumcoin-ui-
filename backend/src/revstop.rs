use std::sync::atomic::{AtomicBool, Ordering};

static REVSTOP_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn activate() {
    REVSTOP_ACTIVE.store(true, Ordering::SeqCst);
    println!("RevStop activated - quantum-safe emergency protocol enabled");
}

pub fn deactivate() {
    REVSTOP_ACTIVE.store(false, Ordering::SeqCst);
    println!("RevStop deactivated - normal operations resumed");
}

pub fn is_revstop_active() -> bool {
    REVSTOP_ACTIVE.load(Ordering::SeqCst)
}

pub fn emergency_halt() -> bool {
    if is_revstop_active() {
        println!("Emergency halt triggered due to RevStop protocol");
        true
    } else {
        false
    }
}
