use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use std::sync::Mutex;

#[derive(Deserialize)]
pub struct SendRequest {
    sender: String,
    recipient: String,
    amount: f64,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    address: String,
    balance: f64,
}

#[derive(Serialize)]
pub struct MessageResponse {
    message: String,
}

// Shared Blockchain state
pub struct AppState {
    pub blockchain: Mutex<Blockchain>,
    pub wallet: Mutex<Wallet>,
}

async fn get_balance(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let address = path.into_inner();
    let chain = data.blockchain.lock().unwrap();
    let balance = chain.get_balance(&address);
    HttpResponse::Ok().json(BalanceResponse { address, balance })
}

async fn send_transaction(data: web::Data<AppState>, tx: web::Json<SendRequest>) -> impl Responder {
    let mut chain = data.blockchain.lock().unwrap();
    let sender = tx.sender.clone();
    let recipient = tx.recipient.clone();
    let amount = tx.amount;

    match chain.create_transaction(&sender, &recipient, amount) {
        Ok(_) => HttpResponse::Ok().json(MessageResponse {
            message: "Transaction queued.".to_string(),
        }),
        Err(e) => HttpResponse::BadRequest().json(MessageResponse {
            message: format!("Error: {}", e),
        }),
    }
}

async fn mine(data: web::Data<AppState>) -> impl Responder {
    let mut chain = data.blockchain.lock().unwrap();
    let miner_address = "QuantumMinerRewardAddress"; // Replace or make dynamic
    let message = chain.mine_pending_transactions(miner_address);
    HttpResponse::Ok().json(MessageResponse { message })
}

async fn transactions(data: web::Data<AppState>) -> impl Responder {
    let chain = data.blockchain.lock().unwrap();
    let txs = chain.get_all_transactions(); // You should implement this in blockchain.rs
    HttpResponse::Ok().json(txs)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/balance/{address}", web::get().to(get_balance))
        .route("/send", web::post().to(send_transaction))
        .route("/mine", web::post().to(mine))
        .route("/transactions", web::get().to(transactions));
}

pub async fn run_server(blockchain: Blockchain, wallet: Wallet) -> std::io::Result<()> {
    let data = web::Data::new(AppState {
        blockchain: Mutex::new(blockchain),
        wallet: Mutex::new(wallet),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .configure(init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}