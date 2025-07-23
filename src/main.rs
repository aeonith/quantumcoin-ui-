mod wallet;
mod blockchain;
mod revstop;
mod routes;

use actix_web::{App, HttpServer};
use std::sync::{Arc, Mutex};
use wallet::Wallet;
use blockchain::Blockchain;
use revstop::RevStop;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wallet = Arc::new(Mutex::new(Wallet::load_from_files().unwrap_or_else(Wallet::new)));
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create()));
    let revstop = Arc::new(Mutex::new(RevStop::status()));

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(wallet.clone()))
            .app_data(actix_web::web::Data::new(blockchain.clone()))
            .app_data(actix_web::web::Data::new(revstop.clone()))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}