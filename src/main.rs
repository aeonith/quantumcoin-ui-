mod blockchain;
mod cli;
mod p2p;
mod revstop;
mod routes;
mod transaction;
mod wallet;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};

use blockchain::Blockchain;
use wallet::Wallet;
use p2p::start_node;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // CLI entry
    if let Some(cmd) = std::env::args().nth(1) {
        return cli::run(&cmd);
    }

    // Load or create blockchain & wallet
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create()));
    let wallet = Arc::new(Mutex::new(Wallet::load_or_generate()));
    let peers = Arc::new(Mutex::new(vec![]));

    // Start P2P networking thread
    {
        let bc = blockchain.clone();
        let ps = peers.clone();
        std::thread::spawn(move || start_node(6000, bc, ps));
    }

    println!("🚀 QuantumCoin node listening at http://localhost:8080");

    // HTTP Server for API
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