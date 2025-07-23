use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};

mod wallet;
mod blockchain;
mod transaction;
mod revstop;
mod routes;
mod coingecko;
mod btc;
mod p2p;

use wallet::Wallet;
use blockchain::Blockchain;
use revstop::RevStop;
use p2p::start_node;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load or generate key-based wallet
    let wallet = Arc::new(Mutex::new(Wallet::load_or_generate()));

    // Load or initialize blockchain
    let mut blockchain = Blockchain::load_or_create();

    // Create RevStop system (cold storage protection)
    let revstop = Arc::new(Mutex::new(RevStop::load_or_generate()));

    // Add genesis if not present
    if blockchain.is_empty() {
        blockchain.initialize_genesis(wallet.lock().unwrap().address());
        blockchain.save_to_file();
    }

    let blockchain = Arc::new(Mutex::new(blockchain));
    let peers = Arc::new(Mutex::new(vec![]));

    // Start P2P network
    {
        let blockchain = blockchain.clone();
        let peers = peers.clone();
        std::thread::spawn(move || {
            start_node(6000, blockchain, peers);
        });
    }

    println!("✅ QuantumCoin running on http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(wallet.clone()))
            .app_data(web::Data::new(revstop.clone()))
            .app_data(web::Data::new(peers.clone()))
            .configure(routes::init)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}