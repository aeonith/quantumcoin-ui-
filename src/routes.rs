use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};

use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use crate::revstop::RevStop;

#[derive(serde::Deserialize)]
pub struct SendData {
    pub recipient: String,
    pub amount: u64,
    pub signature: String,
}

#[post("/send")]
async fn send(
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    data: web::Json<SendData>,
) -> impl Responder {
    let wallet = wallet.lock().unwrap();

    // Signature verification
    let sig = data.signature.clone();
    if !wallet.verify(data.recipient.as_bytes(), &sig) {
        return HttpResponse::Unauthorized().body("Invalid signature");
    }

    let from = wallet.get_address();
    let to = data.recipient.clone();
    let amount = data.amount;

    drop(wallet); // release lock before accessing blockchain
    let mut chain = blockchain.lock().unwrap();
    chain.add_transaction(from, to, amount);

    HttpResponse::Ok().body("Transaction added to mempool")
}

#[post("/mine")]
async fn mine(
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
) -> impl Responder {
    let miner = wallet.lock().unwrap().get_address();
    let result = blockchain.lock().unwrap().mine_pending(&miner);
    HttpResponse::Ok().body(result)
}

#[get("/revstop-status")]
async fn revstop_status(
    revstop: web::Data<Arc<Mutex<RevStop>>>
) -> impl Responder {
    let locked = RevStop::is_active();
    HttpResponse::Ok().body(if locked { "Locked" } else { "Unlocked" })
}

#[get("/balance")]
async fn balance(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let balance = wallet.lock().unwrap().get_balance();
    HttpResponse::Ok().body(format!("Balance: {:.2} QTC", balance))
}

#[post("/airdrop")]
async fn airdrop(
    wallet: web::Data<Arc<Mutex<Wallet>>>
) -> impl Responder {
    use crate::utils::get_btc_price_usd;

    let btc_price = match get_btc_price_usd().await {
        Ok(p) => p,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch BTC price"),
    };

    let qtc_amount: u64 = ((btc_price * 10.0) as u64).min(1000);
    let mut user_wallet = wallet.lock().unwrap();
    user_wallet.add_balance(qtc_amount as f64);

    HttpResponse::Ok().body(format!("Airdropped {} QTC", qtc_amount))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(send)
        .service(mine)
        .service(revstop_status)
        .service(balance)
        .service(airdrop);
}