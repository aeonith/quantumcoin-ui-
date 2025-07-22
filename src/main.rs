use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use std::sync::Mutex;

mod blockchain;
mod routes;
mod transaction;
mod wallet;

use blockchain::Blockchain;
use routes::{get_balance, send_transaction};
use wallet::Wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize wallet and blockchain
    let wallet = Wallet::load_or_generate("wallet.json".to_string());
    let blockchain = Blockchain::load_or_create("blockchain.json".to_string());

    let wallet_data = web::Data::new(Mutex::new(wallet));
    let blockchain_data = web::Data::new(Mutex::new(blockchain));

    println!("🚀 QuantumCoin backend running at https://quantumcoin-it.onrender.com");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://quantumcoincrypto.com")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(wallet_data.clone())
            .app_data(blockchain_data.clone())
            .service(get_balance)
            .service(send_transaction)
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}