use axum::{
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};

// Example shared state for blockchain
#[derive(Clone)]
pub struct AppState {
    pub total_supply: Arc<Mutex<u64>>,
    pub mined_coins: Arc<Mutex<u64>>,
}

#[derive(Serialize)]
struct SupplyResponse {
    total_supply: u64,
    mined_coins: u64,
    circulating_supply: u64,
}

#[derive(Serialize)]
struct PriceResponse {
    current_price: f64,
}

// GET /supply
async fn get_supply(state: Arc<AppState>) -> Json<SupplyResponse> {
    let total = *state.total_supply.lock().unwrap();
    let mined = *state.mined_coins.lock().unwrap();
    let circulating = mined; // assuming all mined coins are circulating
    Json(SupplyResponse {
        total_supply: total,
        mined_coins: mined,
        circulating_supply: circulating,
    })
}

// GET /price
async fn get_price(state: Arc<AppState>) -> Json<PriceResponse> {
    let mined = *state.mined_coins.lock().unwrap();
    let remaining = 21_000_000 - mined;
    let base_price = 0.25;
    let demand_factor = 1.0 + (mined as f64 / 1_000_000.0);
    let price = (base_price * demand_factor).max(0.25); // never below $0.25
    Json(PriceResponse {
        current_price: price,
    })
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/supply", get({
            let state = Arc::clone(&state);
            move || get_supply(state.clone())
        }))
        .route("/price", get({
            let state = Arc::clone(&state);
            move || get_price(state.clone())
        }))
}