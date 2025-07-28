use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::transaction::Transaction;

#[derive(Deserialize)]
pub struct SendRequest {
    pub recipient: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct WalletInfo {
    pub address: String,
    pub balance: f64,
}

#[get("/wallet")]
async fn wallet_info(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    let blockchain = blockchain.lock().unwrap();
    let address = wallet.get_address_base64();
    let balance = blockchain.get_balance(&address);

    HttpResponse::Ok().json(WalletInfo { address, balance })
}

#[post("/send")]
async fn send_coins(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> impl Responder {
    let mut blockchain = blockchain.lock().unwrap();
    let wallet = wallet_data.lock().unwrap();
    let sender_address = wallet.get_address_base64();

    let tx = wallet.create_transaction(&req.recipient, req.amount, &sender_address);
    blockchain.add_transaction(tx);

    HttpResponse::Ok().body("Transaction submitted")
}

#[post("/mine")]
async fn mine_block(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let mut blockchain = blockchain.lock().unwrap();
    let wallet = wallet_data.lock().unwrap();
    let miner_address = wallet.get_address_base64();

    blockchain.mine_pending_transactions(&miner_address);

    HttpResponse::Ok().body("Block mined")
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(wallet_info);
    cfg.service(send_coins);
    cfg.service(mine_block);
}