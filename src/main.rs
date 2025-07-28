use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};

mod blockchain;
mod revstop;
mod transaction;
mod wallet;
mod routes;

use blockchain::Blockchain;
use wallet::Wallet;
use routes::init_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin Engine Initializing...");

    // Load or create wallet
    let wallet = Wallet::load_from_files("wallet_public.key", "wallet_private.key").unwrap_or_else(|_| {
        let w = Wallet::new().expect("❌ Failed to generate wallet");
        w.save_to_files("wallet_public.key", "wallet_private.key")
            .expect("❌ Failed to save wallet files");
        println!("🔐 New wallet created and saved.");
        w
    });

    // Initialize blockchain
    let mut blockchain = Blockchain::new();
    blockchain.load_from_disk("blockchain.json");
    println!("📦 Blockchain loaded.");

    // Shared state
    let wallet_data = Arc::new(Mutex::new(wallet));
    let blockchain_data = Arc::new(Mutex::new(blockchain));

    // Launch server
    println!("🌐 Starting QuantumCoin API server at http://0.0.0.0:8080...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain_data.clone()))
            .app_data(web::Data::new(wallet_data.clone()))
            .configure(init_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}