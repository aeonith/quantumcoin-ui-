use actix_web::{web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::revstop::RevStop;
use crate::transaction::Transaction;
use crate::utils::get_btc_price_usd;

pub async fn send_transaction(
    data: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    req_body: String,
) -> impl Responder {
    let tx_data: Vec<&str> = req_body.trim().split(',').collect();
    if tx_data.len() != 3 {
        return HttpResponse::BadRequest().body("Invalid input");
    }

    let from = tx_data[0].to_string();
    let to = tx_data[1].to_string();
    let amount: u64 = match tx_data[2].parse() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Invalid amount"),
    };

    let tx = Transaction::new(from, to, amount, None);
    let mut chain = data.lock().unwrap();
    chain.add_transaction(tx);

    HttpResponse::Ok().body("Transaction added")
}

pub async fn mine_block(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    miner: web::Query<String>,
) -> impl Responder {
    let mut chain = blockchain.lock().unwrap();
    let result = chain.mine_pending(&miner);
    HttpResponse::Ok().body(format!("Block mined: {}", result))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/send").route(web::post().to(send_transaction)));
    cfg.service(web::resource("/mine").route(web::post().to(mine_block)));
}