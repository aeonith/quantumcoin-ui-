use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::blockchain::Blockchain;
use crate::wallet::Wallet;

#[derive(Deserialize)]
pub struct SendRequest {
    recipient: String,
    amount: u64,
}

#[derive(Serialize)]
pub struct WalletInfo {
    publicKey: String,
    privateKey: String,
}

#[get("/wallet")]
async fn get_wallet(data: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = data.lock().unwrap();
    HttpResponse::Ok().json(WalletInfo {
        publicKey: wallet.get_public_key_base64(),   // ✅ updated
        privateKey: wallet.get_private_key_base64(), // ✅ updated
    })
}

#[post("/send")]
async fn send_transaction(
    blockchain_data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    let mut blockchain = blockchain_data.lock().unwrap();

    let tx = wallet.create_transaction(&req.recipient, req.amount);
    blockchain.add_transaction(tx);
    HttpResponse::Ok().body("Transaction created")
}

#[post("/mine")]
async fn mine_block(blockchain_data: web::Data<Arc<Mutex<Blockchain>>>) -> impl Responder {
    let mut blockchain = blockchain_data.lock().unwrap();
    blockchain.mine_pending_transactions();
    HttpResponse::Ok().body("Block mined")
}

#[get("/balance")]
async fn get_balance(
    blockchain_data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    let blockchain = blockchain_data.lock().unwrap();
    let balance = blockchain.get_balance(&wallet.get_address());
    HttpResponse::Ok().body(format!("Balance: {} QTC", balance))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_wallet);
    cfg.service(send_transaction);
    cfg.service(mine_block);
    cfg.service(get_balance);
}