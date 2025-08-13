use std::{fs, io::Write, path::Path};
use serde::{Serialize, de::DeserializeOwned};
use crate::utils::hash::sha256_hex;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Wrapped<T> {
    pub payload: T,
    pub checksum: String,
}

pub fn atomic_write_json<T: Serialize>(p: &Path, value: &T) -> std::io::Result<()> {
    let json = serde_json::to_vec(value).expect("serialize");
    let checksum = sha256_hex(&json);
    let wrapped = Wrapped { payload: value, checksum };
    let bytes = serde_json::to_vec_pretty(&wrapped).expect("wrap");
    let tmp = p.with_extension("tmp");
    let bak = p.with_extension("bak");

    if p.exists() {
        if let Err(_) = fs::copy(p, &bak) {
            // ignore backup failure
        }
    }
    let mut f = fs::File::create(&tmp)?;
    f.write_all(&bytes)?;
    f.sync_all()?;
    fs::rename(&tmp, p)?;
    Ok(())
}

pub fn read_json_verified<T: DeserializeOwned>(p: &Path) -> std::io::Result<T> {
    let bytes = fs::read(p)?;
    let wrapped: Wrapped<serde_json::Value> = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let payload_bytes = serde_json::to_vec(&wrapped.payload).unwrap();
    let sum = sha256_hex(&payload_bytes);
    if sum != wrapped.checksum {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "checksum mismatch"));
    }
    let t: T = serde_json::from_slice(&payload_bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(t)
}
