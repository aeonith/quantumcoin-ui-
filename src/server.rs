use actix_web::{App, HttpServer};
use actix_cors::Cors;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::p2p::start_node;
use crate::routes;

pub async fn run_server() -> std::io::Result<()> {
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create()));
    let wallet = Arc::new(Mutex::new(Wallet::load_or_generate()));
    let peers = Arc::new(Mutex::new(vec![]));

    let p2p_peers = peers.clone();
    std::thread::spawn(move || start_node(6000, p2p_peers));

    println!("✅ QuantumCoin node running at http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(actix_web::web::Data::from(blockchain.clone()))
            .app_data(actix_web::web::Data::from(wallet.clone()))
            .app_data(actix_web::web::Data::from(peers.clone()))
            .configure(routes::init)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}