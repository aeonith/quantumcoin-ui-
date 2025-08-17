//! Network health monitoring

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub peer_count: usize,
    pub message_rate: u64,
    pub network_score: f64,
    pub is_healthy: bool,
}

pub struct NetworkHealth {
    metrics: RwLock<HealthMetrics>,
}

impl NetworkHealth {
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HealthMetrics {
                peer_count: 0,
                message_rate: 0,
                network_score: 1.0,
                is_healthy: true,
            }),
        }
    }

    pub async fn get_current_metrics(&self) -> HealthMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn update_metrics(&self, peer_count: usize, message_rate: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.peer_count = peer_count;
        metrics.message_rate = message_rate;
        metrics.is_healthy = peer_count > 10 && message_rate < 10000;
        metrics.network_score = if metrics.is_healthy { 1.0 } else { 0.5 };
    }

    pub async fn detect_partition(&self) -> bool {
        let metrics = self.metrics.read().await;
        metrics.peer_count < 5
    }
}

pub struct PartitionDetector;

impl PartitionDetector {
    pub fn new() -> Self {
        Self
    }
}
