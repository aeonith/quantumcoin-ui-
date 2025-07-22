use actix_web::{web, HttpResponse};
use crate::{blockchain::Blockchain, wallet::Wallet, transaction::Transaction};
use std::sync::Mutex;

#[derive(serde::Deserialize)]
pub struct SendRequest {
    pub recipient: String,
    pub amount: u64,
    pub password: String,
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/wallet").route(web::get().to(wallet)))
       .service(web::resource("/balance").route(web::get().to(balance)))
       .service(web::resource("/mine").route(web::post().to(mine)))
       .service(web::resource("/send").route(web::post().to(send_tx)));
}

async fn wallet(wallet: web::Data<Mutex<Wallet>>) -> HttpResponse {
    let w = wallet.lock().unwrap();
    HttpResponse::Ok().json(&*w)
}

async fn balance(
    blockchain: web::Data<Mutex<Blockchain>>,
    wallet: web::Data<Mutex<Wallet>>,
) -> HttpResponse {
    let bc = blockchain.lock().unwrap();
    let w  = wallet.lock().unwrap();
    let b = bc.get_balance(&w.get_address());
    HttpResponse::Ok().json(serde_json::json!({ "balance": b }))
}

async fn mine(
    blockchain: web::Data<Mutex<Blockchain>>,
    wallet: web::Data<Mutex<Wallet>>,
) -> HttpResponse {
    let mut bc = blockchain.lock().unwrap();
    let w      = wallet.lock().unwrap();
    bc.mine_pending(&w.get_address());
    HttpResponse::Ok().body("Mined a new block")
}

async fn send_tx(
    blockchain: web::Data<Mutex<Blockchain>>,
    wallet: web::Data<Mutex<Wallet>>,
    req: web::Json<SendRequest>,
) -> HttpResponse {
    let mut bc = blockchain.lock().unwrap();
    let w = wallet.lock().unwrap();

    if !w.verify_password(&req.password) {
        return HttpResponse::Unauthorized().body("Bad password");
    }

    let tx = w.create_transaction(&req.recipient, req.amount);
    bc.add_transaction(tx);
    HttpResponse::Ok().body("Transaction added")
}