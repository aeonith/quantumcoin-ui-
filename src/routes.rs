use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use base64::{engine::general_purpose, Engine as _};

use crate::{blockchain::Blockchain, transaction::Transaction, wallet::Wallet, revstop::RevStop};
use crate::coingecko::get_btc_price_usd;
use crate::btc::get_btc_payment_status;

#[derive(Deserialize)]
pub struct SendRequest {
    pub recipient: String,
    pub amount: u64,
    pub signature: String,
}

#[derive(Deserialize)]
pub struct BuyRequest {
    pub user_wallet: String,
    pub txid: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/balance", web::get().to(get_balance));
    cfg.route("/send", web::post().to(send));
    cfg.route("/mine", web::post().to(mine));
    cfg.route("/revstop/status", web::get().to(revstop_status));
    cfg.route("/buy-bitcoin", web::post().to(buy_bitcoin));
}

async fn get_balance(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let balance = wallet.get_balance();
    HttpResponse::Ok().body(format!("Your balance: {} QTC", balance))
}

async fn send(
    data: web::Json<SendRequest>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
) -> impl Responder {
    let decoded_sig = general_purpose::STANDARD.decode(&data.signature);
    if decoded_sig.is_err() {
        return HttpResponse::BadRequest().body("Invalid signature format");
    }

    let sig = decoded_sig.unwrap();
    let wallet = wallet.lock().unwrap();
    if !wallet.verify(data.recipient.as_bytes(), &sig) {
        return HttpResponse::Unauthorized().body("Signature verification failed");
    }

    let tx = Transaction::new(
        wallet.address(),
        data.recipient.clone(),
        data.amount,
        Some(data.signature.clone()),
    );

    let mut chain = blockchain.lock().unwrap();
    chain.add_transaction(tx);
    chain.save_to_file();

    HttpResponse::Ok().body("Transaction added to mempool.")
}

async fn mine(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let mut chain = blockchain.lock().unwrap();
    let miner = wallet.lock().unwrap().address();
    let result = chain.mine_pending(&miner);
    chain.save_to_file();

    match result {
        Some(block) => HttpResponse::Ok().json(block),
        None => HttpResponse::Ok().body("No transactions to mine"),
    }
}

async fn revstop_status(revstop: web::Data<Arc<Mutex<RevStop>>>) -> impl Responder {
    let locked = revstop.lock().unwrap().is_active();
    let status = if locked { "Locked" } else { "Unlocked" };
    HttpResponse::Ok().json(StatusResponse {
        status: status.to_string(),
    })
}

async fn buy_bitcoin(
    data: web::Json<BuyRequest>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let btc_price = match get_btc_price_usd().await {
        Ok(p) => p,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch BTC price"),
    };

    let paid = get_btc_payment_status(&data.txid).await;
    if !paid {
        return HttpResponse::BadRequest().body("BTC payment not confirmed yet.");
    }

    let qtc_amount = (10.0 / btc_price * 100.0) as u64; // $10 minimum
    let mut user_wallet = wallet.lock().unwrap();
    user_wallet.add_balance(qtc_amount);

    HttpResponse::Ok().body(format!("You purchased {} QTC!", qtc_amount))
}