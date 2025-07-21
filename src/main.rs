use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: u64,
    timestamp: u64,
}

// Simulated in-memory mempool
struct AppState {
    mempool: Mutex<Vec<Transaction>>,
}

fn get_wallet_address() -> String {
    let pub_key_bytes = b"MyQuantumCoinPublicKey1234567890";
    general_purpose::STANDARD.encode(pub_key_bytes)
}

async fn wallet_handler() -> impl Responder {
    let wallet_address = get_wallet_address();
    HttpResponse::Ok().body(format!("QuantumCoin Wallet Address:\n{}", wallet_address))
}

async fn create_transaction(
    transaction: web::Json<Transaction>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut mempool = data.mempool.lock().unwrap();
    let mut tx = transaction.into_inner();
    tx.timestamp = chrono::Utc::now().timestamp() as u64;
    mempool.push(tx.clone());

    HttpResponse::Ok().json(&tx)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin Web Server Running");

    let app_state = web::Data::new(AppState {
        mempool: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/wallet", web::get().to(wallet_handler))
            .route("/transaction", web::post().to(create_transaction))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}