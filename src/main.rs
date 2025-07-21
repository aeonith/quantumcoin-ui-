mod block;
mod blockchain;
mod cli;
mod kyc;
mod mining;
mod revstop;
mod transaction;
mod wallet;

use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::sync::Mutex;

use crate::blockchain::Blockchain;
use crate::revstop::RevStop;
use crate::wallet::Wallet;

struct AppState {
    blockchain: Mutex<Blockchain>,
    wallet:     Mutex<Wallet>,
    revstop:    Mutex<RevStop>,
}

async fn api_wallet_address(data: web::Data<AppState>) -> HttpResponse {
    let w = data.wallet.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "address": w.get_address() }))
}

async fn api_balance(data: web::Data<AppState>) -> HttpResponse {
    let w = data.wallet.lock().unwrap();
    let bc = data.blockchain.lock().unwrap();
    let bal = w.get_balance(&bc);
    HttpResponse::Ok().json(serde_json::json!({ "balance": bal }))
}

async fn api_revstop(data: web::Data<AppState>) -> HttpResponse {
    let r = data.revstop.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "enabled": r.is_active() }))
}

async fn api_mine(data: web::Data<AppState>) -> HttpResponse {
    let mut bc = data.blockchain.lock().unwrap();
    let w_addr = data.wallet.lock().unwrap().get_address().clone();
    bc.mine_pending_transactions(&w_addr);
    HttpResponse::Ok().json(serde_json::json!({ "message": "Block mined" }))
}

async fn api_kyc_verify(form: web::Json<kyc::KycForm>) -> HttpResponse {
    if kyc::verify(&form.email, &form.code) {
        HttpResponse::Ok().json(serde_json::json!({ "status": "verified" }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({ "status": "failed" }))
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // --- Initialize or load wallet ---
    let wallet_dir = "data";
    let mut wallet = Wallet::load_from_files(wallet_dir)?
        .unwrap_or_else(|| Wallet::generate_and_save(wallet_dir).unwrap());

    // --- Initialize or load blockchain ---
    let mut blockchain = Blockchain::load("blockchain.json")?
        .unwrap_or_else(|| Blockchain::new_with_genesis(wallet.get_address()));

    // --- Initialize or load revstop ---
    let mut revstop = RevStop::load("revstop.json")?
        .unwrap_or_else(|| RevStop::default_and_save("revstop.json").unwrap());

    // --- Persist defaults ---
    wallet.save(wallet_dir).ok();
    blockchain.save("blockchain.json").ok();
    revstop.save_status("revstop.json").ok();

    // --- Shared state ---
    let state = web::Data::new(AppState {
        blockchain: Mutex::new(blockchain),
        wallet:     Mutex::new(wallet),
        revstop:    Mutex::new(revstop),
    });

    // --- Spawn CLI ---
    {
        let s = state.clone();
        std::thread::spawn(move || {
            let mut w  = s.wallet.lock().unwrap();
            let mut bc = s.blockchain.lock().unwrap();
            let mut rs = s.revstop.lock().unwrap();
            cli::start_cli(&mut w, &mut bc, &mut rs);
        });
    }

    // --- Start HTTP server ---
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8080".into()).parse()?;
    println!("🚀 Listening 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/api/address", web::get().to(api_wallet_address))
            .route("/api/balance", web::get().to(api_balance))
            .route("/api/revstop",  web::get().to(api_revstop))
            .route("/api/mine",     web::post().to(api_mine))
            .route("/api/kyc",      web::post().to(api_kyc_verify))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}