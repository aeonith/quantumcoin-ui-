use actix_web::{web, HttpResponse};
use base64::Engine;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use crate::{
    blockchain::Blockchain,
    transaction::Transaction,
    wallet::Wallet,
    revstop::{is_revstop_active, get_revstop_status},
};

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: u64,
    pub signature: String,
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/balance", web::get().to(balance))
       .route("/address", web::get().to(address))
       .route("/send", web::post().to(send))
       .route("/mine", web::post().to(mine))
       .route("/revstop-status", web::get().to(revstop_status));
}

async fn balance(
    bc: web::Data<Arc<Mutex<Blockchain>>>,
    w: web::Data<Arc<Mutex<Wallet>>>,
) -> HttpResponse {
    let chain = bc.lock().unwrap();
    let wallet = w.lock().unwrap();
    let bal = chain.get_balance(&wallet.get_address());
    HttpResponse::Ok().json(bal)
}

async fn address(w: web::Data<Arc<Mutex<Wallet>>>) -> HttpResponse {
    let wallet = w.lock().unwrap();
    HttpResponse::Ok().json(wallet.get_address())
}

async fn send(
    bc: web::Data<Arc<Mutex<Blockchain>>>,
    w: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> HttpResponse {
    let wallet = w.lock().unwrap();
    let data = format!("{}{}{}", wallet.get_address(), &req.to, req.amount);
    let sig_bytes = general_purpose::STANDARD.decode(&req.signature).unwrap();
    if !wallet.verify_signature(data.as_bytes(), &sig_bytes) {
        return HttpResponse::BadRequest().body("Invalid signature");
    }

    let tx = Transaction::new(
        wallet.get_address(),
        req.to.clone(),
        req.amount,
        Some(req.signature.clone()),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    );
    let mut chain = bc.lock().unwrap();
    chain.add_transaction(tx);
    HttpResponse::Ok().body("✅ Transaction added")
}

async fn mine(
    bc: web::Data<Arc<Mutex<Blockchain>>>,
    w: web::Data<Arc<Mutex<Wallet>>>,
) -> HttpResponse {
    let wallet = w.lock().unwrap();
    let mut chain = bc.lock().unwrap();
    chain.mine_pending_transactions(&wallet.get_address());
    HttpResponse::Ok().body("⛏️ Block mined")
}

async fn revstop_status(w: web::Data<Arc<Mutex<Wallet>>>) -> HttpResponse {
    let wallet = w.lock().unwrap();
    let status = get_revstop_status(&wallet.get_address());
    HttpResponse::Ok().body(status)
}