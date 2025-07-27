use actix_web::{web::Data, App, HttpServer};
use std::sync::{Arc, Mutex};

// ✅ CORS additions
use actix_cors::Cors;
use actix_web::http;

// Your modules
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
    println!("🚀 QuantumCoin API is launching...");

    // ✅ Load wallet or generate/save new one
    let wallet = Arc::new(Mutex::new(
        Wallet::load_from_files("public.key", "private.key")
            .unwrap_or_else(|| {
                let wallet = Wallet::new().expect("❌ Failed to generate wallet");
                wallet.save_to_files("public.key", "private.key").expect("❌ Failed to save wallet");
                wallet
            }),
    ));

    // ✅ Load blockchain from file or initialize
    let blockchain = Arc::new(Mutex::new(
        Blockchain::load_from_file("blockchain.json")
            .unwrap_or_else(Blockchain::new),
    ));

    // ✅ Load RevStop status
    let revstop = Arc::new(Mutex::new(
        RevStop::load_status("revstop_status.json")
    ));

    // ✅ Actix Web Server with CORS enabled
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://quantumcoincrypto.com")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::CONTENT_TYPE])
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(Data::new(wallet.clone()))
            .app_data(Data::new(blockchain.clone()))
            .app_data(Data::new(revstop.clone()))
            .configure(init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}