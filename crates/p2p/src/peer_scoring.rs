//! Peer scoring system

use crate::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum ScoreReason {
    ValidMessage,
    InvalidMessage,
    SlowResponse,
    FastResponse,
}

#[derive(Debug, Clone)]
pub enum PeerBehavior {
    ValidMessage,
    InvalidMessage,
    Disconnect,
    Timeout,
}

pub struct PeerScorer {
    scores: RwLock<HashMap<SocketAddr, i32>>,
}

impl PeerScorer {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_peer(&self, addr: SocketAddr) {
        self.scores.write().await.insert(addr, 100); // Start with neutral score
    }

    pub async fn remove_peer(&self, addr: SocketAddr) {
        self.scores.write().await.remove(&addr);
    }

    pub async fn record_good_behavior(&self, addr: SocketAddr, _behavior: PeerBehavior) {
        if let Some(score) = self.scores.write().await.get_mut(&addr) {
            *score = (*score + 1).min(1000);
        }
    }

    pub async fn get_peer_score(&self, addr: SocketAddr) -> Option<i32> {
        self.scores.read().await.get(&addr).copied()
    }

    pub async fn get_lowest_scoring_peer(&self) -> Option<(SocketAddr, i32)> {
        self.scores.read().await.iter()
            .min_by_key(|(_, &score)| score)
            .map(|(&addr, &score)| (addr, score))
    }
}
