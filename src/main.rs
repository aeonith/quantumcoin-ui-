mod transaction;
mod wallet;
mod blockchain;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::sync::Mutex;
use blockchain::Blockchain;
use wallet::Wallet;

#[derive(Debug, Deserialize)]
struct TransactionRequest {
    to: String,
    amount: u64,
    password: String,
}

#[get("/balance")]
async fn get_balance(wallet: web::Data<Wallet>, blockchain: web::Data<Mutex<Blockchain>>) -> impl Responder {
    let wallet = wallet.get_ref();
    let blockchain = blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&wallet.get_address());
    HttpResponse::Ok().body(format!("Balance for {}: {}", wallet.get_address(), balance))
}

#[post("/send")]
async fn send_transaction(
    wallet: web::Data<Wallet>,
    blockchain: web::Data<Mutex<Blockchain>>,
    info: web::Json<TransactionRequest>,
) -> impl Responder {
    let wallet = wallet.get_ref();

    if !wallet.verify_password(&info.password) {
        return HttpResponse::Unauthorized().body("Invalid password");
    }

    let tx = wallet.create_transaction(info.to.clone(), info.amount);
    let mut blockchain = blockchain.lock().unwrap();
    blockchain.add_transaction(tx);

    HttpResponse::Ok().body("Transaction added")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wallet = web::Data::new(Wallet {
        address: "quantum-wallet-001".to_string(),
        password: "secret".to_string(),
    });

    let blockchain = web::Data::new(Mutex::new(Blockchain {
        transactions: vec![],
    }));

    println!("🚀 Server running at http://localhost:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(wallet.clone())
            .app_data(blockchain.clone())
            .service(get_balance)
            .service(send_transaction)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}