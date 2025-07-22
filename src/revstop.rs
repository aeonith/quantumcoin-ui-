use std::fs::{File, OpenOptions, remove_file};
use std::io::{Write, Read};
use std::path::Path;

const REVSTOP_FILE: &str = "revstop.lock";

pub fn enable_revstop(password: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(REVSTOP_FILE)?;
    file.write_all(password.as_bytes())?;
    Ok(())
}

pub fn disable_revstop(password: &str) -> std::io::Result<bool> {
    if !Path::new(REVSTOP_FILE).exists() {
        return Ok(false);
    }
    let mut file = File::open(REVSTOP_FILE)?;
    let mut stored_password = String::new();
    file.read_to_string(&mut stored_password)?;
    if stored_password == password {
        remove_file(REVSTOP_FILE)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn is_revstop_enabled() -> bool {
    Path::new(REVSTOP_FILE).exists()
}