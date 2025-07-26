use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::revstop::RevStop;
use crate::transaction::Transaction;
use serde::Serialize;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(health_check)
        .service(send_transaction)
        .service(get_balance)
        .service(mine_block)
        .service(get_transactions)
        .service(create_wallet_route); // ✅ added wallet API
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

#[get("/balance/{address}")]
async fn get_balance(
    path: web::Path<String>,
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
) -> impl Responder {
    let address = path.into_inner();
    let chain = blockchain.lock().unwrap();
    let balance = chain.get_balance(&address);
    HttpResponse::Ok().body(balance.to_string())
}

#[post("/mine")]
async fn mine_block(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
) -> impl Responder {
    let mut chain = blockchain.lock().unwrap();
    let default_miner = "SYSTEM".to_string(); // Change if needed
    chain.mine_pending_transactions(default_miner);
    HttpResponse::Ok().body("✅ Mined new block")
}

#[get("/transactions")]
async fn get_transactions(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
) -> impl Responder {
    let chain = blockchain.lock().unwrap();
    let txs = chain.get_all_transactions();
    HttpResponse::Ok().json(txs)
}

#[derive(Serialize)]
struct WalletResponse {
    publicKey: String,
    privateKey: String,
}

#[get("/api/create-wallet")]
async fn create_wallet_route(
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let response = WalletResponse {
        publicKey: wallet.get_public_key(),
        privateKey: wallet.get_private_key(),
    };
    HttpResponse::Ok().json(response)
}