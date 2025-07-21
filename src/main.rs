use actix_files::Files;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use quantumcoin::{blockchain::Blockchain, transaction::Transaction, wallet::Wallet};

mod blockchain;
mod block;
mod transaction;
mod wallet;
mod revstop;

static BLOCKCHAIN: Lazy<Mutex<Blockchain>> = Lazy::new(|| {
    let mut blockchain = Blockchain::new();
    blockchain.load_from_file("blockchain.json");
    Mutex::new(blockchain)
});

async fn status() -> impl Responder {
    HttpResponse::Ok().body("🚀 QuantumCoin API is live")
}

async fn get_balance(data: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    if let Some(addr) = data.get("address") {
        let blockchain = BLOCKCHAIN.lock().unwrap();
        let balance = blockchain.get_balance(addr);
        HttpResponse::Ok().body(format!("Balance for {}: {}", addr, balance))
    } else {
        HttpResponse::BadRequest().body("Missing 'address' parameter")
    }
}

async fn send_transaction(tx: web::Json<Transaction>) -> impl Responder {
    let mut blockchain = BLOCKCHAIN.lock().unwrap();
    blockchain.add_transaction(tx.into_inner());
    blockchain.save_to_file("blockchain.json");
    HttpResponse::Ok().body("Transaction received and added to mempool.")
}

async fn mine() -> impl Responder {
    let mut blockchain = BLOCKCHAIN.lock().unwrap();
    blockchain.mine_pending_transactions("Quantum_Miner".to_string());
    blockchain.save_to_file("blockchain.json");
    HttpResponse::Ok().body("Block mined and added to blockchain.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🔗 QuantumCoin node starting...");
    HttpServer::new(|| {
        App::new()
            .route("/status", web::get().to(status))
            .route("/balance", web::get().to(get_balance))
            .route("/transaction", web::post().to(send_transaction))
            .route("/mine", web::post().to(mine))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}