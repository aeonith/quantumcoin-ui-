use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;

#[derive(Deserialize)]
pub struct TransactionRequest {
    pub recipient: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct WalletResponse {
    pub publicKey: String,
}

#[get("/wallet")]
async fn get_wallet(wallet: web::Data<Mutex<Wallet>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    HttpResponse::Ok().json(WalletResponse {
        publicKey: wallet.get_address_base64(),
    })
}

#[post("/transaction")]
async fn create_transaction(
    req: web::Json<TransactionRequest>,
    wallet: web::Data<Mutex<Wallet>>,
    blockchain: web::Data<Mutex<Blockchain>>,
) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let tx = wallet.create_transaction(&req.recipient, req.amount, &wallet.get_address_base64());

    let mut blockchain = blockchain.lock().unwrap();
    blockchain.add_transaction(tx);

    HttpResponse::Ok().body("Transaction added to mempool")
}

#[post("/mine")]
async fn mine(blockchain: web::Data<Mutex<Blockchain>>, wallet: web::Data<Mutex<Wallet>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let mut blockchain = blockchain.lock().unwrap();

    let miner_address = wallet.get_address_base64();
    blockchain.mine_pending_transactions(&miner_address);

    HttpResponse::Ok().body("Block mined successfully")
}

#[get("/balance")]
async fn get_balance(wallet: web::Data<Mutex<Wallet>>, blockchain: web::Data<Mutex<Blockchain>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let blockchain = blockchain.lock().unwrap();

    let balance = blockchain.get_balance(&wallet.get_address_base64());
    HttpResponse::Ok().body(format!("Balance: {} QTC", balance))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_wallet);
    cfg.service(create_transaction);
    cfg.service(mine);
    cfg.service(get_balance);
}