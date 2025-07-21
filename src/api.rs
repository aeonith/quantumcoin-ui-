use axum::{
    routing::{get, post},
    Router, Json, extract::State
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::{blockchain::Blockchain, wallet::Wallet, transaction::Transaction};
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub wallet: Arc<Mutex<Wallet>>,
}

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct CreateWalletRequest {}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub balance: f64,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub message: String,
}

pub async fn send_transaction(
    State(state): State<AppState>,
    Json(payload): Json<SendRequest>,
) -> Json<StatusResponse> {
    let mut wallet = state.wallet.lock().unwrap();
    let blockchain = state.blockchain.lock().unwrap();

    match wallet.create_transaction(&payload.to, payload.amount) {
        Ok(tx) => {
            drop(wallet); // unlock
            drop(blockchain); // unlock
            let mut chain = state.blockchain.lock().unwrap();
            chain.add_transaction(tx);
            Json(StatusResponse {
                message: "Transaction added.".to_string(),
            })
        }
        Err(e) => Json(StatusResponse {
            message: format!("Error: {}", e),
        }),
    }
}

pub async fn get_balance(State(state): State<AppState>) -> Json<BalanceResponse> {
    let wallet = state.wallet.lock().unwrap();
    Json(BalanceResponse {
        balance: wallet.balance,
    })
}

pub async fn mine(State(state): State<AppState>) -> Json<StatusResponse> {
    let mut blockchain = state.blockchain.lock().unwrap();
    let wallet = state.wallet.lock().unwrap();
    blockchain.mine_pending_transactions(&wallet);
    Json(StatusResponse {
        message: "Block mined.".to_string(),
    })
}

pub async fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/balance", get(get_balance))
        .route("/api/send", post(send_transaction))
        .route("/api/mine", post(mine))
        .with_state(state)
}

// To launch this API server:
// let state = AppState { blockchain: Arc::new(Mutex::new(...)), wallet: Arc::new(Mutex::new(...)) };
// axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
//     .serve(routes(state).await.into_make_service())
//     .await.unwrap();