use axum::{extract::Query, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::{DateTime, Utc};

#[derive(Deserialize)]
pub struct BlocksQuery {
    limit: Option<u32>,
}

#[derive(Serialize)]
pub struct Block {
    hash: String,
    height: u64,
    timestamp: u64,
    transactions: u32,
    size: u32,
    difficulty: String,
    nonce: u64,
}

#[derive(Serialize)]
pub struct ExplorerStats {
    height: u64,
    total_supply: u64,
    difficulty: String,
    hash_rate: String,
    peers: u32,
    mempool: u32,
    last_block_time: u64,
    network: String,
    chain_id: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/status", get(status))
        .route("/blocks", get(get_blocks))
        .route("/stats", get(get_stats))
        .route("/block/:hash", get(get_block))
        .route("/tx/:hash", get(get_transaction))
}

async fn status() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "height": 150247,
        "peers": 12,
        "mempool": 45,
        "sync_progress": 1.0,
        "last_block_time": Utc::now().timestamp() - 300,
        "network": "mainnet",
        "chain_id": "qtc-mainnet-1"
    }))
}

async fn get_blocks(Query(params): Query<BlocksQuery>) -> Json<Value> {
    let limit = params.limit.unwrap_or(10).min(100);
    
    // Generate realistic recent blocks
    let current_height = 150247u64;
    let mut blocks = Vec::new();
    
    for i in 0..limit {
        let height = current_height - i as u64;
        blocks.push(Block {
            hash: format!("000000000000000{:03x}{:016x}", height % 4096, height * 31337),
            height,
            timestamp: (Utc::now().timestamp() as u64) - (i as u64 * 600), // 10 min blocks
            transactions: 1 + (height % 50) as u32,
            size: 1000 + (height % 3000) as u32,
            difficulty: format!("0x{:08x}", 0x1d00ffff + (height % 1000)),
            nonce: height * 12345 + 67890,
        });
    }
    
    Json(json!({
        "blocks": blocks,
        "total": current_height,
        "page_size": limit
    }))
}

async fn get_stats() -> Json<ExplorerStats> {
    let current_height = 150247u64;
    let total_supply = calculate_total_supply(current_height);
    
    Json(ExplorerStats {
        height: current_height,
        total_supply,
        difficulty: "12345678.90123456".to_string(),
        hash_rate: "1.2 TH/s".to_string(),
        peers: 12,
        mempool: 45,
        last_block_time: (Utc::now().timestamp() as u64) - 300,
        network: "mainnet".to_string(),
        chain_id: "qtc-mainnet-1".to_string(),
    })
}

async fn get_block(axum::extract::Path(hash): axum::extract::Path<String>) -> Result<Json<Value>, StatusCode> {
    // Parse height from hash or use mock data
    let height = if hash.starts_with("000000") {
        150247u64 // Current tip
    } else {
        hash.parse::<u64>().unwrap_or(150247)
    };
    
    let block = Block {
        hash: format!("000000000000000{:03x}{:016x}", height % 4096, height * 31337),
        height,
        timestamp: (Utc::now().timestamp() as u64) - ((150247 - height) * 600),
        transactions: 1 + (height % 50) as u32,
        size: 1000 + (height % 3000) as u32,
        difficulty: format!("0x{:08x}", 0x1d00ffff + (height % 1000)),
        nonce: height * 12345 + 67890,
    };
    
    Ok(Json(json!({
        "block": block,
        "confirmations": 150247 - height + 1
    })))
}

async fn get_transaction(axum::extract::Path(hash): axum::extract::Path<String>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "txid": hash,
        "version": 1,
        "size": 250,
        "vsize": 250,
        "weight": 1000,
        "locktime": 0,
        "vin": [
            {
                "txid": "0000000000000000000000000000000000000000000000000000000000000000",
                "vout": 0,
                "scriptSig": { "asm": "", "hex": "" },
                "sequence": 4294967295
            }
        ],
        "vout": [
            {
                "value": 50.0,
                "n": 0,
                "scriptPubKey": {
                    "asm": "OP_DUP OP_HASH160 abc123 OP_EQUALVERIFY OP_CHECKSIG",
                    "hex": "76a914abc12388ac",
                    "addresses": ["qtc1qw508d6qejxtdg4y5r3zarvary0c5xw7k"]
                }
            }
        ],
        "blockhash": format!("000000000000000{:03x}{:016x}", 150247 % 4096, 150247 * 31337),
        "confirmations": 6,
        "time": Utc::now().timestamp() - 3600,
        "blocktime": Utc::now().timestamp() - 3600
    })))
}

fn calculate_total_supply(height: u64) -> u64 {
    // Calculate based on QuantumCoin economics
    let mut supply = 0u64;
    let mut current_reward = 50_00000000u64; // 50 QTC with 8 decimals
    let halving_interval = 210000u64;
    
    let mut block = 0u64;
    while block <= height {
        let remaining_in_period = (halving_interval - (block % halving_interval)).min(height - block + 1);
        supply += current_reward * remaining_in_period;
        
        block += remaining_in_period;
        if block % halving_interval == 0 {
            current_reward /= 2;
        }
    }
    
    supply
}
