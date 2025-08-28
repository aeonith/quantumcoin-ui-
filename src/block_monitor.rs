use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, debug, error};
use serde::{Serialize, Deserialize};

use crate::{
    blockchain::Blockchain,
    mempool::Mempool,
    p2p::P2PNode,
    ai_learning::AILearningSystem,
    revstop::RevStop,
    economics::EconomicsEngine,
};

/// Live block height monitor that ensures continuous operation
pub struct BlockMonitor {
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Mempool>>,
    p2p_node: Arc<P2PNode>,
    ai_system: Arc<RwLock<AILearningSystem>>,
    revstop: Arc<RwLock<RevStop>>,
    economics: Arc<RwLock<EconomicsEngine>>,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<MonitorStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStats {
    pub uptime_seconds: u64,
    pub blocks_processed: u64,
    pub transactions_processed: u64,
    pub ai_learning_sessions: u64,
    pub revstop_alerts: u64,
    pub network_events: u64,
    pub last_block_time: Option<chrono::DateTime<chrono::Utc>>,
    pub avg_processing_time: f64,
}

impl Default for MonitorStats {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            blocks_processed: 0,
            transactions_processed: 0,
            ai_learning_sessions: 0,
            revstop_alerts: 0,
            network_events: 0,
            last_block_time: None,
            avg_processing_time: 0.0,
        }
    }
}

impl BlockMonitor {
    pub fn new(
        blockchain: Arc<RwLock<Blockchain>>,
        mempool: Arc<RwLock<Mempool>>,
        p2p_node: Arc<P2PNode>,
        ai_system: Arc<RwLock<AILearningSystem>>,
        revstop: Arc<RwLock<RevStop>>,
        economics: Arc<RwLock<EconomicsEngine>>,
    ) -> Self {
        Self {
            blockchain,
            mempool,
            p2p_node,
            ai_system,
            revstop,
            economics,
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(MonitorStats::default())),
        }
    }
    
    /// Start the live monitoring system
    pub async fn start(&self) -> Result<()> {
        info!("🔍 Starting QuantumCoin Live Block Monitor");
        
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }
        
        // Start monitoring tasks
        self.start_block_height_monitor().await;
        self.start_transaction_monitor().await;
        self.start_ai_learning_monitor().await;
        self.start_revstop_monitor().await;
        self.start_performance_monitor().await;
        self.start_network_monitor().await;
        
        info!("✅ All monitoring systems active");
        
        // Main monitoring loop
        let mut status_interval = interval(Duration::from_secs(10));
        let start_time = std::time::Instant::now();
        
        loop {
            status_interval.tick().await;
            
            let running = *self.is_running.read().await;
            if !running {
                break;
            }
            
            // Update uptime
            {
                let mut stats = self.stats.write().await;
                stats.uptime_seconds = start_time.elapsed().as_secs();
            }
            
            // Log status
            self.log_status().await;
        }
        
        info!("🛑 Block monitor stopped");
        Ok(())
    }
    
    /// Stop the monitoring system
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        info!("🛑 Stopping block monitor");
    }
    
    /// Start block height monitoring
    async fn start_block_height_monitor(&self) {
        let blockchain = Arc::clone(&self.blockchain);
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1)); // Check every second
            let mut last_height = 0u64;
            
            while *is_running.read().await {
                interval.tick().await;
                
                let current_height = {
                    let blockchain_guard = blockchain.read().await;
                    blockchain_guard.chain.len() as u64
                };
                
                if current_height > last_height {
                    info!("📈 NEW BLOCK: Height {} -> {}", last_height, current_height);
                    
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.blocks_processed += current_height - last_height;
                        stats_guard.last_block_time = Some(chrono::Utc::now());
                    }
                    
                    last_height = current_height;
                }
            }
        });
    }
    
    /// Start transaction monitoring
    async fn start_transaction_monitor(&self) {
        let mempool = Arc::clone(&self.mempool);
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            let mut last_tx_count = 0usize;
            
            while *is_running.read().await {
                interval.tick().await;
                
                let current_tx_count = {
                    let mempool_guard = mempool.read().await;
                    mempool_guard.size()
                };
                
                if current_tx_count != last_tx_count {
                    debug!("💰 Mempool update: {} transactions", current_tx_count);
                    
                    {
                        let mut stats_guard = stats.write().await;
                        if current_tx_count > last_tx_count {
                            stats_guard.transactions_processed += current_tx_count - last_tx_count;
                        }
                    }
                    
                    last_tx_count = current_tx_count;
                }
            }
        });
    }
    
    /// Start AI learning monitoring
    async fn start_ai_learning_monitor(&self) {
        let blockchain = Arc::clone(&self.blockchain);
        let mempool = Arc::clone(&self.mempool);
        let ai_system = Arc::clone(&self.ai_system);
        let p2p_node = Arc::clone(&self.p2p_node);
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Learn every 30 seconds
            
            while *is_running.read().await {
                interval.tick().await;
                
                let blockchain_guard = blockchain.read().await;
                let mempool_guard = mempool.read().await;
                let network_stats = p2p_node.get_stats().await;
                
                let mut ai_guard = ai_system.write().await;
                
                // Trigger AI learning session
                if let Err(e) = ai_guard.learn_from_block(&blockchain_guard, &mempool_guard, &network_stats, &[]).await {
                    error!("AI learning error: {}", e);
                } else {
                    debug!("🧠 AI learning session completed");
                    
                    let mut stats_guard = stats.write().await;
                    stats_guard.ai_learning_sessions += 1;
                }
            }
        });
    }
    
    /// Start RevStop monitoring
    async fn start_revstop_monitor(&self) {
        let revstop = Arc::clone(&self.revstop);
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            
            while *is_running.read().await {
                interval.tick().await;
                
                let mut revstop_guard = revstop.write().await;
                
                // Process pending reversals
                match revstop_guard.process_reversals().await {
                    Ok(executed) => {
                        if !executed.is_empty() {
                            info!("🛡️ RevStop executed {} reversals", executed.len());
                            
                            let mut stats_guard = stats.write().await;
                            stats_guard.revstop_alerts += executed.len() as u64;
                        }
                    }
                    Err(e) => error!("RevStop processing error: {}", e),
                }
            }
        });
    }
    
    /// Start performance monitoring
    async fn start_performance_monitor(&self) {
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Monitor every minute
            
            while *is_running.read().await {
                interval.tick().await;
                
                // Collect performance metrics
                let start_time = std::time::Instant::now();
                
                // Simulate some processing work to measure performance
                tokio::time::sleep(Duration::from_millis(1)).await;
                
                let processing_time = start_time.elapsed().as_secs_f64();
                
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.avg_processing_time = 
                        (stats_guard.avg_processing_time + processing_time) / 2.0;
                }
                
                debug!("📊 Performance check: {:.3}ms avg processing", processing_time * 1000.0);
            }
        });
    }
    
    /// Start network monitoring
    async fn start_network_monitor(&self) {
        let p2p_node = Arc::clone(&self.p2p_node);
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(15));
            let mut last_peer_count = 0usize;
            
            while *is_running.read().await {
                interval.tick().await;
                
                let network_stats = p2p_node.get_stats().await;
                let current_peer_count = network_stats.connected_peers;
                
                if current_peer_count != last_peer_count {
                    info!("🌐 Network update: {} peers connected", current_peer_count);
                    
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.network_events += 1;
                    }
                    
                    last_peer_count = current_peer_count;
                }
                
                // Check for network issues
                if current_peer_count == 0 {
                    error!("🚨 Network isolation detected - no peers connected");
                } else if current_peer_count > 20 {
                    info!("🌐 High peer connectivity: {} peers", current_peer_count);
                }
            }
        });
    }
    
    /// Log current system status
    async fn log_status(&self) {
        let blockchain = self.blockchain.read().await;
        let mempool = self.mempool.read().await;
        let network_stats = self.p2p_node.get_stats().await;
        let stats = self.stats.read().await;
        
        info!(
            "📊 QuantumCoin Status - Height: {}, Mempool: {}, Peers: {}, Uptime: {}s",
            blockchain.chain.len(),
            mempool.size(),
            network_stats.connected_peers,
            stats.uptime_seconds
        );
    }
    
    /// Get monitoring statistics
    pub async fn get_stats(&self) -> MonitorStats {
        self.stats.read().await.clone()
    }
    
    /// Get system health report
    pub async fn get_health_report(&self) -> SystemHealthReport {
        let blockchain = self.blockchain.read().await;
        let mempool = self.mempool.read().await;
        let network_stats = self.p2p_node.get_stats().await;
        let stats = self.stats.read().await;
        
        let ai_stats = {
            let ai_guard = self.ai_system.read().await;
            ai_guard.get_stats().clone()
        };
        
        let revstop_stats = {
            let revstop_guard = self.revstop.read().await;
            revstop_guard.get_stats().clone()
        };
        
        SystemHealthReport {
            timestamp: chrono::Utc::now(),
            blockchain_height: blockchain.chain.len() as u64,
            mempool_size: mempool.size(),
            connected_peers: network_stats.connected_peers,
            uptime_seconds: stats.uptime_seconds,
            ai_accuracy: ai_stats.prediction_accuracy,
            revstop_active_reversals: revstop_stats.total_reversals,
            system_status: if network_stats.connected_peers > 0 && blockchain.chain.len() > 0 {
                "Healthy".to_string()
            } else {
                "Degraded".to_string()
            },
            performance_metrics: PerformanceMetrics {
                blocks_per_second: stats.blocks_processed as f64 / stats.uptime_seconds.max(1) as f64,
                transactions_per_second: stats.transactions_processed as f64 / stats.uptime_seconds.max(1) as f64,
                avg_processing_time: stats.avg_processing_time,
                memory_usage_mb: 128.0, // Placeholder - would get real memory usage
                cpu_usage_percent: 25.0, // Placeholder - would get real CPU usage
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub blockchain_height: u64,
    pub mempool_size: usize,
    pub connected_peers: usize,
    pub uptime_seconds: u64,
    pub ai_accuracy: f64,
    pub revstop_active_reversals: u64,
    pub system_status: String,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub blocks_per_second: f64,
    pub transactions_per_second: f64,
    pub avg_processing_time: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{blockchain::Blockchain, mempool::Mempool};
    
    #[tokio::test]
    async fn test_block_monitor_creation() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let mempool = Arc::new(RwLock::new(Mempool::new(1000)));
        let p2p_node = Arc::new(P2PNode::new(
            "127.0.0.1:0".parse().unwrap(),
            Arc::clone(&blockchain),
            Arc::clone(&mempool),
        ));
        let ai_system = Arc::new(RwLock::new(AILearningSystem::new()));
        let revstop = Arc::new(RwLock::new(RevStop::new()));
        let economics = Arc::new(RwLock::new(EconomicsEngine::new()));
        
        let monitor = BlockMonitor::new(
            blockchain,
            mempool,
            p2p_node,
            ai_system,
            revstop,
            economics,
        );
        
        let stats = monitor.get_stats().await;
        assert_eq!(stats.uptime_seconds, 0);
        assert_eq!(stats.blocks_processed, 0);
    }
    
    #[tokio::test]
    async fn test_health_report_generation() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let mempool = Arc::new(RwLock::new(Mempool::new(1000)));
        let p2p_node = Arc::new(P2PNode::new(
            "127.0.0.1:0".parse().unwrap(),
            Arc::clone(&blockchain),
            Arc::clone(&mempool),
        ));
        let ai_system = Arc::new(RwLock::new(AILearningSystem::new()));
        let revstop = Arc::new(RwLock::new(RevStop::new()));
        let economics = Arc::new(RwLock::new(EconomicsEngine::new()));
        
        let monitor = BlockMonitor::new(
            blockchain,
            mempool,
            p2p_node,
            ai_system,
            revstop,
            economics,
        );
        
        let health_report = monitor.get_health_report().await;
        
        assert!(health_report.blockchain_height >= 1); // At least genesis
        assert_eq!(health_report.mempool_size, 0);
        assert!(health_report.ai_accuracy >= 0.0 && health_report.ai_accuracy <= 1.0);
    }
}
