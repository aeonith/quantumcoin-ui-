mod p2p;
mod blockchain;
mod wallet;
mod transaction;
mod routes;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::{Arc, Mutex};
use std::env;
use blockchain::Blockchain;
use wallet::Wallet;
use p2p::start_node;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load or create blockchain and wallet with safe error handling
    let blockchain = Arc::new(Mutex::new(
        Blockchain::load_or_create().unwrap_or_else(|_| Blockchain::new())
    ));
    let wallet = Arc::new(Mutex::new(
        Wallet::load_or_generate().unwrap_or_else(|_| Wallet::new())
    ));

    // Start peer-to-peer networking
    let peers = Arc::new(Mutex::new(vec![]));
    let p2p_peers = peers.clone();
    std::thread::spawn(move || start_node(6000, p2p_peers));

    // Use port from environment or default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 QuantumCoin Node & API live on http://{}", addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://quantumcoincrypto.com")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Content-Type"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::from(blockchain.clone()))
            .app_data(web::Data::from(wallet.clone()))
            .configure(routes::init)
    })
    .bind(addr)?
    .run()
    .await
}