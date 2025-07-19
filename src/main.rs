use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use std::fs;
use serde::{Serialize, Deserialize};

mod blockchain;
mod wallet;
mod revstop;

use blockchain::Blockchain;
use wallet::{Wallet, Transaction};

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    version: String,
}

// Shared App State
struct AppState {
    blockchain: Mutex<Blockchain>,
}

// Health check
async fn health() -> impl Responder {
    web::Json(HealthStatus {
        status: "OK".to_string(),
        version: "1.0.0".to_string(),
    })
}

// View blockchain
async fn get_chain(data: web::Data<AppState>) -> impl Responder {
    let chain = data.blockchain.lock().unwrap();
    HttpResponse::Ok().json(&chain.chain)
}

// Create transaction
#[derive(Deserialize)]
struct TxRequest {
    sender: String,
    recipient: String,
    amount: f64,
    signature: String,
}

async fn new_transaction(req: web::Json<TxRequest>, data: web::Data<AppState>) -> impl Responder {
    let tx = Transaction {
        sender: req.sender.clone(),
        recipient: req.recipient.clone(),
        amount: req.amount,
        signature: Some(req.signature.clone()),
    };

    let mut chain = data.blockchain.lock().unwrap();
    chain.add_transaction(tx);
    HttpResponse::Ok().body("Transaction added")
}

// Mine block
async fn mine(data: web::Data<AppState>) -> impl Responder {
    let mut chain = data.blockchain.lock().unwrap();
    chain.mine_pending_transactions();
    HttpResponse::Ok().body("Block mined")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load or initialize blockchain
    let blockchain = match Blockchain::load_from_disk() {
        Ok(bc) => bc,
        Err(_) => Blockchain::new("GENESIS_PUBLIC_KEY_STRING".to_string()),
    };

    println!("🚀 QuantumCoin™ Backend Server Running on http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                blockchain: Mutex::new(blockchain.clone()),
            }))
            .route("/health", web::get().to(health))
            .route("/chain", web::get().to(get_chain))
            .route("/transaction", web::post().to(new_transaction))
            .route("/mine", web::post().to(mine))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}