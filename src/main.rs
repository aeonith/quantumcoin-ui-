use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};

mod wallet;
mod blockchain;
mod transaction;
mod revstop;
mod routes;
mod btc;
mod coingecko;

use wallet::Wallet;
use blockchain::Blockchain;
use revstop::RevStop;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin Backend Starting...");

    let wallet = Arc::new(Mutex::new(Wallet::load_or_create()));
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create(wallet.clone())));
    let revstop = Arc::new(Mutex::new(RevStop::load_status()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(wallet.clone()))
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(revstop.clone()))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}