use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use crate::revstop::{is_revstop_active, get_revstop_status};
use std::sync::{Arc, Mutex};
use base64::decode;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/send").route(web::post().to(send)));
    cfg.service(web::resource("/mine").route(web::post().to(mine)));
    cfg.service(web::resource("/balance").route(web::get().to(balance)));
    cfg.service(web::resource("/revstop-status").route(web::get().to(rev_status)));
}

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: u64,
    pub signature: String,
}

async fn send(
    req: web::Json<SendRequest>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> HttpResponse {
    let wallet = wallet.lock().unwrap();
    let signature_bytes = decode(&req.signature).unwrap();
    let signature = pqcrypto_dilithium::dilithium2::DetachedSignature::from_bytes(&signature_bytes).unwrap();

    let tx = Transaction {
        sender: wallet.get_address(),
        recipient: req.to.clone(),
        amount: req.amount,
        signature: req.signature.clone(),
    };

    let mut chain = blockchain.lock().unwrap();
    let success = chain.add_transaction(tx);

    if success {
        HttpResponse::Ok().body("✅ Transaction added")
    } else {
        HttpResponse::BadRequest().body("❌ RevStop active or invalid tx")
    }
}

async fn mine(blockchain: web::Data<Arc<Mutex<Blockchain>>>, wallet: web::Data<Arc<Mutex<Wallet>>>) -> HttpResponse {
    let wallet = wallet.lock().unwrap();
    let mut chain = blockchain.lock().unwrap();
    chain.mine_block(&wallet.get_address());
    HttpResponse::Ok().body("⛏️ Mined 1 block")
}

async fn balance(blockchain: web::Data<Arc<Mutex<Blockchain>>>, wallet: web::Data<Arc<Mutex<Wallet>>>) -> HttpResponse {
    let wallet = wallet.lock().unwrap();
    let chain = blockchain.lock().unwrap();
    let balance = chain.get_balance(&wallet.get_address());
    HttpResponse::Ok().body(format!("💰 Balance: {} QTC", balance))
}

async fn rev_status(wallet: web::Data<Arc<Mutex<Wallet>>>) -> HttpResponse {
    let wallet = wallet.lock().unwrap();
    let status = get_revstop_status(&wallet.get_address());
    HttpResponse::Ok().body(status)
}