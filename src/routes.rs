use actix_web::{web, HttpResponse};
use base64::engine::general_purpose;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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
       .route("/revstop", web::get().to(revstop_status));
}

async fn balance(
    bc: web::Data<Arc<Mutex<Blockchain>>>,
    w: web::Data<Arc<Mutex<Wallet>>>,
) -> HttpResponse {
    let chain = bc.lock().unwrap();
    let wallet = w.lock().unwrap();
    HttpResponse::Ok().json(chain.get_balance(&wallet.address()))
}

async fn address(w: web::Data<Arc<Mutex<Wallet>>>) -> HttpResponse {
    let wallet = w.lock().unwrap();
    HttpResponse::Ok().json(wallet.address())
}

async fn send(
    bc: web::Data<Arc<Mutex<Blockchain>>>,
    w: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> HttpResponse {
    let wallet = w.lock().unwrap();
    let data = format!("{}{}{}", wallet.address(), &req.to, req.amount);
    let sig = general_purpose::STANDARD.decode(&req.signature).unwrap();
    if !wallet.verify(data.as_bytes(), &sig) {
        return HttpResponse::BadRequest().body("Invalid signature");
    }

    let tx = Transaction::new(
        wallet.address(),
        req.to.clone(),
        req.amount,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        Some(req.signature.clone()),
    );
    let mut chain = bc.lock().unwrap();
    if is_revstop_active(&wallet.address()) {
        return HttpResponse::Forbidden().body("RevStop active");
    }
    chain.add_transaction(tx);
    HttpResponse::Ok().body("Transaction queued")
}

async fn mine(
    bc: web::Data<Arc<Mutex<Blockchain>>>,
    w: web::Data<Arc<Mutex<Wallet>>>,
) -> HttpResponse {
    let wallet = w.lock().unwrap();
    let mut chain = bc.lock().unwrap();
    chain.mine_pending(&wallet.address());
    HttpResponse::Ok().body("Block mined")
}

async fn revstop_status(w: web::Data<Arc<Mutex<Wallet>>>) -> HttpResponse {
    let wallet = w.lock().unwrap();
    HttpResponse::Ok().body(get_revstop_status(&wallet.address()))
}