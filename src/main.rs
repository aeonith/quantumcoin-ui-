use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::sync::Mutex;

mod block;
mod blockchain;
mod transaction;
mod wallet;
mod revstop;
mod cli;

use blockchain::Blockchain;
use wallet::Wallet;
use revstop::{RevStop, is_revstop_active};

struct AppState {
    blockchain: Mutex<Blockchain>,
    wallet:     Mutex<Wallet>,
    revstop:    Mutex<RevStop>,
}

/* ===== simple REST helpers for your front-end ===== */

async fn api_wallet_address(data: web::Data<AppState>) -> HttpResponse {
    let w = data.wallet.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "address": w.get_address() }))
}

async fn api_check_revstop(data: web::Data<AppState>) -> HttpResponse {
    let r = data.revstop.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "revstop_enabled": r.enabled }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    /* --- load or bootstrap local data --- */
    let mut wallet = Wallet::load_from_files().unwrap_or_else(|_| Wallet::generate());
    let mut blockchain = Blockchain::load_from_file("blockchain.json")
        .unwrap_or_else(|| Blockchain::new_with_genesis(wallet.get_address()));
    let revstop  = RevStop::load_status().unwrap_or_else(|| RevStop::new(false));

    /* --- persist (just to ensure the files exist) --- */
    wallet.save_to_files().ok();
    blockchain.save_to_file("blockchain.json").ok();
    revstop.save_status().ok();

    let state = web::Data::new(AppState {
        blockchain: Mutex::new(blockchain),
        wallet:     Mutex::new(wallet),
        revstop:    Mutex::new(revstop),
    });

    /* --- spawn CLI in a background thread so you still get a REPL locally --- */
    {
        let s = state.clone();
        std::thread::spawn(move || {
            let mut bc = s.blockchain.lock().unwrap();
            let mut w  = s.wallet.lock().unwrap();
            cli::start_cli(&mut w, &mut bc);
        });
    }

    /* --- Actix server --- */
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/api/wallet_address", web::get().to(api_wallet_address))
            .route("/api/revstop",        web::get().to(api_check_revstop))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", std::env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().unwrap()))?
    .run()
    .await
}