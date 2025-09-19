pub mod analytics;
pub mod attack_detection;
pub mod network_optimizer;
pub mod performance_tuner;

pub use analytics::*;
pub use attack_detection::*;
pub use network_optimizer::*;
pub use performance_tuner::*;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockData {
    pub height: u64,
    pub timestamp: DateTime<Utc>,
    pub hash: String,
    pub difficulty: f64,
    pub tx_count: u32,
    pub size_bytes: u64,
    pub propagation_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkMetrics {
    pub peer_count: u32,
    pub mempool_size: u32,
    pub avg_block_time: f64,
    pub hashrate_estimate: f64,
    pub orphan_rate: f64,
    pub fee_percentiles: Vec<f64>,
}
