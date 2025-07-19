mod wallet;
mod block;
mod transaction;
mod blockchain;
mod revstop;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use parking_lot::Mutex;
use std::sync::Arc;
use std::path::Path;
use std::time::Instant;

use wallet::Wallet;
use blockchain::Blockchain;

struct AppState {
    wallet: Mutex<Wallet>,
    blockchain: Mutex<Blockchain>,
}

// GET /address
async fn get_address(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock();
    HttpResponse::Ok().body(wallet.get_address())
}

// GET /balance
async fn get_balance(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock();
    let address = wallet.get_address();
    let chain = data.blockchain.lock();
    let bal = chain.get_balance(&address);
    HttpResponse::Ok().body(format!("{}", bal))
}

// POST /mine
async fn mine(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock();
    let addr = wallet.get_address();
    let mut chain = data.blockchain.lock();
    chain.mine_pending_transactions(addr.clone());
    chain.save_to_disk();
    HttpResponse::Ok().body("Mined one block")
}

// POST /send
async fn send_tx(tx: web::Json<transaction::Transaction>, data: web::Data<AppState>) -> impl Responder {
    let mut chain = data.blockchain.lock();
    chain.add_transaction(tx.into_inner());
    HttpResponse::Ok().body("Transaction added")
}

// POST /revstop/lock
async fn revstop_lock(data: web::Data<AppState>) -> impl Responder {
    let mut chain = data.blockchain.lock();
    revstop::lock_revstop(&mut chain);
    HttpResponse::Ok().body("RevStop locked")
}

// POST /revstop/unlock
async fn revstop_unlock(data: web::Data<AppState>) -> impl Responder {
    let mut chain = data.blockchain.lock();
    revstop::unlock_revstop(&mut chain);
    HttpResponse::Ok().body("RevStop unlocked")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    println!("🚀 QuantumCoin Web API starting...");

    // Wallet load/create
    let pub_path = "wallet_public.key";
    let priv_path = "wallet_private.key";
    let wallet = if Path::new(pub_path).exists() {
        Wallet::load_from_files(pub_path, priv_path)
            .expect("Failed to load wallet")
    } else {
        let w = Wallet::new();
        w.save_to_files(pub_path, priv_path);
        w
    };
    println!("🔐 Address: {}", wallet.get_address());

    // Blockchain load/create
    let start = Instant::now();
    let blockchain = Blockchain::load_from_disk().unwrap_or_else(|| {
        println!("📦 Creating genesis...");
        Blockchain::new(wallet.get_address().clone())
    });
    println!("✅ Chain loaded in {:?}", start.elapsed());

    let state = Arc::new(AppState {
        wallet: Mutex::new(wallet),
        blockchain: Mutex::new(blockchain),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::from(state.clone()))
            .route("/address", web::get().to(get_address))
            .route("/balance", web::get().to(get_balance))
            .route("/mine", web::post().to(mine))
            .route("/send", web::post().to(send_tx))
            .route("/revstop/lock", web::post().to(revstop_lock))
            .route("/revstop/unlock", web::post().to(revstop_unlock))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}