mod blockchain;
mod wallet;
mod revstop;
mod routes;
mod p2p;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};
use blockchain::Blockchain;
use wallet::Wallet;
use p2p::start_node;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load blockchain and wallet
    let blockchain = Arc::new(Mutex::new(Blockchain::load_from_file().unwrap_or_else(|| {
        Blockchain::new("tNzCy5NT+GORGIA+JCVIGAJUIBM...QNSATLVTHNBWXMZA783YP/ALNCM2GEAO1TZ==")
    })));
    let wallet = Arc::new(Mutex::new(Wallet::load_from_files()));
    let peers = Arc::new(Mutex::new(vec![]));

    // Start P2P networking
    let chain_clone = blockchain.clone();
    let peers_clone = peers.clone();
    std::thread::spawn(move || {
        start_node(6000, chain_clone, peers_clone);
    });

    println!("✅ QuantumCoin Node running at http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin().allow_any_method().allow_any_header();

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