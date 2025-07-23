use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::revstop::RevStop;
use crate::transaction::Transaction;
use crate::utils::get_btc_price_usd;

#[post("/send")]
pub async fn send(
    data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    form: web::Json<Transaction>,
) -> impl Responder {
    let mut chain = data.lock().unwrap();
    chain.add_transaction(Transaction {
        sender: form.sender.clone(),
        recipient: form.recipient.clone(),
        amount: form.amount,
    });
    HttpResponse::Ok().body("Transaction added.")
}

#[post("/mine")]
pub async fn mine(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    _revstop: web::Data<Arc<Mutex<RevStop>>>,
) -> impl Responder {
    let miner = wallet.lock().unwrap().get_address();
    let result = blockchain.lock().unwrap().mine_pending_transactions(&miner);
    if result {
        HttpResponse::Ok().body("Block mined.")
    } else {
        HttpResponse::InternalServerError().body("Mining failed.")
    }
}

#[get("/price")]
pub async fn btc_price() -> impl Responder {
    match get_btc_price_usd().await {
        Ok(price) => HttpResponse::Ok().body(format!("BTC = ${:.2}", price)),
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch BTC price."),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(send);
    cfg.service(mine);
    cfg.service(btc_price);
}