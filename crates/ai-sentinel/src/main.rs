use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

mod analytics;
mod attack_detection;
mod network_optimizer;
mod performance_tuner;

use analytics::*;
use attack_detection::*;
use network_optimizer::*;
use performance_tuner::*;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct SentinelOutput {
    pub timestamp: DateTime<Utc>,
    pub risk_level: f64,           // 0.0 = safe, 1.0 = high risk
    pub confirmation_multiplier: f64,  // multiply standard confirmations
    pub min_relay_fee: f64,        // satoshis per byte
    pub relay_priority_boost: f64, // peer connection priority adjustment
    pub reorg_protection_depth: u32, // blocks to protect against reorg
    pub attack_probability: HashMap<String, f64>, // attack type -> probability
    pub performance_optimizations: HashMap<String, f64>,
}

pub struct AISentinel {
    analytics: BlockchainAnalytics,
    attack_detector: AttackDetector,
    network_optimizer: NetworkOptimizer,
    performance_tuner: PerformanceTuner,
    node_rpc_url: String,
    db_pool: sqlx::PgPool,
    redis_client: redis::Client,
}

impl AISentinel {
    pub async fn new(
        node_rpc_url: String,
        database_url: String,
        redis_url: String
    ) -> Result<Self> {
        let db_pool = sqlx::PgPool::connect(&database_url).await?;
        let redis_client = redis::Client::open(redis_url)?;
        
        // Initialize AI subsystems
        let analytics = BlockchainAnalytics::new(&db_pool).await?;
        let attack_detector = AttackDetector::new();
        let network_optimizer = NetworkOptimizer::new();
        let performance_tuner = PerformanceTuner::new();

        Ok(Self {
            analytics,
            attack_detector,
            network_optimizer,
            performance_tuner,
            node_rpc_url,
            db_pool,
            redis_client,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("🤖 AI Sentinel starting - enhancing QuantumCoin blockchain");
        
        let mut block_monitor = interval(Duration::from_secs(1));  // Monitor every second
        let mut optimization_cycle = interval(Duration::from_secs(30)); // Optimize every 30s
        let mut ml_training = interval(Duration::from_secs(300));  // Retrain every 5 minutes
        
        loop {
            tokio::select! {
                _ = block_monitor.tick() => {
                    if let Err(e) = self.monitor_blockchain().await {
                        error!("Block monitoring error: {}", e);
                    }
                }
                _ = optimization_cycle.tick() => {
                    if let Err(e) = self.optimize_network().await {
                        error!("Network optimization error: {}", e);
                    }
                }
                _ = ml_training.tick() => {
                    if let Err(e) = self.train_models().await {
                        error!("ML training error: {}", e);
                    }
                }
            }
        }
    }

    async fn monitor_blockchain(&mut self) -> Result<()> {
        // Fetch latest block data from node
        let block_data = self.fetch_latest_block_data().await?;
        let network_metrics = self.fetch_network_metrics().await?;

        // Store in analytics database
        self.analytics.store_block_data(&block_data).await?;
        self.analytics.store_network_metrics(&network_metrics).await?;

        // Real-time attack detection
        let attack_analysis = self.attack_detector.analyze_block(&block_data, &network_metrics).await?;
        
        if attack_analysis.risk_level > 0.7 {
            warn!("🚨 High attack risk detected: {:.2}%", attack_analysis.risk_level * 100.0);
            self.trigger_defensive_measures(&attack_analysis).await?;
        }

        Ok(())
    }

    async fn optimize_network(&mut self) -> Result<()> {
        let recent_data = self.analytics.get_recent_data(1000).await?;
        
        // AI-driven network optimizations
        let optimizations = self.network_optimizer.compute_optimizations(&recent_data).await?;
        let performance_tuning = self.performance_tuner.analyze_performance(&recent_data).await?;

        // Apply optimizations to network
        self.apply_optimizations(&optimizations, &performance_tuning).await?;

        info!("🔧 Network optimizations applied - system performance improved");
        Ok(())
    }

    async fn train_models(&mut self) -> Result<()> {
        info!("🧠 Training AI models with latest blockchain data...");
        
        let training_data = self.analytics.get_training_data(10000).await?;
        
        // Train attack detection models
        self.attack_detector.train_models(&training_data).await?;
        
        // Train network optimization models  
        self.network_optimizer.update_models(&training_data).await?;
        
        // Train performance prediction models
        self.performance_tuner.train_predictors(&training_data).await?;

        info!("✅ AI models updated - system intelligence enhanced");
        Ok(())
    }

    async fn fetch_latest_block_data(&self) -> Result<BlockData> {
        let response = reqwest::get(&format!("{}/status", self.node_rpc_url)).await?;
        let status: serde_json::Value = response.json().await?;
        
        let height = status["height"].as_u64().unwrap_or(0);
        let latest_block_response = reqwest::get(&format!("{}/block/latest", self.node_rpc_url)).await?;
        let block: serde_json::Value = latest_block_response.json().await?;

        Ok(BlockData {
            height,
            timestamp: Utc::now(), // In real implementation, parse from block
            hash: block["hash"].as_str().unwrap_or("").to_string(),
            difficulty: status["difficulty"].as_f64().unwrap_or(0.0),
            tx_count: block["transactions"].as_array().map(|v| v.len() as u32).unwrap_or(0),
            size_bytes: block["size"].as_u64().unwrap_or(0),
            propagation_time_ms: None, // Measured by peer network
        })
    }

    async fn fetch_network_metrics(&self) -> Result<NetworkMetrics> {
        let response = reqwest::get(&format!("{}/network/metrics", self.node_rpc_url)).await?;
        let metrics: serde_json::Value = response.json().await?;

        Ok(NetworkMetrics {
            peer_count: metrics["peer_count"].as_u64().unwrap_or(0) as u32,
            mempool_size: metrics["mempool_size"].as_u64().unwrap_or(0) as u32,
            avg_block_time: metrics["avg_block_time"].as_f64().unwrap_or(15.0),
            hashrate_estimate: metrics["hashrate"].as_f64().unwrap_or(0.0),
            orphan_rate: metrics["orphan_rate"].as_f64().unwrap_or(0.0),
            fee_percentiles: metrics["fee_percentiles"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
                .unwrap_or_default(),
        })
    }

    async fn trigger_defensive_measures(&self, analysis: &AttackAnalysis) -> Result<()> {
        let output = SentinelOutput {
            timestamp: Utc::now(),
            risk_level: analysis.risk_level,
            confirmation_multiplier: if analysis.risk_level > 0.8 { 3.0 } else { 1.5 },
            min_relay_fee: analysis.recommended_fee_floor,
            relay_priority_boost: 1.0 - analysis.risk_level,
            reorg_protection_depth: if analysis.risk_level > 0.9 { 20 } else { 10 },
            attack_probability: analysis.attack_probabilities.clone(),
            performance_optimizations: HashMap::new(),
        };

        // Send to all network nodes
        self.broadcast_sentinel_output(&output).await?;
        
        warn!("🛡️ Defensive measures activated - risk level: {:.1}%", analysis.risk_level * 100.0);
        Ok(())
    }

    async fn apply_optimizations(
        &self, 
        network_opts: &NetworkOptimizations,
        performance_opts: &PerformanceTuning
    ) -> Result<()> {
        let output = SentinelOutput {
            timestamp: Utc::now(),
            risk_level: 0.0, // No attack detected
            confirmation_multiplier: 1.0,
            min_relay_fee: network_opts.optimal_fee_rate,
            relay_priority_boost: network_opts.peer_priority_adjustment,
            reorg_protection_depth: 6,
            attack_probability: HashMap::new(),
            performance_optimizations: performance_opts.optimizations.clone(),
        };

        self.broadcast_sentinel_output(&output).await?;
        Ok(())
    }

    async fn broadcast_sentinel_output(&self, output: &SentinelOutput) -> Result<()> {
        // Send to Redis for real-time consumption
        let mut conn = self.redis_client.get_connection()?;
        let output_json = serde_json::to_string(output)?;
        redis::cmd("PUBLISH")
            .arg("quantumcoin:sentinel")
            .arg(&output_json)
            .execute(&mut conn);

        // Send directly to node via RPC
        let client = reqwest::Client::new();
        client.post(&format!("{}/sentinel/update", self.node_rpc_url))
            .json(output)
            .send()
            .await?;

        info!("📡 AI optimizations broadcasted to network");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();

    let node_rpc_url = std::env::var("NODE_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/quantumcoin".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let mut sentinel = AISentinel::new(node_rpc_url, database_url, redis_url).await?;
    
    info!("🤖 QuantumCoin AI Sentinel initialized - beginning blockchain enhancement");
    sentinel.run().await?;

    Ok(())
}
