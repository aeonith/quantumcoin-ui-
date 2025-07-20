use warp::Filter;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TransactionRequest {
    sender: String,
    recipient: String,
    amount: f64,
}

#[derive(Serialize)]
struct TransactionResponse {
    message: String,
}

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["POST", "GET"]);

    let send_route = warp::path!("api" / "send")
        .and(warp::post())
        .and(warp::body::json())
        .map(|tx: TransactionRequest| {
            println!("Transaction from {} to {} of {}", tx.sender, tx.recipient, tx.amount);
            warp::reply::json(&TransactionResponse {
                message: "Transaction processed".to_string(),
            })
        });

    let balance_route = warp::path!("api" / "balance")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({ "balance": 1250000.0 })));

    let routes = send_route.or(balance_route).with(cors);

    println!("✅ QuantumCoin backend running at /api/*");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}