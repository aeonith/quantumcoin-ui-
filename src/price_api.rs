// src/price_api.rs
use axum::{
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use crate::price::{calculate_live_price, load_price_history};

pub async fn get_price() -> Json<serde_json::Value> {
    let price = calculate_live_price(21_000_000, 4000, 900); // example numbers
    Json(json!({ "price": price }))
}

pub async fn get_price_history() -> Json<serde_json::Value> {
    let history = load_price_history();
    Json(json!(history))
}

pub fn price_routes() -> Router {
    Router::new()
        .route("/api/price", get(get_price))
        .route("/api/price/history", get(get_price_history))
}