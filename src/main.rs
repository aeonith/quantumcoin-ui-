//! src/main.rs
use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::sync::Mutex;
use std::env;

mod block;
mod blockchain;
mod wallet;
mod transaction;
mod revstop;
mod price_engine;
mod kyc;
mod mining;          // <— new

use blockchain::Blockchain;
use wallet::Wallet;
use revstop::{RevStop, is_revstop_active};
use price_engine::get_current_price;
use kyc::verify_user;
use mining::MINING_INFO;   // exposed as lazy-static in mining.rs

/// ---------------------------------------------------------------------------
/// Shared state
/// ---------------------------------------------------------------------------
struct AppState {
    blockchain: Mutex<Blockchain>,
    wallet:     Mutex<Wallet>,
    revstop:    Mutex<RevStop>,
}

/// ---------------------------------------------------------------------------
/// actix-web entry
/// ---------------------------------------------------------------------------
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // --- Load / create on first run -----------------------------------------------------------
    let wallet    = Wallet::load_from_files("wallet.json");
    let revstop   = RevStop::load_status("revstop_status.json");
    let mut chain = Blockchain::load_from_file("blockchain.json")
                                .unwrap_or_else(|_| Blockchain::new());

    if chain.chain_is_empty() {                       // helper we added to blockchain.rs
        chain.create_genesis_block(wallet.get_address());
    }

    // --- Bundle into Actix shared state -------------------------------------------------------
    let state = web::Data::new(AppState {
        blockchain: Mutex::new(chain),
        wallet:     Mutex::new(wallet),
        revstop:    Mutex::new(revstop),
    });

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
    println!("🚀  QuantumCoin API running on http://0.0.0.0:{port}");

    // --- Start HTTP server --------------------------------------------------------------------
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(actix_cors::Cors::permissive())                 // allow front-end fetches
            .service(Files::new("/", "./static").index_file("index.html"))
            .route("/api/price",          web::get().to(api_get_price))
            .route("/api/revstop",        web::get().to(api_check_revstop))
            .route("/api/address",        web::get().to(api_wallet_address))
            .route("/api/verify-kyc",     web::post().to(api_verify_kyc))
            .route("/api/mine",           web::post().to(api_mine_block))
            .route("/api/mining-info",    web::get().to(api_mining_info))
    })
    .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
    .run()
    .await
}

/// ---------------------------------------------------------------------------
/// ROUTES
/// ---------------------------------------------------------------------------
async fn api_get_price(data: web::Data<AppState>) -> HttpResponse {
    let chain = data.blockchain.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "price": get_current_price(&chain) }))
}

async fn api_check_revstop(data: web::Data<AppState>) -> HttpResponse {
    let revstop = data.revstop.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "revstop_enabled": revstop.enabled }))
}

async fn api_wallet_address(data: web::Data<AppState>) -> HttpResponse {
    let wallet = data.wallet.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "address": wallet.get_address() }))
}

async fn api_verify_kyc(info: web::Json<serde_json::Value>) -> HttpResponse {
    if verify_user(&info).await {
        HttpResponse::Ok().body("✅ KYC Verified")
    } else {
        HttpResponse::Unauthorized().body("❌ KYC Failed")
    }
}

async fn api_mine_block(data: web::Data<AppState>) -> HttpResponse {
    // RevStop gate
    if is_revstop_active(&data.revstop.lock().unwrap()) {
        return HttpResponse::Forbidden().body("⛔ RevStop active");
    }

    let mut chain  = data.blockchain.lock().unwrap();
    let wallet     = data.wallet.lock().unwrap();
    let reward_to  = wallet.get_address();

    if chain.mine_pending_transactions(&reward_to) {
        chain.save_to_file("blockchain.json").unwrap_or_default();
        HttpResponse::Ok().body("✅ Block mined")
    } else {
        HttpResponse::BadRequest().body("⚠️ Nothing to mine")
    }
}

async fn api_mining_info() -> HttpResponse {
    let info = MINING_INFO.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "reward":     info.get_current_reward(),
        "difficulty": info.get_current_difficulty(),
        "halvings":   info.reward_halvings
    }))
}