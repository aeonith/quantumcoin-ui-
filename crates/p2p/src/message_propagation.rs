//! Message propagation management

use crate::GossipMessage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Default)]
pub struct PropagationStats {
    pub total_messages: u64,
    pub successful_propagations: u64,
    pub failed_propagations: u64,
    pub avg_propagation_time_ms: f64,
}

pub struct PropagationManager {
    stats: Arc<RwLock<PropagationStats>>,
}

impl PropagationManager {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(PropagationStats::default())),
        }
    }

    pub async fn record_broadcast(&self, message: &GossipMessage, peer_count: usize) {
        let mut stats = self.stats.write().await;
        stats.total_messages += 1;
        stats.successful_propagations += peer_count as u64;
    }
}
