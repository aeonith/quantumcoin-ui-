use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::wallet::Wallet;
use crate::blockchain::{Blockchain, Transaction};
use crate::revstop::{is_revstop_active, get_revstop_status};

#[derive(Serialize)]
pub struct BalanceResponse {
    pub balance: u64,
}

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: u64,
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct AddressResponse {
    pub address: String,
}

#[get("/balance")]
pub fn balance() -> Json<BalanceResponse> {
    let wallet = Wallet::load_from_files();
    Json(BalanceResponse {
        balance: wallet.get_balance(),
    })
}

#[get("/address")]
pub fn address() -> Json<AddressResponse> {
    let wallet = Wallet::load_from_files();
    Json(AddressResponse {
        address: wallet.public_key,
    })
}

#[post("/send", format = "application/json", data = "<req>")]
pub fn send(req: Json<SendRequest>, blockchain: &rocket::State<Arc<Mutex<Blockchain>>>) -> String {
    let wallet = Wallet::load_from_files();

    if is_revstop_active(&wallet.public_key) {
        return "❌ RevStop protection is active. Transaction blocked.".to_string();
    }

    let tx = Transaction {
        sender: wallet.public_key.clone(),
        recipient: req.to.clone(),
        amount: req.amount,
        signature: None, // Optionally sign this
    };

    let mut chain = blockchain.lock().unwrap();
    if chain.add_transaction(tx) {
        chain.save_to_disk();
        "✅ Transaction added.".to_string()
    } else {
        "❌ Transaction failed.".to_string()
    }
}

#[get("/mine")]
pub fn mine(blockchain: &rocket::State<Arc<Mutex<Blockchain>>>) -> String {
    let wallet = Wallet::load_from_files();
    let mut chain = blockchain.lock().unwrap();
    chain.mine_pending_transactions(&wallet.public_key);
    chain.save_to_disk();
    "⛏️ Mining complete.".to_string()
}

#[get("/revstop")]
pub fn revstop_status() -> String {
    let wallet = Wallet::load_from_files();
    if get_revstop_status(&wallet.public_key) {
        "🔒 RevStop is ACTIVE".to_string()
    } else {
        "🔓 RevStop is NOT active".to_string()
    }
}

#[post("/export", format = "application/json", data = "<req>")]
pub fn export(req: Json<ExportRequest>) -> String {
    let wallet = Wallet::load_from_files();
    if let Some(encoded) = wallet.export_with_2fa(&req.code) {
        format!("✅ Exported: {}", encoded)
    } else {
        "❌ Export failed: invalid code.".to_string()
    }
}