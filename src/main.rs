use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};

mod blockchain;
mod transaction;
mod wallet;
mod revstop;
mod routes;
mod utils;

use blockchain::Blockchain;
use wallet::Wallet;
use revstop::RevStop;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let wallet = Arc::new(Mutex::new(Wallet::load_from_files("public
    let wallet = Arc::new(Mutex::new(Wallet::new()));
    let revstop = Arc::new(Mutex::new(RevStop::load_status("revstop_status.json")));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(wallet.clone()))
            .app_data(web::Data::new(revstop.clone()))
            .configure(routes::init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}