use actix_web::{web::Data, App, HttpServer};
use std::sync::{Arc, Mutex};

mod blockchain;
mod revstop;
mod routes;
mod transaction;
mod wallet;

use blockchain::Blockchain;
use revstop::RevStop;
use routes::init_routes;
use wallet::Wallet;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load wallet or generate new one if missing
    let wallet = Arc::new(Mutex::new(
        Wallet::load_from_files("public.key", "private.key")
            .unwrap_or_else(Wallet::new_and_save),
    ));

    // Load blockchain or initialize new one
    let blockchain = Arc::new(Mutex::new(
        Blockchain::load_from_file("blockchain.json")
            .unwrap_or_else(Blockchain::new),
    ));

    // Load RevStop status
    let revstop = Arc::new(Mutex::new(
        RevStop::load_status("revstop_status.json")
    ));

    // Start Actix Web server
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(wallet.clone()))
            .app_data(Data::new(blockchain.clone()))
            .app_data(Data::new(revstop.clone()))
            .configure(init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}