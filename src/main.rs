mod wallet;
mod blockchain;
mod models;

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use wallet::Wallet;
use blockchain::Blockchain;
use models::{TransactionRequest};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wallet = Arc::new(Mutex::new(Wallet::load_or_create()));
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(wallet.clone()))
            .app_data(web::Data::from(blockchain.clone()))
            .route("/address", web::get().to(get_address))
            .route("/balance", web::get().to(get_balance))
            .route("/send", web::post().to(send_transaction))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn get_address(wallet: web::Data<Mutex<Wallet>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    HttpResponse::Ok().body(wallet.get_address())
}

async fn get_balance(blockchain: web::Data<Mutex<Blockchain>>, wallet: web::Data<Mutex<Wallet>>) -> impl Responder {
    let blockchain = blockchain.lock().unwrap();
    let wallet = wallet.lock().unwrap();
    let balance = blockchain.get_balance(&wallet.get_address());
    HttpResponse::Ok().body(balance.to_string())
}

async fn send_transaction(
    blockchain: web::Data<Mutex<Blockchain>>,
    wallet: web::Data<Mutex<Wallet>>,
    req: web::Json<TransactionRequest>
) -> impl Responder {
    let mut blockchain = blockchain.lock().unwrap();
    let wallet = wallet.lock().unwrap();

    if wallet.verify_password(&req.password) {
        let tx = wallet.create_transaction(req.recipient.clone(), req.amount);
        blockchain.add_transaction(tx);
        HttpResponse::Ok().body("Transaction sent")
    } else {
        HttpResponse::Unauthorized().body("Invalid password")
    }
}