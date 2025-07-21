use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::transaction::Transaction;

#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub wallet: Arc<Mutex<Wallet>>,
}

// Used for POST /send
#[derive(Deserialize)]
pub struct SendRequest {
    pub recipient: String,
    pub amount: f64,
}

// Used for GET /balance
#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: f64,
}

// Used for GET /price
#[derive(Serialize)]
pub struct PriceResponse {
    pub current_price: f64,
}

// Create the router with all endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/balance", get(get_balance))
        .route("/send", post(send_coins))
        .route("/mine", post(mine))
        .route("/price", get(get_price))
        .with_state(state)
}

// GET /balance?address=xyz
async fn get_balance(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<BalanceResponse> {
    let addr = params.get("address").cloned().unwrap_or_default();
    let blockchain = state.blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&addr);
    Json(BalanceResponse {
        address: addr,
        balance,
    })
}

// POST /send { recipient, amount }
async fn send_coins(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<SendRequest>,
) -> Json<&'static str> {
    let wallet = state.wallet.lock().unwrap();
    let tx = wallet.create_transaction(&payload.recipient, payload.amount);
    drop(wallet);

    let mut blockchain = state.blockchain.lock().unwrap();
    blockchain.add_transaction(tx);
    Json("Transaction submitted")
}

// POST /mine
async fn mine(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<&'static str> {
    let mut blockchain = state.blockchain.lock().unwrap();
    blockchain.mine_pending_transactions();
    Json("Block mined successfully")
}

// GET /price
async fn get_price(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<PriceResponse> {
    let blockchain = state.blockchain.lock().unwrap();
    let price = blockchain.current_price;
    Json(PriceResponse {
        current_price: price,
    })
}