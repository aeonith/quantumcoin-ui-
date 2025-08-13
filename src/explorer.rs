use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use warp::Filter;
use serde::{Serialize};

#[derive(Serialize)]
struct BlockInfo {
    index: usize,
    timestamp: u128,
    hash: String,
    previous_hash: String,
    transactions: Vec<String>,
}

#[derive(Serialize)]
struct ExplorerStats {
    total_blocks: usize,
    total_supply: f64,
    halving_stage: usize,
}

pub async fn start_explorer(blockchain: Arc<Mutex<Blockchain>>) {
    // Route: /blocks
    let blocks_route = {
        let blockchain = blockchain.clone();
        warp::path("blocks")
            .and(warp::get())
            .and_then(move || {
                let blockchain = blockchain.clone();
                async move {
                    let chain = blockchain.lock().unwrap();
                    let blocks_info: Vec<BlockInfo> = chain.chain.iter().map(|block| {
                        BlockInfo {
                            index: block.index,
                            timestamp: block.timestamp,
                            hash: block.hash.clone(),
                            previous_hash: block.previous_hash.clone(),
                            transactions: block.transactions.iter().map(|tx| format!("{} -> {} ({})", tx.sender, tx.recipient, tx.amount)).collect(),
                        }
                    }).collect();
                    Ok::<_, warp::Rejection>(warp::reply::json(&blocks_info))
                }
            })
    };

    // Route: /stats
    let stats_route = {
        let blockchain = blockchain.clone();
        warp::path("stats")
            .and(warp::get())
            .and_then(move || {
                let blockchain = blockchain.clone();
                async move {
                    let chain = blockchain.lock().unwrap();
                    let stats = ExplorerStats {
                        total_blocks: chain.chain.len(),
                        total_supply: chain.get_total_supply(),
                        halving_stage: chain.get_halving_stage(),
                    };
                    Ok::<_, warp::Rejection>(warp::reply::json(&stats))
                }
            })
    };

    // Route: /balance/{address}
    let balance_route = {
        let blockchain = blockchain.clone();
        warp::path!("balance" / String)
            .and(warp::get())
            .and_then(move |address: String| {
                let blockchain = blockchain.clone();
                async move {
                    let chain = blockchain.lock().unwrap();
                    let balance = chain.get_balance(&address);
                    Ok::<_, warp::Rejection>(warp::reply::json(&balance))
                }
            })
    };

    // Combine all routes
    let routes = blocks_route.or(stats_route).or(balance_route);

    // Start server
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("🌐 Explorer running at http://{}", addr);
    warp::serve(routes).run(addr).await;
}