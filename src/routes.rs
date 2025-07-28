use actix_web::{get, post, web, HttpResponse, Responder};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SendRequest {
    pub recipient: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct WalletResponse {
    pub public_key: String,
    pub private_key: String,
    pub address: String,
}

#[get("/wallet")]
pub async fn get_wallet(wallet_data: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    HttpResponse::Ok().json(WalletResponse {
        public_key: wallet.get_public_key_base64(),
        private_key: wallet.get_private_key_base64(),
        address: wallet.get_address_base64(),
    })
}

#[post("/send")]
pub async fn send_transaction(
    blockchain_data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    let sender_address = wallet.get_address_base64();

    let tx = wallet.create_transaction(&req.recipient, req.amount, &sender_address);
    let mut blockchain = blockchain_data.lock().unwrap();
    blockchain.add_transaction(tx);

    HttpResponse::Ok().body("Transaction submitted")
}

#[get("/mine")]
pub async fn mine_block(
    blockchain_data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    let miner_address = wallet.get_address_base64();
    let mut blockchain = blockchain_data.lock().unwrap();
    blockchain.mine_pending_transactions(&miner_address);
    HttpResponse::Ok().body("Block mined")
}

#[get("/balance")]
pub async fn get_balance(
    blockchain_data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet_data: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let wallet = wallet_data.lock().unwrap();
    let address = wallet.get_address_base64();
    let blockchain = blockchain_data.lock().unwrap();
    let balance = blockchain.get_balance(&address);
    HttpResponse::Ok().body(format!("Your balance is: {} QTC", balance))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_wallet);
    cfg.service(send_transaction);
    cfg.service(mine_block);
    cfg.service(get_balance);
}