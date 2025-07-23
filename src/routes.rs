use actix_web::{post, get, web, HttpResponse, Responder};
use std::sync::Mutex;
use std::fs;
use uuid::Uuid;
use std::io::Write;

use crate::wallet::{Wallet};
use crate::blockchain;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WalletRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct WalletResponse {
    pub address: String,
    pub balance: u64,
}

#[post("/create_wallet")]
async fn create_wallet(data: web::Json<WalletRequest>) -> impl Responder {
    // Load or create a wallet
    let wallet = Wallet::load_or_generate();
    wallet.save_to_file();

    HttpResponse::Ok().json(WalletResponse {
        address: wallet.get_address(),
        balance: wallet.get_balance(),
    })
}

#[get("/wallet_balance")]
async fn wallet_balance() -> impl Responder {
    if let Some(wallet) = Wallet::load_from_file() {
        HttpResponse::Ok().json(WalletResponse {
            address: wallet.get_address(),
            balance: wallet.get_balance(),
        })
    } else {
        HttpResponse::NotFound().body("Wallet not found")
    }
}

#[get("/transactions")]
async fn last_transactions() -> impl Responder {
    if let Some(wallet) = Wallet::load_from_file() {
        let txs = wallet.recent_tx.clone();
        HttpResponse::Ok().json(txs)
    } else {
        HttpResponse::NotFound().body("Wallet not found")
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(create_wallet);
    cfg.service(wallet_balance);
    cfg.service(last_transactions);
}