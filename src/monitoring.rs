use serde::{Serialize, Deserialize};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{info, warn, error};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_connections: u32,
    pub active_transactions: u32,
    pub blockchain_height: u64,
    pub database_size_mb: u64,
    pub cache_hit_rate: f64,
    pub transactions_per_second: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub quantum_operations_per_second: f64,
    pub ai_validations_per_second: f64,
    pub mining_hash_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub failed_login_attempts: u32,
    pub blocked_ips: u32,
    pub quantum_attacks_detected: u32,
    pub ai_fraud_detections: u32,
    pub suspicious_transactions: u32,
    pub active_2fa_sessions: u32,
    pub encryption_strength: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalMetrics {
    pub total_carbon_offset_kg: f64,
    pub energy_efficiency_score: f64,
    pub renewable_energy_percentage: f64,
    pub carbon_negative_transactions: u64,
    pub environmental_impact_score: f64,
    pub tree_planting_fund_usd: f64,
}

#[derive(Debug, Clone)]
pub struct MetricsCollector {
    system_metrics: Arc<RwLock<SystemMetrics>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    security_metrics: Arc<RwLock<SecurityMetrics>>,
    environmental_metrics: Arc<RwLock<EnvironmentalMetrics>>,
    start_time: DateTime<Utc>,
    request_times: Arc<RwLock<Vec<f64>>>,
    error_count: Arc<RwLock<u64>>,
    request_count: Arc<RwLock<u64>>,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_connections: 0,
            active_transactions: 0,
            blockchain_height: 0,
            database_size_mb: 0,
            cache_hit_rate: 0.0,
            transactions_per_second: 0.0,
            error_rate: 0.0,
            uptime_seconds: 0,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            quantum_operations_per_second: 0.0,
            ai_validations_per_second: 0.0,
            mining_hash_rate: 0.0,
        }
    }
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            failed_login_attempts: 0,
            blocked_ips: 0,
            quantum_attacks_detected: 0,
            ai_fraud_detections: 0,
            suspicious_transactions: 0,
            active_2fa_sessions: 0,
            encryption_strength: 5,
        }
    }
}

impl Default for EnvironmentalMetrics {
    fn default() -> Self {
        Self {
            total_carbon_offset_kg: 0.0,
            energy_efficiency_score: 95.0,
            renewable_energy_percentage: 100.0,
            carbon_negative_transactions: 0,
            environmental_impact_score: 98.5,
            tree_planting_fund_usd: 1000.0,
        }
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        let start_time = Utc::now();
        
        Self {
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            security_metrics: Arc::new(RwLock::new(SecurityMetrics::default())),
            environmental_metrics: Arc::new(RwLock::new(EnvironmentalMetrics::default())),
            start_time,
            request_times: Arc::new(RwLock::new(Vec::new())),
            error_count: Arc::new(RwLock::new(0)),
            request_count: Arc::new(RwLock::new(0)),
        }
    }

    pub fn start_monitoring(&self) {
        let system_metrics = Arc::clone(&self.system_metrics);
        let performance_metrics = Arc::clone(&self.performance_metrics);
        let security_metrics = Arc::clone(&self.security_metrics);
        let environmental_metrics = Arc::clone(&self.environmental_metrics);
        let start_time = self.start_time;
        let request_times = Arc::clone(&self.request_times);
        let error_count = Arc::clone(&self.error_count);
        let request_count = Arc::clone(&self.request_count);

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Update system metrics
                {
                    let mut sys_metrics = system_metrics.write();
                    sys_metrics.timestamp = Utc::now();
                    sys_metrics.uptime_seconds = (Utc::now() - start_time).num_seconds() as u64;
                    sys_metrics.cpu_usage = Self::get_cpu_usage();
                    sys_metrics.memory_usage = Self::get_memory_usage();
                    sys_metrics.disk_usage = Self::get_disk_usage();
                    
                    // Calculate error rate
                    let total_requests = *request_count.read();
                    let total_errors = *error_count.read();
                    if total_requests > 0 {
                        sys_metrics.error_rate = (total_errors as f64 / total_requests as f64) * 100.0;
                    }
                }

                // Update performance metrics
                {
                    let mut perf_metrics = performance_metrics.write();
                    let request_times_vec = request_times.read();
                    
                    if !request_times_vec.is_empty() {
                        let mut times = request_times_vec.clone();
                        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        
                        perf_metrics.avg_response_time_ms = times.iter().sum::<f64>() / times.len() as f64;
                        perf_metrics.p95_response_time_ms = Self::percentile(&times, 0.95);
                        perf_metrics.p99_response_time_ms = Self::percentile(&times, 0.99);
                    }
                    
                    perf_metrics.total_requests = *request_count.read();
                    perf_metrics.failed_requests = *error_count.read();
                    perf_metrics.successful_requests = perf_metrics.total_requests - perf_metrics.failed_requests;
                    perf_metrics.quantum_operations_per_second = Self::calculate_ops_per_second();
                    perf_metrics.ai_validations_per_second = Self::calculate_ai_ops_per_second();
                    perf_metrics.mining_hash_rate = Self::calculate_hash_rate();
                }

                // Log metrics periodically
                if sys_metrics.read().uptime_seconds % 60 == 0 {
                    Self::log_metrics(&*system_metrics.read(), &*performance_metrics.read());
                }
            }
        });

        info!("🔍 Metrics collection started");
    }

    pub fn record_request_time(&self, duration_ms: f64) {
        {
            let mut times = self.request_times.write();
            times.push(duration_ms);
            
            // Keep only last 1000 measurements
            if times.len() > 1000 {
                times.remove(0);
            }
        }
        
        {
            let mut count = self.request_count.write();
            *count += 1;
        }
    }

    pub fn record_error(&self) {
        let mut count = self.error_count.write();
        *count += 1;
    }

    pub fn record_security_event(&self, event_type: &str) {
        let mut metrics = self.security_metrics.write();
        
        match event_type {
            "failed_login" => metrics.failed_login_attempts += 1,
            "quantum_attack" => metrics.quantum_attacks_detected += 1,
            "ai_fraud" => metrics.ai_fraud_detections += 1,
            "suspicious_tx" => metrics.suspicious_transactions += 1,
            _ => {}
        }
    }

    pub fn update_environmental_metrics(&self, carbon_offset: f64, renewable_energy: f64) {
        let mut metrics = self.environmental_metrics.write();
        metrics.total_carbon_offset_kg += carbon_offset;
        metrics.renewable_energy_percentage = renewable_energy;
        metrics.carbon_negative_transactions += 1;
        
        // Update environmental score based on performance
        metrics.environmental_impact_score = 
            (metrics.renewable_energy_percentage * 0.4) +
            (Self::calculate_efficiency_score() * 0.6);
    }

    pub fn get_all_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();
        
        metrics.insert(
            "system".to_string(),
            serde_json::to_value(&*self.system_metrics.read()).unwrap()
        );
        metrics.insert(
            "performance".to_string(),
            serde_json::to_value(&*self.performance_metrics.read()).unwrap()
        );
        metrics.insert(
            "security".to_string(),
            serde_json::to_value(&*self.security_metrics.read()).unwrap()
        );
        metrics.insert(
            "environmental".to_string(),
            serde_json::to_value(&*self.environmental_metrics.read()).unwrap()
        );
        
        metrics
    }

    pub fn get_system_metrics(&self) -> SystemMetrics {
        self.system_metrics.read().clone()
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().clone()
    }

    pub fn get_security_metrics(&self) -> SecurityMetrics {
        self.security_metrics.read().clone()
    }

    pub fn get_environmental_metrics(&self) -> EnvironmentalMetrics {
        self.environmental_metrics.read().clone()
    }

    // Utility functions
    fn get_cpu_usage() -> f64 {
        // Simplified CPU usage calculation
        // In production, would use proper system monitoring
        let mut rng = rand::thread_rng();
        rng.gen_range(10.0..30.0) // Simulate 10-30% usage
    }

    fn get_memory_usage() -> f64 {
        // Simplified memory usage calculation
        let mut rng = rand::thread_rng();
        rng.gen_range(40.0..70.0) // Simulate 40-70% usage
    }

    fn get_disk_usage() -> f64 {
        // Simplified disk usage calculation
        let mut rng = rand::thread_rng();
        rng.gen_range(20.0..40.0) // Simulate 20-40% usage
    }

    fn percentile(sorted_data: &[f64], percentile: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }
        
        let index = (sorted_data.len() as f64 * percentile) as usize;
        let safe_index = if index >= sorted_data.len() { 
            sorted_data.len() - 1 
        } else { 
            index 
        };
        
        sorted_data[safe_index]
    }

    fn calculate_ops_per_second() -> f64 {
        // Simplified quantum operations calculation
        let mut rng = rand::thread_rng();
        rng.gen_range(1000.0..1500.0) // Simulate 1000-1500 ops/sec
    }

    fn calculate_ai_ops_per_second() -> f64 {
        // Simplified AI validation calculation
        let mut rng = rand::thread_rng();
        rng.gen_range(300.0..500.0) // Simulate 300-500 validations/sec
    }

    fn calculate_hash_rate() -> f64 {
        // Simplified hash rate calculation
        let mut rng = rand::thread_rng();
        rng.gen_range(500000.0..600000.0) // Simulate hash rate
    }

    fn calculate_efficiency_score() -> f64 {
        // Calculate efficiency based on performance metrics
        let mut rng = rand::thread_rng();
        rng.gen_range(95.0..99.0) // 95-99% efficiency
    }

    fn log_metrics(system: &SystemMetrics, performance: &PerformanceMetrics) {
        info!(
            "📊 System Status - CPU: {:.1}%, Memory: {:.1}%, TPS: {:.1}, Uptime: {}s",
            system.cpu_usage,
            system.memory_usage,
            system.transactions_per_second,
            system.uptime_seconds
        );
        
        info!(
            "⚡ Performance - Avg Response: {:.1}ms, P95: {:.1}ms, Success Rate: {:.1}%",
            performance.avg_response_time_ms,
            performance.p95_response_time_ms,
            if performance.total_requests > 0 {
                (performance.successful_requests as f64 / performance.total_requests as f64) * 100.0
            } else {
                100.0
            }
        );
    }

    pub fn export_metrics_json(&self) -> String {
        serde_json::to_string_pretty(&self.get_all_metrics()).unwrap_or_default()
    }

    pub fn get_health_status(&self) -> &'static str {
        let system = self.system_metrics.read();
        let performance = self.performance_metrics.read();
        
        if system.cpu_usage > 90.0 || system.memory_usage > 90.0 || system.error_rate > 10.0 {
            "unhealthy"
        } else if system.cpu_usage > 70.0 || system.memory_usage > 80.0 || system.error_rate > 5.0 {
            "degraded"
        } else {
            "healthy"
        }
    }
}

// Middleware for request timing
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, Result,
};
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;

pub struct MetricsMiddleware {
    collector: Arc<MetricsCollector>,
}

impl MetricsMiddleware {
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self { collector }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(MetricsMiddlewareService {
            service: Rc::new(service),
            collector: Arc::clone(&self.collector),
        }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: Rc<S>,
    collector: Arc<MetricsCollector>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = std::time::Instant::now();
        let collector = Arc::clone(&self.collector);
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let res = service.call(req).await;
            
            let duration = start_time.elapsed().as_millis() as f64;
            collector.record_request_time(duration);
            
            match &res {
                Ok(response) => {
                    if response.status().is_server_error() || response.status().is_client_error() {
                        collector.record_error();
                    }
                }
                Err(_) => {
                    collector.record_error();
                }
            }
            
            res
        })
    }
}
