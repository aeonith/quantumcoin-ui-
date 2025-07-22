mod wallet;
mod block;
mod blockchain;
mod transaction;
mod revstop;
mod routes;
mod p2p;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::{Arc, Mutex};
use blockchain::Blockchain;
use wallet::Wallet;
use p2p::start_node;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load blockchain and wallet
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create()));
    let wallet = Arc::new(Mutex::new(Wallet::load_or_generate()));
    let peers = Arc::new(Mutex::new(vec![]));

    // Start P2P networking in separate thread
    let p2p_peers = peers.clone();
    std::thread::spawn(move || start_node(6000, p2p_peers));

    println!("✅ QuantumCoin Node running at http://localhost:8080");

    // HTTP Server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::from(blockchain.clone()))
            .app_data(web::Data::from(wallet.clone()))
            .app_data(web::Data::from(peers.clone()))
            .configure(routes::init)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}