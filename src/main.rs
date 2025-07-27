mod blockchain;
mod block;
mod transaction;
mod wallet;
mod revstop;
mod routes;

use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};
use blockchain::Blockchain;
use wallet::Wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin Engine Initialized");

    // Load blockchain
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // Load or generate wallet
    let wallet = Wallet::load_from_files("wallet_public.key", "wallet_private.key")
        .unwrap_or_else(|| {
            let w = Wallet::generate();
            w.save_to_files("wallet_public.key", "wallet_private.key").unwrap();
            w
        });

    let wallet_data = Arc::new(Mutex::new(wallet));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(wallet_data.clone()))
            .configure(routes::init_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}