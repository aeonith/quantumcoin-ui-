use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::revstop::RevStop;
use crate::transaction::Transaction;
// Optional:
// use crate::utils::get_btc_price_usd;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
    cfg.service(send_transaction);
}

#[get("/")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("QuantumCoin API is live")
}

#[post("/send")]
async fn send_transaction(
    data: web::Json<Transaction>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let tx = data.into_inner();
    let mut chain = blockchain.lock().unwrap();
    chain.add_transaction(tx);
    HttpResponse::Ok().body("Transaction added")
}