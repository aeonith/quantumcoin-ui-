mod p2p;
mod blockchain;
mod wallet;
mod transaction;
mod routes;
mod auth;
mod user;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::{Arc, Mutex};
use blockchain::Blockchain;
use wallet::Wallet;
use p2p::start_node;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load blockchain and a shared test wallet (initial user only)
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create()));
    let wallet     = Arc::new(Mutex::new(Wallet::generate_with_password("admin123")));

    // Start P2P node on port 6000
    let peers = Arc::new(Mutex::new(vec![]));
    let p2p_peers = peers.clone();
    std::thread::spawn(move || start_node(6000, p2p_peers));

    println!("🚀 QuantumCoin Node + API running on http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://quantumcoincrypto.com")
            .allowed_methods(vec!["GET","POST"])
            .allowed_headers(vec!["Content-Type"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::from(blockchain.clone()))
            .app_data(web::Data::from(wallet.clone()))
            .configure(routes::init)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}