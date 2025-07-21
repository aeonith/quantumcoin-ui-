use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{wallet::Wallet, blockchain::Blockchain, revstop::RevStop};

#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub revstop: Arc<Mutex<RevStop>>,
    pub wallet: Arc<Mutex<Wallet>>,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    address: String,
    balance: f64,
}

#[derive(Deserialize)]
pub struct SendRequest {
    recipient: String,
    amount: f64,
}

#[derive(Serialize)]
pub struct ExplorerResponse {
    height: usize,
    total_transactions: usize,
    last_block_hash: String,
}

pub async fn create_wallet() -> Json<Wallet> {
    let wallet = Wallet::generate();
    Json(wallet)
}

pub async fn get_balance(state: Arc<Mutex<Wallet>>) -> Json<BalanceResponse> {
    let wallet = state.lock().await;
    Json(BalanceResponse {
        address: wallet.get_address(),
        balance: wallet.balance,
    })
}

pub async fn send_coins(
    state: Arc<AppState>,
    Json(payload): Json<SendRequest>,
) -> Json<String> {
    let mut wallet = state.wallet.lock().await;
    let mut blockchain = state.blockchain.lock().await;

    match wallet.create_transaction(&payload.recipient, payload.amount) {
        Some(tx) => {
            blockchain.add_transaction(tx);
            blockchain.save_to_file("blockchain.json").ok();
            Json("Transaction sent and added to mempool.".to_string())
        }
        None => Json("Transaction failed: Insufficient funds or RevStop active.".to_string()),
    }
}

pub async fn mine_block(state: Arc<AppState>) -> Json<String> {
    let mut blockchain = state.blockchain.lock().await;
    let wallet = state.wallet.lock().await;
    let address = wallet.get_address();
    blockchain.mine_pending_transactions(&address);
    blockchain.save_to_file("blockchain.json").ok();
    Json("Block mined and added to blockchain.".to_string())
}

pub async fn toggle_revstop(state: Arc<AppState>) -> Json<String> {
    let mut revstop = state.revstop.lock().await;
    if revstop.is_locked() {
        Json("RevStop is currently locked.".to_string())
    } else {
        revstop.lock("frontend-toggle".to_string());
        Json("RevStop locked.".to_string())
    }
}

pub async fn explorer(state: Arc<AppState>) -> Json<ExplorerResponse> {
    let blockchain = state.blockchain.lock().await;
    Json(ExplorerResponse {
        height: blockchain.chain.len(),
        total_transactions: blockchain.total_tx_count(),
        last_block_hash: blockchain.last_hash(),
    })
}

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/wallet/new", get(create_wallet))
        .route("/wallet/balance", get({
            let wallet = state.wallet.clone();
            move || get_balance(wallet)
        }))
        .route("/wallet/send", post({
            let state = state.clone();
            move |req| send_coins(state.clone(), req)
        }))
        .route("/mine", post({
            let state = state.clone();
            move || mine_block(state.clone())
        }))
        .route("/revstop/toggle", post({
            let state = state.clone();
            move || toggle_revstop(state.clone())
        }))
        .route("/explorer", get({
            let state = state.clone();
            move || explorer(state.clone())
        }))
}