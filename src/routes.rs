use actix_web::{get, post, web, HttpResponse, Responder};
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::blockchain::Blockchain;
use crate::transaction::Transaction;
use crate::wallet::Wallet;

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: u64,
    pub signature: String,
}

#[get("/balance")]
async fn balance(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let chain = blockchain.lock().unwrap();
    let wallet = wallet.lock().unwrap();
    let balance = chain.get_balance(&wallet.public_key);
    HttpResponse::Ok().json(balance)
}

#[get("/address")]
async fn address(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    HttpResponse::Ok().json(wallet.public_key.clone())
}

#[post("/send")]
async fn send(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let chain = &mut blockchain.lock().unwrap();

    let signature_bytes = general_purpose::STANDARD.decode(&req.signature).unwrap();
    if !wallet.verify_signature(req.to.as_bytes(), &signature_bytes) {
        return HttpResponse::BadRequest().body("Invalid signature");
    }

    let tx = Transaction::new(
        wallet.public_key.clone(),
        req.to.clone(),
        req.amount,
        Some(req.signature.clone()),
    );

    chain.add_transaction(tx);
    chain.mine_pending_transactions(&wallet.public_key);

    HttpResponse::Ok().body("Transaction sent and block mined")
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(balance)
        .service(address)
        .service(send);
}