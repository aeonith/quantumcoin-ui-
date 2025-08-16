// DoS protection and security management
use crate::network::{ChainSpec, NetworkMetrics};
use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Security manager for DoS protection and peer validation
pub struct SecurityManager {
    chain_spec: Arc<ChainSpec>,
    metrics: Arc<NetworkMetrics>,
    connection_limits: Arc<RwLock<ConnectionLimits>>,
    rate_limiters: Arc<RwLock<HashMap<IpAddr, RateLimiter>>>,
    threat_detection: Arc<RwLock<ThreatDetection>>,
    security_config: SecurityConfig,
}

#[derive(Debug)]
pub struct ConnectionLimits {
    pub per_ip: HashMap<IpAddr, ConnectionInfo>,
    pub per_subnet: HashMap<String, u32>,
    pub global_connections: u32,
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub count: u32,
    pub last_connection: Instant,
    pub failed_attempts: u32,
    pub last_failure: Option<Instant>,
}

#[derive(Debug)]
pub struct RateLimiter {
    pub requests: VecDeque<Instant>,
    pub bytes_sent: VecDeque<(Instant, u64)>,
    pub violations: u32,
    pub last_violation: Option<Instant>,
    pub is_throttled: bool,
    pub throttle_until: Option<Instant>,
}

#[derive(Debug)]
pub struct ThreatDetection {
    pub suspicious_ips: HashMap<IpAddr, ThreatScore>,
    pub attack_patterns: Vec<AttackPattern>,
    pub recent_attacks: VecDeque<AttackEvent>,
}

#[derive(Debug, Clone)]
pub struct ThreatScore {
    pub score: f32,
    pub last_updated: Instant,
    pub indicators: Vec<ThreatIndicator>,
}

#[derive(Debug, Clone)]
pub enum ThreatIndicator {
    HighConnectionRate,
    ExcessiveMessageRate,
    InvalidProtocolUsage,
    SuspiciousUserAgent,
    KnownMaliciousIp,
    BotnetPattern,
    TimeBasedAttack,
}

#[derive(Debug)]
pub struct AttackPattern {
    pub pattern_type: AttackType,
    pub threshold: f32,
    pub window: Duration,
    pub description: String,
}

#[derive(Debug, PartialEq)]
pub enum AttackType {
    ConnectionFlood,
    MessageFlood,
    SlowLoris,
    ResourceExhaustion,
    ProtocolAbuse,
    Eclipse,
}

#[derive(Debug)]
pub struct AttackEvent {
    pub attack_type: AttackType,
    pub source_ip: IpAddr,
    pub timestamp: Instant,
    pub severity: AttackSeverity,
    pub details: String,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum AttackSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct SecurityConfig {
    pub max_connections_per_ip: u32,
    pub max_connections_per_subnet: u32,
    pub max_failed_attempts: u32,
    pub failure_ban_duration: Duration,
    pub message_rate_limit: u32,
    pub bytes_rate_limit: u64,
    pub rate_limit_window: Duration,
    pub threat_score_threshold: f32,
    pub enable_ip_reputation: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_connections_per_ip: 3,
            max_connections_per_subnet: 10,
            max_failed_attempts: 5,
            failure_ban_duration: Duration::from_secs(3600), // 1 hour
            message_rate_limit: 100,  // messages per minute
            bytes_rate_limit: 1024 * 1024, // 1MB per minute
            rate_limit_window: Duration::from_secs(60),
            threat_score_threshold: 0.7,
            enable_ip_reputation: true,
        }
    }
}

impl SecurityManager {
    pub fn new(chain_spec: Arc<ChainSpec>, metrics: Arc<NetworkMetrics>) -> Self {
        let attack_patterns = vec![
            AttackPattern {
                pattern_type: AttackType::ConnectionFlood,
                threshold: 0.8,
                window: Duration::from_secs(60),
                description: "Rapid connection attempts from single IP".to_string(),
            },
            AttackPattern {
                pattern_type: AttackType::MessageFlood,
                threshold: 0.9,
                window: Duration::from_secs(30),
                description: "Excessive message rate".to_string(),
            },
            AttackPattern {
                pattern_type: AttackType::SlowLoris,
                threshold: 0.7,
                window: Duration::from_secs(300),
                description: "Slow connection exhaustion attack".to_string(),
            },
            AttackPattern {
                pattern_type: AttackType::ResourceExhaustion,
                threshold: 0.8,
                window: Duration::from_secs(120),
                description: "Resource exhaustion attempt".to_string(),
            },
        ];

        Self {
            chain_spec,
            metrics,
            connection_limits: Arc::new(RwLock::new(ConnectionLimits {
                per_ip: HashMap::new(),
                per_subnet: HashMap::new(),
                global_connections: 0,
            })),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            threat_detection: Arc::new(RwLock::new(ThreatDetection {
                suspicious_ips: HashMap::new(),
                attack_patterns,
                recent_attacks: VecDeque::new(),
            })),
            security_config: SecurityConfig::default(),
        }
    }

    /// Check if connection should be allowed
    pub async fn allow_connection(&self, addr: SocketAddr) -> bool {
        let ip = addr.ip();
        
        // Check IP-based limits
        if !self.check_ip_limits(ip).await {
            log::warn!("Connection rejected from {}: IP limits exceeded", ip);
            return false;
        }
        
        // Check subnet limits
        if !self.check_subnet_limits(ip).await {
            log::warn!("Connection rejected from {}: Subnet limits exceeded", ip);
            return false;
        }
        
        // Check threat score
        if let Some(threat_score) = self.get_threat_score(ip).await {
            if threat_score > self.security_config.threat_score_threshold {
                log::warn!("Connection rejected from {}: High threat score ({})", ip, threat_score);
                return false;
            }
        }
        
        // Check if IP is currently rate limited
        if self.is_rate_limited(ip).await {
            log::debug!("Connection rejected from {}: Rate limited", ip);
            return false;
        }
        
        // Update connection tracking
        self.record_connection_attempt(ip, true).await;
        
        true
    }

    /// Record successful connection
    pub async fn on_connection_established(&self, addr: SocketAddr) {
        let ip = addr.ip();
        
        let mut limits = self.connection_limits.write().await;
        limits.global_connections += 1;
        
        if let Some(info) = limits.per_ip.get_mut(&ip) {
            info.count += 1;
            info.last_connection = Instant::now();
        }
        
        // Update subnet count
        let subnet = get_subnet_key(ip);
        *limits.per_subnet.entry(subnet).or_insert(0) += 1;
        
        self.metrics.increment_security_events("connection_allowed").await;
    }

    /// Record connection failure
    pub async fn on_connection_failed(&self, addr: SocketAddr, reason: &str) {
        let ip = addr.ip();
        self.record_connection_attempt(ip, false).await;
        
        // Update threat score
        self.update_threat_score(ip, vec![ThreatIndicator::HighConnectionRate]).await;
        
        log::debug!("Connection failed from {}: {}", ip, reason);
        self.metrics.increment_security_events("connection_denied").await;
    }

    /// Record connection closed
    pub async fn on_connection_closed(&self, addr: SocketAddr) {
        let ip = addr.ip();
        
        let mut limits = self.connection_limits.write().await;
        limits.global_connections = limits.global_connections.saturating_sub(1);
        
        if let Some(info) = limits.per_ip.get_mut(&ip) {
            info.count = info.count.saturating_sub(1);
        }
        
        // Update subnet count
        let subnet = get_subnet_key(ip);
        if let Some(count) = limits.per_subnet.get_mut(&subnet) {
            *count = count.saturating_sub(1);
        }
    }

    /// Check message rate limits
    pub async fn check_message_rate(&self, addr: SocketAddr, message_size: u64) -> bool {
        let ip = addr.ip();
        
        let mut limiters = self.rate_limiters.write().await;
        let limiter = limiters.entry(ip).or_insert_with(|| RateLimiter {
            requests: VecDeque::new(),
            bytes_sent: VecDeque::new(),
            violations: 0,
            last_violation: None,
            is_throttled: false,
            throttle_until: None,
        });
        
        let now = Instant::now();
        
        // Check if currently throttled
        if let Some(throttle_until) = limiter.throttle_until {
            if now < throttle_until {
                return false;
            } else {
                limiter.is_throttled = false;
                limiter.throttle_until = None;
            }
        }
        
        // Clean old entries
        let window_start = now - self.security_config.rate_limit_window;
        limiter.requests.retain(|&timestamp| timestamp > window_start);
        limiter.bytes_sent.retain(|(timestamp, _)| *timestamp > window_start);
        
        // Check message rate
        if limiter.requests.len() >= self.security_config.message_rate_limit as usize {
            self.record_rate_limit_violation(ip, "message_rate").await;
            return false;
        }
        
        // Check byte rate
        let bytes_in_window: u64 = limiter.bytes_sent.iter().map(|(_, bytes)| *bytes).sum();
        if bytes_in_window + message_size > self.security_config.bytes_rate_limit {
            self.record_rate_limit_violation(ip, "byte_rate").await;
            return false;
        }
        
        // Record this request
        limiter.requests.push_back(now);
        limiter.bytes_sent.push_back((now, message_size));
        
        true
    }

    /// Record rate limit violation
    async fn record_rate_limit_violation(&self, ip: IpAddr, violation_type: &str) {
        let mut limiters = self.rate_limiters.write().await;
        if let Some(limiter) = limiters.get_mut(&ip) {
            limiter.violations += 1;
            limiter.last_violation = Some(Instant::now());
            
            // Escalating throttle
            let throttle_duration = match limiter.violations {
                1..=2 => Duration::from_secs(30),
                3..=5 => Duration::from_secs(300),   // 5 minutes
                6..=10 => Duration::from_secs(1800), // 30 minutes
                _ => Duration::from_secs(3600),      // 1 hour
            };
            
            limiter.is_throttled = true;
            limiter.throttle_until = Some(Instant::now() + throttle_duration);
        }
        
        log::warn!("Rate limit violation from {}: {} (throttled)", ip, violation_type);
        self.metrics.increment_security_events("rate_limit_violation").await;
        
        // Update threat score
        self.update_threat_score(ip, vec![ThreatIndicator::ExcessiveMessageRate]).await;
    }

    /// Check IP connection limits
    async fn check_ip_limits(&self, ip: IpAddr) -> bool {
        let limits = self.connection_limits.read().await;
        
        if let Some(info) = limits.per_ip.get(&ip) {
            // Check active connections
            if info.count >= self.security_config.max_connections_per_ip {
                return false;
            }
            
            // Check failure rate
            if info.failed_attempts >= self.security_config.max_failed_attempts {
                if let Some(last_failure) = info.last_failure {
                    if last_failure.elapsed() < self.security_config.failure_ban_duration {
                        return false;
                    }
                }
            }
        }
        
        true
    }

    /// Check subnet connection limits
    async fn check_subnet_limits(&self, ip: IpAddr) -> bool {
        let limits = self.connection_limits.read().await;
        let subnet = get_subnet_key(ip);
        
        if let Some(&count) = limits.per_subnet.get(&subnet) {
            return count < self.security_config.max_connections_per_subnet;
        }
        
        true
    }

    /// Check if IP is rate limited
    async fn is_rate_limited(&self, ip: IpAddr) -> bool {
        let limiters = self.rate_limiters.read().await;
        
        if let Some(limiter) = limiters.get(&ip) {
            if limiter.is_throttled {
                if let Some(throttle_until) = limiter.throttle_until {
                    return Instant::now() < throttle_until;
                }
            }
        }
        
        false
    }

    /// Record connection attempt
    async fn record_connection_attempt(&self, ip: IpAddr, success: bool) {
        let mut limits = self.connection_limits.write().await;
        let info = limits.per_ip.entry(ip).or_insert(ConnectionInfo {
            count: 0,
            last_connection: Instant::now(),
            failed_attempts: 0,
            last_failure: None,
        });
        
        info.last_connection = Instant::now();
        
        if success {
            // Reset failure count on success
            info.failed_attempts = 0;
            info.last_failure = None;
        } else {
            info.failed_attempts += 1;
            info.last_failure = Some(Instant::now());
        }
    }

    /// Get threat score for IP
    async fn get_threat_score(&self, ip: IpAddr) -> Option<f32> {
        let detection = self.threat_detection.read().await;
        detection.suspicious_ips.get(&ip).map(|score| score.score)
    }

    /// Update threat score
    async fn update_threat_score(&self, ip: IpAddr, indicators: Vec<ThreatIndicator>) {
        let mut detection = self.threat_detection.write().await;
        
        let threat_score = detection.suspicious_ips.entry(ip).or_insert(ThreatScore {
            score: 0.0,
            last_updated: Instant::now(),
            indicators: Vec::new(),
        });
        
        // Add new indicators
        for indicator in indicators {
            if !threat_score.indicators.contains(&indicator) {
                threat_score.indicators.push(indicator.clone());
            }
            
            // Increase threat score based on indicator
            let score_increase = match indicator {
                ThreatIndicator::HighConnectionRate => 0.2,
                ThreatIndicator::ExcessiveMessageRate => 0.3,
                ThreatIndicator::InvalidProtocolUsage => 0.4,
                ThreatIndicator::SuspiciousUserAgent => 0.1,
                ThreatIndicator::KnownMaliciousIp => 0.8,
                ThreatIndicator::BotnetPattern => 0.6,
                ThreatIndicator::TimeBasedAttack => 0.5,
            };
            
            threat_score.score = (threat_score.score + score_increase).min(1.0);
        }
        
        threat_score.last_updated = Instant::now();
        
        // Record attack event if score is high
        if threat_score.score > self.security_config.threat_score_threshold {
            let attack_event = AttackEvent {
                attack_type: AttackType::ProtocolAbuse,
                source_ip: ip,
                timestamp: Instant::now(),
                severity: if threat_score.score > 0.9 { AttackSeverity::Critical }
                         else if threat_score.score > 0.8 { AttackSeverity::High }
                         else { AttackSeverity::Medium },
                details: format!("Threat indicators: {:?}", threat_score.indicators),
            };
            
            detection.recent_attacks.push_back(attack_event);
            
            // Keep only recent attacks (last 1000)
            if detection.recent_attacks.len() > 1000 {
                detection.recent_attacks.pop_front();
            }
        }
        
        log::debug!("Updated threat score for {}: {:.2}", ip, threat_score.score);
    }

    /// Detect attack patterns
    pub async fn detect_attack_patterns(&self) -> Vec<AttackEvent> {
        let detection = self.threat_detection.read().await;
        let mut detected_attacks = Vec::new();
        
        // Analyze recent connection patterns
        let limits = self.connection_limits.read().await;
        let now = Instant::now();
        
        for (ip, info) in &limits.per_ip {
            // Check for connection flood
            if info.failed_attempts > 10 && 
               info.last_failure.map_or(false, |t| now.duration_since(t) < Duration::from_secs(60)) {
                
                detected_attacks.push(AttackEvent {
                    attack_type: AttackType::ConnectionFlood,
                    source_ip: *ip,
                    timestamp: now,
                    severity: AttackSeverity::High,
                    details: format!("Failed attempts: {}", info.failed_attempts),
                });
            }
        }
        
        // Check rate limiter violations
        let limiters = self.rate_limiters.read().await;
        for (ip, limiter) in limiters.iter() {
            if limiter.violations > 5 {
                detected_attacks.push(AttackEvent {
                    attack_type: AttackType::MessageFlood,
                    source_ip: *ip,
                    timestamp: now,
                    severity: AttackSeverity::Medium,
                    details: format!("Rate limit violations: {}", limiter.violations),
                });
            }
        }
        
        detected_attacks
    }

    /// Get security statistics
    pub async fn get_security_stats(&self) -> SecurityStats {
        let limits = self.connection_limits.read().await;
        let limiters = self.rate_limiters.read().await;
        let detection = self.threat_detection.read().await;
        
        SecurityStats {
            total_connections: limits.global_connections,
            unique_ips: limits.per_ip.len() as u32,
            rate_limited_ips: limiters.values().filter(|l| l.is_throttled).count() as u32,
            suspicious_ips: detection.suspicious_ips.len() as u32,
            recent_attacks: detection.recent_attacks.len() as u32,
            banned_ips: limits.per_ip.values().filter(|info| {
                info.failed_attempts >= self.security_config.max_failed_attempts &&
                info.last_failure.map_or(false, |t| t.elapsed() < self.security_config.failure_ban_duration)
            }).count() as u32,
        }
    }

    /// Cleanup old entries periodically
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let cleanup_threshold = Duration::from_secs(3600); // 1 hour
        
        // Cleanup connection limits
        let mut limits = self.connection_limits.write().await;
        limits.per_ip.retain(|_, info| {
            info.count > 0 || now.duration_since(info.last_connection) < cleanup_threshold
        });
        
        // Cleanup rate limiters
        let mut limiters = self.rate_limiters.write().await;
        limiters.retain(|_, limiter| {
            !limiter.requests.is_empty() || !limiter.bytes_sent.is_empty() || limiter.is_throttled
        });
        
        // Cleanup threat detection
        let mut detection = self.threat_detection.write().await;
        detection.suspicious_ips.retain(|_, score| {
            now.duration_since(score.last_updated) < Duration::from_secs(86400) // 24 hours
        });
    }
}

impl Clone for SecurityManager {
    fn clone(&self) -> Self {
        Self {
            chain_spec: self.chain_spec.clone(),
            metrics: self.metrics.clone(),
            connection_limits: self.connection_limits.clone(),
            rate_limiters: self.rate_limiters.clone(),
            threat_detection: self.threat_detection.clone(),
            security_config: SecurityConfig::default(),
        }
    }
}

pub struct SecurityStats {
    pub total_connections: u32,
    pub unique_ips: u32,
    pub rate_limited_ips: u32,
    pub suspicious_ips: u32,
    pub recent_attacks: u32,
    pub banned_ips: u32,
}

/// Helper function to get subnet key for IPv4/IPv6
fn get_subnet_key(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
        }
        IpAddr::V6(ipv6) => {
            let segments = ipv6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", segments[0], segments[1], segments[2], segments[3])
        }
    }
}
