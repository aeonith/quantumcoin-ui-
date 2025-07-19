mod wallet;
mod blockchain;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use wallet::Wallet;
use blockchain::Blockchain;
use std::path::Path;
use std::sync::Mutex;

use std::time::Instant;

struct AppState {
    wallet: Mutex<Wallet>,
    blockchain: Mutex<Blockchain>,
}

// 📡 GET /address
async fn get_address(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock().unwrap();
    HttpResponse::Ok().body(wallet.get_address())
}

// 📡 GET /balance
async fn get_balance(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock().unwrap();
    let address = wallet.get_address();
    let blockchain = data.blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&address);
    HttpResponse::Ok().body(format!("Balance: {} QTC", balance))
}

// 📡 POST /mine
async fn mine(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock().unwrap();
    let address = wallet.get_address();
    let mut blockchain = data.blockchain.lock().unwrap();

    blockchain.mine_pending_transactions(address.clone());
    blockchain.save_to_disk();

    HttpResponse::Ok().body("✅ Block mined and saved.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin Web Node Initializing...");

    // Wallet logic
    let wallet_path_pub = "wallet_public.key";
    let wallet_path_priv = "wallet_private.key";

    let wallet = if Path::new(wallet_path_pub).exists() {
        Wallet::load_from_files(wallet_path_pub, wallet_path_priv)
            .expect("⚠️ Failed to load wallet from files")
    } else {
        println!("📄 No wallet found — generating new one.");
        let wallet = Wallet::new();
        wallet.save_to_files(wallet_path_pub, wallet_path_priv);
        println!("✅ Wallet saved to disk.");
        wallet
    };

    let address = wallet.get_address();
    println!("🔐 Wallet Address: {}", address);

    // Blockchain logic
    let start_time = Instant::now();
    let blockchain = Blockchain::load_from_disk().unwrap_or_else(|| {
        println!("📦 No blockchain found — creating new one.");
        Blockchain::new(address.clone())
    });
    println!("✅ Blockchain ready. Load time: {:?}", start_time.elapsed());

    // Shared app state
    let app_data = web::Data::new(AppState {
        wallet: Mutex::new(wallet),
        blockchain: Mutex::new(blockchain),
    });

    // Web server
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/address", web::get().to(get_address))
            .route("/balance", web::get().to(get_balance))
            .route("/mine", web::post().to(mine))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}