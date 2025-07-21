use axum::{
    routing::{get, post},
    Json, Router, extract::State,
};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};

mod blockchain;
mod wallet;
mod transaction;
mod revstop;
mod mempool;

use blockchain::Blockchain;
use wallet::Wallet;
use transaction::Transaction;
use mempool::Mempool;
use revstop::RevStop;

#[derive(Clone)]
struct AppState {
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    wallet: Arc<Mutex<Wallet>>,
    revstop: Arc<Mutex<RevStop>>,
}

#[derive(Deserialize)]
struct SendRequest {
    recipient: String,
    amount: f64,
}

#[tokio::main]
async fn main() {
    let wallet = Wallet::load_from_files().unwrap_or_else(|_| Wallet::new());
    let revstop = RevStop::load_status().unwrap_or_else(|_| RevStop::default());
    let mut blockchain = Blockchain::new();
    blockchain.load_or_create_genesis(wallet.get_address(), 1_250_000.0);

    let state = AppState {
        blockchain: Arc::new(Mutex::new(blockchain)),
        mempool: Arc::new(Mutex::new(Mempool::new())),
        wallet: Arc::new(Mutex::new(wallet)),
        revstop: Arc::new(Mutex::new(revstop)),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/price", get(get_price))
        .route("/send", post(send_transaction))
        .route("/mine", post(mine_block))
        .with_state(state.clone())
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✅ QuantumCoin API running at http://{}/", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "✅ QuantumCoin Node is Alive"
}

async fn get_price(State(state): State<AppState>) -> Json<PriceResponse> {
    let chain = state.blockchain.lock().unwrap();
    let price = chain.calculate_price();
    Json(PriceResponse {
        current_price: price,
    })
}

#[derive(Serialize)]
struct PriceResponse {
    current_price: f64,
}

async fn send_transaction(
    State(state): State<AppState>,
    Json(payload): Json<SendRequest>,
) -> Json<String> {
    let wallet = state.wallet.lock().unwrap();
    let tx = wallet.create_transaction(&payload.recipient, payload.amount);
    let mut mempool = state.mempool.lock().unwrap();
    mempool.add_transaction(tx);
    Json("Transaction submitted.".to_string())
}

async fn mine_block(State(state): State<AppState>) -> Json<String> {
    let mut blockchain = state.blockchain.lock().unwrap();
    let mut mempool = state.mempool.lock().unwrap();
    let transactions = mempool.drain();
    blockchain.mine_pending_transactions(transactions);
    blockchain.save_to_file().unwrap();
    Json("✅ Block mined.".to_string())
}