use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::revstop::{is_revstop_active, get_revstop_status};

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub balance: u64,
}

#[derive(Serialize)]
pub struct AddressResponse {
    pub address: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
}

#[get("/balance")]
async fn balance(wallet: web::Data<Arc<Mutex<Wallet>>>, blockchain: web::Data<Arc<Mutex<Blockchain>>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let blockchain = blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&wallet.public_key);
    HttpResponse::Ok().json(BalanceResponse { balance })
}

#[get("/address")]
async fn address(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    HttpResponse::Ok().json(AddressResponse {
        address: wallet.public_key.clone(),
    })
}

#[post("/send")]
async fn send(
    req: web::Json<SendRequest>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let mut chain = blockchain.lock().unwrap();

    let message = format!("{}{}{}", &wallet.public_key, req.to, req.amount);
    let signature = wallet.sign_message(message.as_bytes());

    let success = chain.add_transaction(&wallet.public_key, &req.to, req.amount, &signature);

    if success {
        HttpResponse::Ok().body("✅ Transaction added to mempool.")
    } else {
        HttpResponse::BadRequest().body("❌ Transaction failed.")
    }
}

#[post("/mine")]
async fn mine(
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let mut chain = blockchain.lock().unwrap();

    chain.mine_pending_transactions(&wallet.public_key);
    HttpResponse::Ok().body("⛏️ Mining complete.")
}

#[get("/revstop")]
async fn revstop_status(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let status = get_revstop_status(&wallet.public_key);

    let message = if is_revstop_active(&wallet.public_key) {
        format!("🔒 RevStop ACTIVE: {}", status)
    } else {
        format!("🔓 RevStop INACTIVE: {}", status)
    };

    HttpResponse::Ok().json(StatusResponse { status: message })
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(balance);
    cfg.service(address);
    cfg.service(send);
    cfg.service(mine);
    cfg.service(revstop_status);
}