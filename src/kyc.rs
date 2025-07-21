use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::revstop::is_revstop_active;
use crate::wallet::Wallet;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum KycStatus {
    Verified,
    Denied,
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct KycMetadata {
    pub wallet_address: String,
    pub status: KycStatus,
    pub created_at: u64,
}

const KYC_FILE: &str = "kyc_registry.json";

pub fn load_kyc_registry() -> HashMap<String, KycMetadata> {
    if Path::new(KYC_FILE).exists() {
        let mut file = File::open(KYC_FILE).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn save_kyc_registry(registry: &HashMap<String, KycMetadata>) {
    let json = serde_json::to_string_pretty(&registry).unwrap();
    let mut file = File::create(KYC_FILE).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

// 🔒 Logic-based autonomous approval
pub fn evaluate_kyc(wallet: &Wallet) -> KycStatus {
    let revstop_status = is_revstop_active();
    let wallet_age = wallet.creation_time; // Assume this is stored in Wallet

    // Require RevStop lock and wallet to be older than 30 days
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let min_age = 60 * 60 * 24 * 30;

    if revstop_status && now > wallet_age + min_age {
        KycStatus::Verified
    } else {
        KycStatus::Denied
    }
}

pub fn get_kyc_status(wallet: &Wallet) -> KycStatus {
    let registry = load_kyc_registry();
    if let Some(entry) = registry.get(&wallet.get_address()) {
        entry.status.clone()
    } else {
        let evaluated = evaluate_kyc(wallet);
        let new_entry = KycMetadata {
            wallet_address: wallet.get_address(),
            status: evaluated.clone(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        let mut updated_registry = registry.clone();
        updated_registry.insert(wallet.get_address(), new_entry);
        save_kyc_registry(&updated_registry);
        evaluated
    }
}