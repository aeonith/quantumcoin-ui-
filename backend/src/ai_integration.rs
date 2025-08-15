use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::{State, get, post};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{Value, json};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SentinelOutput {
    pub timestamp: DateTime<Utc>,
    pub risk_level: f64,
    pub confirmation_multiplier: f64,
    pub min_relay_fee: f64,
    pub relay_priority_boost: f64,
    pub reorg_protection_depth: u32,
    pub attack_probability: HashMap<String, f64>,
    pub performance_optimizations: HashMap<String, f64>,
}

#[derive(Debug, Default)]
pub struct AIState {
    pub current_optimizations: Option<SentinelOutput>,
    pub risk_level: f64,
    pub performance_boost: f64,
    pub security_level: f64,
}

#[post("/sentinel/update", data = "<sentinel_data>")]
pub fn update_ai_optimizations(
    sentinel_data: Json<SentinelOutput>,
    ai_state: &State<Arc<RwLock<AIState>>>
) -> Json<Value> {
    let sentinel = sentinel_data.into_inner();
    
    // Apply AI optimizations in real-time
    let risk_level = sentinel.risk_level;
    let performance_boost = sentinel.performance_optimizations
        .values()
        .fold(1.0, |acc, &val| acc * val);
    
    // Update AI state
    let ai_state_clone = ai_state.clone();
    tokio::spawn(async move {
        let mut ai = ai_state_clone.write().await;
        ai.current_optimizations = Some(sentinel);
        ai.risk_level = risk_level;
        ai.performance_boost = performance_boost;
        ai.security_level = 1.0 - risk_level;
    });

    tracing::info!("🤖 AI optimizations applied - Performance: {:.2}x, Security: {:.1}%", 
        performance_boost, (1.0 - risk_level) * 100.0);

    Json(json!({
        "status": "AI optimizations applied",
        "performance_boost": performance_boost,
        "security_level": 1.0 - risk_level,
        "timestamp": Utc::now()
    }))
}

#[get("/ai/status")]
pub fn get_ai_status(ai_state: &State<Arc<RwLock<AIState>>>) -> Json<Value> {
    let ai = futures::executor::block_on(ai_state.read());
    
    Json(json!({
        "ai_active": ai.current_optimizations.is_some(),
        "risk_level": ai.risk_level,
        "performance_boost": ai.performance_boost,
        "security_level": ai.security_level,
        "last_update": ai.current_optimizations.as_ref()
            .map(|opt| opt.timestamp)
            .unwrap_or_else(Utc::now),
        "optimizations_active": ai.current_optimizations.as_ref()
            .map(|opt| opt.performance_optimizations.len())
            .unwrap_or(0)
    }))
}

#[get("/network/metrics")]
pub fn get_network_metrics(
    blockchain_state: &State<Arc<RwLock<crate::blockchain::Blockchain>>>,
    ai_state: &State<Arc<RwLock<AIState>>>
) -> Json<Value> {
    let blockchain = futures::executor::block_on(blockchain_state.read());
    let ai = futures::executor::block_on(ai_state.read());
    
    // Calculate real network metrics
    let chain_len = blockchain.chain.len() as u64;
    let mempool_size = blockchain.pending_transactions.len() as u32;
    
    // Calculate average block time from recent blocks
    let avg_block_time = if blockchain.chain.len() >= 10 {
        let recent_blocks = &blockchain.chain[blockchain.chain.len()-10..];
        let time_diffs: Vec<f64> = recent_blocks.windows(2)
            .map(|w| (w[1].timestamp - w[0].timestamp).num_seconds() as f64)
            .collect();
        time_diffs.iter().sum::<f64>() / time_diffs.len() as f64
    } else {
        15.0 // Default target
    };

    // Estimate hashrate from difficulty
    let current_difficulty = blockchain.difficulty as f64;
    let hashrate_estimate = current_difficulty * 1000.0; // Simplified calculation

    Json(json!({
        "height": chain_len,
        "peer_count": 8, // Placeholder - would come from P2P layer
        "mempool_size": mempool_size,
        "avg_block_time": avg_block_time,
        "hashrate": hashrate_estimate,
        "difficulty": current_difficulty,
        "orphan_rate": 0.02, // 2% typical
        "fee_percentiles": [1.0, 2.0, 5.0, 10.0, 20.0],
        "ai_performance_boost": ai.performance_boost,
        "ai_security_level": ai.security_level,
        "network_efficiency": ai.performance_boost * ai.security_level
    }))
}

#[get("/block/latest")]
pub fn get_latest_block(
    blockchain_state: &State<Arc<RwLock<crate::blockchain::Blockchain>>>
) -> Json<Value> {
    let blockchain = futures::executor::block_on(blockchain_state.read());
    
    if let Some(latest_block) = blockchain.chain.last() {
        Json(json!({
            "height": latest_block.index,
            "hash": latest_block.hash,
            "timestamp": latest_block.timestamp,
            "difficulty": blockchain.difficulty,
            "tx_count": latest_block.transactions.len(),
            "size": serde_json::to_string(&latest_block).unwrap_or_default().len(),
            "transactions": latest_block.transactions,
            "merkle_root": latest_block.merkle_root,
            "nonce": latest_block.nonce,
            "previous_hash": latest_block.previous_hash
        }))
    } else {
        Json(json!({
            "error": "No blocks found"
        }))
    }
}
