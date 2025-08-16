// Network metrics and monitoring
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

/// Production-grade network metrics collection
pub struct NetworkMetrics {
    connections: Arc<RwLock<ConnectionMetrics>>,
    traffic: Arc<RwLock<TrafficMetrics>>,
    performance: Arc<RwLock<PerformanceMetrics>>,
    security: Arc<RwLock<SecurityMetrics>>,
    sync: Arc<RwLock<SyncMetrics>>,
    system: Arc<RwLock<SystemMetrics>>,
    start_time: Instant,
    event_sender: mpsc::Sender<MetricEvent>,
}

#[derive(Debug, Default)]
pub struct ConnectionMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub inbound_connections: u64,
    pub outbound_connections: u64,
    pub connection_attempts: u64,
    pub connection_failures: u64,
    pub disconnections: u64,
    pub avg_connection_duration: Duration,
    pub connection_durations: Vec<Duration>,
}

#[derive(Debug, Default)]
pub struct TrafficMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub message_types: HashMap<String, u64>,
    pub bandwidth_usage: f64, // MB/s
    pub peak_bandwidth: f64,
    pub traffic_history: Vec<TrafficSample>,
}

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub avg_latency: Duration,
    pub max_latency: Duration,
    pub min_latency: Duration,
    pub latency_samples: Vec<Duration>,
    pub message_processing_time: Duration,
    pub dns_resolution_time: Duration,
    pub connection_establishment_time: Duration,
    pub sync_speed: f64, // blocks per second
    pub memory_usage: u64,
    pub cpu_usage: f32,
}

#[derive(Debug, Default)]
pub struct SecurityMetrics {
    pub rejected_connections: u64,
    pub banned_ips: u64,
    pub rate_limited_connections: u64,
    pub protocol_violations: u64,
    pub threat_score_updates: u64,
    pub security_events: HashMap<String, u64>,
    pub dos_attempts: u64,
    pub malicious_behavior_detected: u64,
}

#[derive(Debug, Default)]
pub struct SyncMetrics {
    pub blocks_downloaded: u64,
    pub blocks_verified: u64,
    pub sync_progress: f32,
    pub sync_speed: f64, // blocks per minute
    pub sync_duration: Option<Duration>,
    pub sync_errors: u64,
    pub peer_sync_scores: HashMap<String, f32>,
    pub reorg_count: u64,
}

#[derive(Debug, Default)]
pub struct SystemMetrics {
    pub uptime: Duration,
    pub memory_usage: u64,
    pub memory_peak: u64,
    pub cpu_usage: f32,
    pub disk_usage: u64,
    pub network_interfaces: Vec<InterfaceStats>,
    pub thread_count: u32,
    pub file_descriptors: u32,
}

#[derive(Debug, Clone)]
pub struct TrafficSample {
    pub timestamp: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub message_count: u64,
}

#[derive(Debug, Clone)]
pub struct InterfaceStats {
    pub name: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug)]
pub enum MetricEvent {
    ConnectionEstablished,
    ConnectionFailed,
    ConnectionClosed,
    MessageSent(String, usize),
    MessageReceived(String, usize),
    LatencyMeasured(Duration),
    SecurityEvent(String),
    SyncProgress(f32),
    DnsResolution(Duration),
}

impl NetworkMetrics {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(10000);
        let start_time = Instant::now();
        
        let metrics = Self {
            connections: Arc::new(RwLock::new(ConnectionMetrics::default())),
            traffic: Arc::new(RwLock::new(TrafficMetrics::default())),
            performance: Arc::new(RwLock::new(PerformanceMetrics::default())),
            security: Arc::new(RwLock::new(SecurityMetrics::default())),
            sync: Arc::new(RwLock::new(SyncMetrics::default())),
            system: Arc::new(RwLock::new(SystemMetrics::default())),
            start_time,
            event_sender: tx,
        };

        // Start event processing
        let metrics_clone = metrics.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                metrics_clone.process_event(event).await;
            }
        });

        metrics
    }

    pub async fn start(&self) -> Result<()> {
        log::info!("Starting network metrics collection");
        
        // Start periodic system metrics collection
        let metrics = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                metrics.collect_system_metrics().await;
            }
        });

        // Start traffic analysis
        let metrics = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                metrics.analyze_traffic().await;
            }
        });

        // Start performance analysis
        let metrics = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                metrics.analyze_performance().await;
            }
        });

        Ok(())
    }

    // Connection metrics
    pub async fn increment_peer_connections(&self) {
        let _ = self.event_sender.send(MetricEvent::ConnectionEstablished).await;
    }

    pub async fn increment_connection_failures(&self) {
        let _ = self.event_sender.send(MetricEvent::ConnectionFailed).await;
    }

    pub async fn increment_peer_disconnections(&self) {
        let _ = self.event_sender.send(MetricEvent::ConnectionClosed).await;
    }

    // Traffic metrics
    pub async fn record_message_sent(&self, message_type: &str, size: usize) {
        let _ = self.event_sender.send(
            MetricEvent::MessageSent(message_type.to_string(), size)
        ).await;
    }

    pub async fn record_message_received(&self, message_type: &str, size: usize) {
        let _ = self.event_sender.send(
            MetricEvent::MessageReceived(message_type.to_string(), size)
        ).await;
    }

    // Performance metrics
    pub async fn record_connection_time(&self, duration: Duration) {
        let _ = self.event_sender.send(MetricEvent::LatencyMeasured(duration)).await;
    }

    pub async fn record_dns_resolution_time(&self, duration: Duration) {
        let _ = self.event_sender.send(MetricEvent::DnsResolution(duration)).await;
    }

    // Security metrics
    pub async fn increment_security_events(&self, event_type: &str) {
        let _ = self.event_sender.send(
            MetricEvent::SecurityEvent(event_type.to_string())
        ).await;
    }

    pub async fn increment_dns_failure(&self) {
        self.increment_security_events("dns_failure").await;
    }

    // Sync metrics
    pub async fn update_sync_progress(&self, progress: f32) {
        let _ = self.event_sender.send(MetricEvent::SyncProgress(progress)).await;
    }

    pub async fn record_dns_discovery(&self, address_count: usize, duration: Duration) {
        let mut performance = self.performance.write().await;
        performance.dns_resolution_time = duration;
        drop(performance);
        
        log::debug!("DNS discovery: {} addresses in {:?}", address_count, duration);
    }

    // Process events
    async fn process_event(&self, event: MetricEvent) {
        match event {
            MetricEvent::ConnectionEstablished => {
                let mut conn = self.connections.write().await;
                conn.total_connections += 1;
                conn.active_connections += 1;
                conn.connection_attempts += 1;
            }
            MetricEvent::ConnectionFailed => {
                let mut conn = self.connections.write().await;
                conn.connection_failures += 1;
                conn.connection_attempts += 1;
            }
            MetricEvent::ConnectionClosed => {
                let mut conn = self.connections.write().await;
                conn.active_connections = conn.active_connections.saturating_sub(1);
                conn.disconnections += 1;
            }
            MetricEvent::MessageSent(msg_type, size) => {
                let mut traffic = self.traffic.write().await;
                traffic.messages_sent += 1;
                traffic.bytes_sent += size as u64;
                *traffic.message_types.entry(msg_type).or_insert(0) += 1;
            }
            MetricEvent::MessageReceived(msg_type, size) => {
                let mut traffic = self.traffic.write().await;
                traffic.messages_received += 1;
                traffic.bytes_received += size as u64;
                *traffic.message_types.entry(msg_type).or_insert(0) += 1;
            }
            MetricEvent::LatencyMeasured(duration) => {
                let mut perf = self.performance.write().await;
                perf.latency_samples.push(duration);
                
                // Keep only recent samples (last 1000)
                if perf.latency_samples.len() > 1000 {
                    perf.latency_samples.remove(0);
                }
                
                // Update averages
                self.update_latency_stats(&mut perf).await;
            }
            MetricEvent::SecurityEvent(event_type) => {
                let mut security = self.security.write().await;
                *security.security_events.entry(event_type).or_insert(0) += 1;
            }
            MetricEvent::SyncProgress(progress) => {
                let mut sync = self.sync.write().await;
                sync.sync_progress = progress;
            }
            MetricEvent::DnsResolution(duration) => {
                let mut perf = self.performance.write().await;
                perf.dns_resolution_time = duration;
            }
        }
    }

    async fn update_latency_stats(&self, perf: &mut PerformanceMetrics) {
        if perf.latency_samples.is_empty() {
            return;
        }
        
        let total: Duration = perf.latency_samples.iter().sum();
        perf.avg_latency = total / perf.latency_samples.len() as u32;
        perf.max_latency = perf.latency_samples.iter().max().copied().unwrap_or_default();
        perf.min_latency = perf.latency_samples.iter().min().copied().unwrap_or_default();
    }

    // System metrics collection
    async fn collect_system_metrics(&self) {
        let mut system = self.system.write().await;
        system.uptime = self.start_time.elapsed();
        
        // Collect memory usage
        if let Ok(memory) = self.get_memory_usage().await {
            system.memory_usage = memory;
            if memory > system.memory_peak {
                system.memory_peak = memory;
            }
        }
        
        // Collect CPU usage
        if let Ok(cpu) = self.get_cpu_usage().await {
            system.cpu_usage = cpu;
            
            let mut perf = self.performance.write().await;
            perf.cpu_usage = cpu;
        }
        
        // Collect thread count
        system.thread_count = self.get_thread_count().await;
    }

    async fn analyze_traffic(&self) {
        let mut traffic = self.traffic.write().await;
        
        // Add current sample
        let sample = TrafficSample {
            timestamp: Instant::now(),
            bytes_sent: traffic.bytes_sent,
            bytes_received: traffic.bytes_received,
            message_count: traffic.messages_sent + traffic.messages_received,
        };
        
        traffic.traffic_history.push(sample);
        
        // Keep only last 300 samples (5 minutes at 1 second intervals)
        if traffic.traffic_history.len() > 300 {
            traffic.traffic_history.remove(0);
        }
        
        // Calculate bandwidth usage
        if traffic.traffic_history.len() >= 2 {
            let recent = &traffic.traffic_history[traffic.traffic_history.len() - 1];
            let previous = &traffic.traffic_history[traffic.traffic_history.len() - 2];
            
            let time_diff = recent.timestamp.duration_since(previous.timestamp).as_secs_f64();
            let bytes_diff = (recent.bytes_sent + recent.bytes_received) 
                           - (previous.bytes_sent + previous.bytes_received);
            
            let bandwidth_mbps = (bytes_diff as f64) / time_diff / 1_048_576.0; // MB/s
            traffic.bandwidth_usage = bandwidth_mbps;
            
            if bandwidth_mbps > traffic.peak_bandwidth {
                traffic.peak_bandwidth = bandwidth_mbps;
            }
        }
    }

    async fn analyze_performance(&self) {
        // Update connection duration averages
        let mut conn = self.connections.write().await;
        if !conn.connection_durations.is_empty() {
            let total: Duration = conn.connection_durations.iter().sum();
            conn.avg_connection_duration = total / conn.connection_durations.len() as u32;
        }
    }

    // System information helpers
    async fn get_memory_usage(&self) -> Result<u64> {
        // Platform-specific memory usage collection
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
            use windows::Win32::System::Threading::GetCurrentProcess;
            
            unsafe {
                let mut counters = PROCESS_MEMORY_COUNTERS::default();
                if GetProcessMemoryInfo(
                    GetCurrentProcess(),
                    &mut counters,
                    std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                ).is_ok() {
                    return Ok(counters.WorkingSetSize as u64);
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/proc/self/status").await {
                for line in contents.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return Ok(kb * 1024); // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        Ok(0) // Fallback
    }

    async fn get_cpu_usage(&self) -> Result<f32> {
        // Simplified CPU usage - would use system APIs in production
        Ok(0.0)
    }

    async fn get_thread_count(&self) -> u32 {
        // Platform-specific thread count
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/proc/self/status").await {
                for line in contents.lines() {
                    if line.starts_with("Threads:") {
                        if let Some(count_str) = line.split_whitespace().nth(1) {
                            if let Ok(count) = count_str.parse::<u32>() {
                                return count;
                            }
                        }
                    }
                }
            }
        }
        
        0 // Fallback
    }

    // Public API for getting metrics
    pub async fn get_network_hashrate(&self) -> f64 {
        // This would be calculated from network data
        // Placeholder implementation
        0.0
    }

    pub async fn get_uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub async fn get_connection_stats(&self) -> ConnectionMetrics {
        self.connections.read().await.clone()
    }

    pub async fn get_traffic_stats(&self) -> TrafficMetrics {
        self.traffic.read().await.clone()
    }

    pub async fn get_performance_stats(&self) -> PerformanceMetrics {
        self.performance.read().await.clone()
    }

    pub async fn get_security_stats(&self) -> SecurityMetrics {
        self.security.read().await.clone()
    }

    pub async fn get_sync_stats(&self) -> SyncMetrics {
        self.sync.read().await.clone()
    }

    pub async fn get_system_stats(&self) -> SystemMetrics {
        self.system.read().await.clone()
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let conn = self.connections.read().await;
        let traffic = self.traffic.read().await;
        let perf = self.performance.read().await;
        let security = self.security.read().await;
        let sync = self.sync.read().await;
        let system = self.system.read().await;
        
        format!(
            r#"# HELP quantumcoin_connections_total Total number of connections
# TYPE quantumcoin_connections_total counter
quantumcoin_connections_total {{}} {}

# HELP quantumcoin_connections_active Current active connections
# TYPE quantumcoin_connections_active gauge
quantumcoin_connections_active {{}} {}

# HELP quantumcoin_bytes_sent_total Total bytes sent
# TYPE quantumcoin_bytes_sent_total counter
quantumcoin_bytes_sent_total {{}} {}

# HELP quantumcoin_bytes_received_total Total bytes received
# TYPE quantumcoin_bytes_received_total counter
quantumcoin_bytes_received_total {{}} {}

# HELP quantumcoin_messages_sent_total Total messages sent
# TYPE quantumcoin_messages_sent_total counter
quantumcoin_messages_sent_total {{}} {}

# HELP quantumcoin_messages_received_total Total messages received
# TYPE quantumcoin_messages_received_total counter
quantumcoin_messages_received_total {{}} {}

# HELP quantumcoin_latency_avg Average connection latency in seconds
# TYPE quantumcoin_latency_avg gauge
quantumcoin_latency_avg {{}} {}

# HELP quantumcoin_bandwidth_usage Current bandwidth usage in MB/s
# TYPE quantumcoin_bandwidth_usage gauge
quantumcoin_bandwidth_usage {{}} {}

# HELP quantumcoin_sync_progress Blockchain sync progress (0.0 to 1.0)
# TYPE quantumcoin_sync_progress gauge
quantumcoin_sync_progress {{}} {}

# HELP quantumcoin_uptime_seconds Node uptime in seconds
# TYPE quantumcoin_uptime_seconds counter
quantumcoin_uptime_seconds {{}} {}

# HELP quantumcoin_memory_usage_bytes Current memory usage in bytes
# TYPE quantumcoin_memory_usage_bytes gauge
quantumcoin_memory_usage_bytes {{}} {}
"#,
            conn.total_connections,
            conn.active_connections,
            traffic.bytes_sent,
            traffic.bytes_received,
            traffic.messages_sent,
            traffic.messages_received,
            perf.avg_latency.as_secs_f64(),
            traffic.bandwidth_usage,
            sync.sync_progress,
            system.uptime.as_secs(),
            system.memory_usage,
        )
    }

    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down network metrics");
        Ok(())
    }
}

impl Clone for NetworkMetrics {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            traffic: self.traffic.clone(),
            performance: self.performance.clone(),
            security: self.security.clone(),
            sync: self.sync.clone(),
            system: self.system.clone(),
            start_time: self.start_time,
            event_sender: self.event_sender.clone(),
        }
    }
}
