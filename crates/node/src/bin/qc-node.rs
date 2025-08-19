use axum::{routing::post, Json, Router};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::time::sleep;

use qc_node::Chain;

#[tokio::main]
async fn main() {
    let chain = Arc::new(Chain::new_genesis());

    // background miner
    let c2 = chain.clone();
    tokio::spawn(async move {
        loop {
            c2.mine_one();
            sleep(Duration::from_secs(5)).await; // mine every ~5s for demo; adjust later
        }
    });

    // JSON-RPC
    let app = Router::new().route("/", post({
        let chain = chain.clone();
        move |Json(req): Json<Value>| async move {
            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let id = req.get("id").cloned().unwrap_or(json!(1));
            let res = match method {
                "qc_blockNumber" => json!({"jsonrpc":"2.0","id":id,"result": format!("0x{:x}", chain.height())}),
                "qc_peerCount"   => json!({"jsonrpc":"2.0","id":id,"result": format!("0x{:x}", chain.peers())}),
                "qc_getBlockByNumber" => {
                    let n_hex = req["params"].get(0).and_then(|v| v.as_str()).unwrap_or("0x0");
                    let n = u64::from_str_radix(n_hex.trim_start_matches("0x"),16).unwrap_or(0);
                    match chain.get_block_by_number(n) {
                        Some(b) => json!({"jsonrpc":"2.0","id":id,"result": b}),
                        None => json!({"jsonrpc":"2.0","id":id,"result": Value::Null})
                    }
                }
                _ => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not found"}})
            };
            Json(res)
        }
    }));

    let addr: SocketAddr = "0.0.0.0:8545".parse().unwrap();
    println!("qc-node JSON-RPC at {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
