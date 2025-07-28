use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::blockchain::Blockchain;
use crate::wallet::Wallet;

#[derive(Deserialize)]
pub struct SendRequest {
    pub recipient: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub balance: u64,
}

#[derive(Serialize)]
pub struct WalletResponse {
    pub publicKey: String,
    pub privateKey: String,
}

#[post("/send")]
async fn send(
    data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> impl Responder {
    let mut blockchain = data.lock().unwrap();
    let wallet = wallet_data.lock().unwrap();

    let sender_address = wallet.get_address_base64();
    let tx = wallet.create_transaction(&req.recipient, req.amount, &sender_address);
    blockchain.add_transaction(tx);

    HttpResponse::Ok().body("✅ Transaction added")
}

#[post("/mine")]
async fn mine(
    data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let mut blockchain = data.lock().unwrap();
    let wallet = wallet_data.lock().unwrap();

    let miner_address = wallet.get_address_base64();
    blockchain.mine_pending_transactions(&miner_address);

    HttpResponse::Ok().body("⛏️ Block mined successfully")
}

#[get("/balance")]
async fn balance(
    data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let blockchain = data.lock().unwrap();
    let wallet = wallet_data.lock().unwrap();

    let balance = blockchain.get_balance(&wallet.get_address_base64());
    HttpResponse::Ok().json(BalanceResponse { balance })
}

#[get("/wallet")]
async fn wallet(wallet_data: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    let public_key = wallet.get_public_key_base64();
    let private_key = wallet.get_private_key_base64();

    HttpResponse::Ok().json(WalletResponse {
        publicKey: public_key,
        privateKey: private_key,
    })
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(send);
    cfg.service(mine);
    cfg.service(balance);
    cfg.service(wallet);
}