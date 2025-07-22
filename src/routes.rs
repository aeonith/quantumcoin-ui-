use actix_web::{web, HttpResponse, Responder};
use crate::{blockchain::Blockchain, wallet::Wallet, transaction::Transaction};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct SendRequest {
    pub recipient: String,
    pub amount: u64,
    pub password: String,
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/wallet").route(web::get().to(get_wallet)))
       .service(web::resource("/balance").route(web::get().to(get_balance)))
       .service(web::resource("/mine").route(web::post().to(mine_block)))
       .service(web::resource("/send").route(web::post().to(send_transaction)))
       .service(web::resource("/info").route(web::get().to(get_address)));
}

async fn get_wallet(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let w = wallet.lock().unwrap();
    HttpResponse::Ok().json(&*w)
}

async fn get_balance(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let bc = blockchain.lock().unwrap();
    let w  = wallet.lock().unwrap();
    let balance = bc.get_balance(&w.get_address());
    HttpResponse::Ok().json(serde_json::json!({ "balance": balance }))
}

async fn mine_block(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let mut bc = blockchain.lock().unwrap();
    let w = wallet.lock().unwrap();
    bc.mine_pending(&w.get_address());
    bc.save();
    HttpResponse::Ok().body("Block mined successfully")
}

async fn send_transaction(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> impl Responder {
    let mut bc = blockchain.lock().unwrap();
    let w = wallet.lock().unwrap();

    if !w.verify_password(&req.password) {
        return HttpResponse::Unauthorized().body("Incorrect password");
    }

    let tx = w.create_transaction(&req.recipient, req.amount);
    bc.add_transaction(tx);
    bc.save();
    HttpResponse::Ok().body("Transaction submitted successfully")
}

async fn get_address(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let w = wallet.lock().unwrap();
    HttpResponse::Ok().json(w.get_address())
}