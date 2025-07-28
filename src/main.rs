mod wallet;
mod transaction;
mod block;
mod blockchain;
mod routes;

use actix_web::{App, HttpServer, web};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin Engine Initialized");

    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let wallet = Arc::new(Mutex::new(Wallet::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(wallet.clone()))
            .configure(routes::init_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}