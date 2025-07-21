use actix_files::Files;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use std::sync::Mutex;
use std::env;

mod block;
mod blockchain;
mod wallet;
mod transaction;
mod revstop;
mod price_engine;
mod kyc;

use blockchain::Blockchain;
use wallet::Wallet;
use revstop::{RevStop, is_revstop_active};
use price_engine::get_current_price;
use kyc::verify_user;

struct AppState {
    blockchain: Mutex<Blockchain>,
    wallet: Mutex<Wallet>,
    revstop: Mutex<RevStop>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wallet = Wallet::load_from_files("wallet_public.key", "wallet_private.key").unwrap();
    let revstop = RevStop::load_status("revstop_status.json").unwrap_or_default();
    let mut blockchain = Blockchain::load_from_file("blockchain.json").unwrap_or_else(Blockchain::new);

    if blockchain.chain.is_empty() {
        blockchain.create_genesis_block(wallet.get_address());
    }

    let state = web::Data::new(AppState {
        blockchain: Mutex::new(blockchain),
        wallet: Mutex::new(wallet),
        revstop: Mutex::new(revstop),
    });

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    println!("🚀 QuantumCoin API running on http://localhost:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(actix_cors::Cors::permissive())
            .service(Files::new("/", "./static").index_file("index.html"))
            .route("/api/price", web::get().to(api_get_price))
            .route("/api/verify-kyc", web::post().to(api_verify_kyc))
            .route("/api/revstop", web::get().to(api_check_revstop))
            .route("/api/mine", web::post().to(api_mine_block))
            .route("/api/address", web::get().to(api_wallet_address))
    })
    .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
    .run()
    .await
}

// === API ROUTES ===

async fn api_get_price(data: web::Data<AppState>) -> impl Responder {
    let blockchain = data.blockchain.lock().unwrap();
    let price = get_current_price(&blockchain);
    HttpResponse::Ok().json(serde_json::json!({ "price": price }))
}

async fn api_check_revstop(data: web::Data<AppState>) -> impl Responder {
    let revstop = data.revstop.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "revstop_enabled": revstop.enabled
    }))
}

async fn api_wallet_address(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "address": wallet.get_address()
    }))
}

async fn api_verify_kyc(info: web::Json<serde_json::Value>) -> impl Responder {
    let result = verify_user(&info).await;
    if result {
        HttpResponse::Ok().body("✅ KYC Verified")
    } else {
        HttpResponse::Unauthorized().body("❌ KYC Failed")
    }
}

async fn api_mine_block(data: web::Data<AppState>) -> impl Responder {
    let mut blockchain = data.blockchain.lock().unwrap();
    let mut wallet = data.wallet.lock().unwrap();

    if is_revstop_active(&data.revstop.lock().unwrap()) {
        return HttpResponse::Forbidden().body("RevStop enabled: Mining denied.");
    }

    let reward_addr = wallet.get_address();
    let success = blockchain.mine_pending_transactions(&reward_addr);

    if success {
        blockchain.save_to_file("blockchain.json").unwrap();
        HttpResponse::Ok().body("✅ Block mined successfully")
    } else {
        HttpResponse::BadRequest().body("⚠️ Mining failed")
    }
}