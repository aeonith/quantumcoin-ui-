use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCoinConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
    pub quantum: QuantumConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub client_shutdown: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub cache_size_mb: u32,
    pub wal_mode: bool,
    pub auto_vacuum: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub auth_token_expiry: u64,
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
    pub quantum_security_level: u8,
    pub enable_2fa: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_output: bool,
    pub file_path: String,
    pub max_file_size_mb: u32,
    pub max_files: u32,
    pub json_format: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_transactions_per_block: u32,
    pub target_block_time_ms: u64,
    pub max_pending_transactions: u32,
    pub cache_ttl_seconds: u64,
    pub enable_parallel_processing: bool,
    pub batch_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumConfig {
    pub enable_quantum_crypto: bool,
    pub security_level: u8,
    pub key_rotation_interval: u64,
    pub enable_quantum_entanglement: bool,
    pub quantum_consensus_threshold: f64,
}

impl Default for QuantumCoinConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: num_cpus::get(),
                keep_alive: 75,
                client_timeout: 5000,
                client_shutdown: 5000,
            },
            database: DatabaseConfig {
                path: "./data/quantumcoin.db".to_string(),
                max_connections: 32,
                connection_timeout: 5000,
                cache_size_mb: 128,
                wal_mode: true,
                auto_vacuum: true,
            },
            security: SecurityConfig {
                jwt_secret: "quantum-production-ultra-secure-key-2024".to_string(),
                auth_token_expiry: 3600, // 1 hour
                rate_limit_requests: 100,
                rate_limit_window: 60, // 1 minute
                quantum_security_level: 5,
                enable_2fa: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_output: true,
                file_path: "./logs/quantumcoin.log".to_string(),
                max_file_size_mb: 10,
                max_files: 5,
                json_format: false,
            },
            performance: PerformanceConfig {
                max_transactions_per_block: 10000,
                target_block_time_ms: 1000, // 1 second
                max_pending_transactions: 50000,
                cache_ttl_seconds: 300, // 5 minutes
                enable_parallel_processing: true,
                batch_size: 1000,
            },
            quantum: QuantumConfig {
                enable_quantum_crypto: true,
                security_level: 5,
                key_rotation_interval: 86400, // 24 hours
                enable_quantum_entanglement: true,
                quantum_consensus_threshold: 0.67,
            },
        }
    }
}

impl QuantumCoinConfig {
    pub fn load() -> Self {
        let mut config = Self::default();
        
        // Override with environment variables if present
        if let Ok(host) = env::var("QTC_SERVER_HOST") {
            config.server.host = host;
        }
        
        if let Ok(port) = env::var("QTC_SERVER_PORT") {
            if let Ok(p) = port.parse::<u16>() {
                config.server.port = p;
            }
        }
        
        if let Ok(db_path) = env::var("QTC_DATABASE_PATH") {
            config.database.path = db_path;
        }
        
        if let Ok(jwt_secret) = env::var("QTC_JWT_SECRET") {
            config.security.jwt_secret = jwt_secret;
        }
        
        if let Ok(log_level) = env::var("QTC_LOG_LEVEL") {
            config.logging.level = log_level;
        }
        
        if let Ok(workers) = env::var("QTC_WORKERS") {
            if let Ok(w) = workers.parse::<usize>() {
                config.server.workers = w;
            }
        }
        
        // Production environment detection
        if env::var("QTC_ENV").unwrap_or_default() == "production" {
            config.logging.level = "warn".to_string();
            config.security.enable_2fa = true;
            config.performance.enable_parallel_processing = true;
        }
        
        config
    }
    
    pub fn validate(&self) -> Result<(), String> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err("Invalid server port".to_string());
        }
        
        if self.server.workers == 0 {
            return Err("Invalid worker count".to_string());
        }
        
        // Validate security configuration
        if self.security.jwt_secret.len() < 32 {
            return Err("JWT secret too short (minimum 32 characters)".to_string());
        }
        
        if self.security.quantum_security_level > 5 {
            return Err("Invalid quantum security level (max 5)".to_string());
        }
        
        // Validate database configuration
        if self.database.max_connections == 0 {
            return Err("Invalid database connection count".to_string());
        }
        
        // Validate performance configuration
        if self.performance.max_transactions_per_block == 0 {
            return Err("Invalid max transactions per block".to_string());
        }
        
        if self.performance.target_block_time_ms == 0 {
            return Err("Invalid target block time".to_string());
        }
        
        Ok(())
    }
    
    pub fn is_production(&self) -> bool {
        env::var("QTC_ENV").unwrap_or_default() == "production"
    }
    
    pub fn get_bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
    
    pub fn get_database_url(&self) -> String {
        format!("sqlite://{}?mode=rwc", self.database.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = QuantumCoinConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_loading() {
        let config = QuantumCoinConfig::load();
        assert!(config.validate().is_ok());
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_production_detection() {
        let config = QuantumCoinConfig::load();
        // In test environment, should not be production
        assert!(!config.is_production());
    }
}
