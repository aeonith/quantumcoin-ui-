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

    // Load blockchain from disk or initialize
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // Load or generate wallet
    let wallet = Wallet::load_from_files("wallet_public.key", "wallet_private.key")
        .unwrap_or_else(|| {
            let w = Wallet::new().expect("❌ Failed to generate wallet");
            w.save_to_files("wallet_public.key", "wallet_private.key")
                .expect("❌ Failed to save wallet");
            println!("✅ New wallet generated and saved.");
            w
        });

    let wallet_data = Arc::new(Mutex::new(wallet));

    // Launch HTTP server
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