//! Configuration management for QuantumCoin node
//! 
//! This module provides the canonical source of truth for all QuantumCoin 
//! economic and network parameters. All other components must use these
//! values instead of hardcoding constants.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Configuration errors
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    /// Failed to read configuration file
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    
    /// Failed to parse configuration
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    
    /// Invalid configuration value
    #[error("Invalid config value: {0}")]
    Invalid(String),
}

/// Economics configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicsConfig {
    /// Total supply cap in QTC
    pub total_supply: u64,
    
    /// Halving period in years
    pub halving_period_years: u32,
    
    /// Total halving duration in years  
    pub halving_duration_years: u32,
    
    /// Target block time in seconds
    pub block_time_target_sec: u32,
    
    /// Genesis premine amount
    pub genesis_premine_qtc: u64,
    
    /// Development fund amount
    pub dev_fund_qtc: u64,
}

impl EconomicsConfig {
    /// Validate economics configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.total_supply == 0 {
            return Err(ConfigError::Invalid("total_supply cannot be zero".to_string()));
        }
        
        if self.halving_period_years == 0 {
            return Err(ConfigError::Invalid("halving_period_years cannot be zero".to_string()));
        }
        
        if self.block_time_target_sec == 0 {
            return Err(ConfigError::Invalid("block_time_target_sec cannot be zero".to_string()));
        }
        
        if self.genesis_premine_qtc + self.dev_fund_qtc > self.total_supply {
            return Err(ConfigError::Invalid("premine + dev fund exceeds total supply".to_string()));
        }
        
        Ok(())
    }
}

/// Network configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Chain identifier
    pub chain_id: String,
    
    /// Network magic bytes
    pub network_magic: u32,
    
    /// P2P port
    pub p2p_port: u16,
    
    /// RPC port  
    pub rpc_port: u16,
    
    /// Explorer API port
    pub explorer_port: u16,
}

/// Proof of Work configuration  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowConfig {
    /// Difficulty adjustment period in blocks
    pub difficulty_adjustment_blocks: u32,
    
    /// Target timespan for difficulty adjustment
    pub target_timespan_seconds: u32,
}

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// Whether RevStop is enabled
    pub revstop_enabled: bool,
    
    /// Whether RevStop is on by default (MUST be false for exchanges)
    pub revstop_default_on: bool,
}

/// Complete chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Economics parameters
    pub economics: EconomicsConfig,
    
    /// Network parameters  
    pub network: NetworkConfig,
    
    /// Proof of Work parameters
    pub pow: PowConfig,
    
    /// Feature flags
    pub features: FeaturesConfig,
}

impl ChainConfig {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: ChainConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Load default configuration (canonical values)
    pub fn default() -> Self {
        ChainConfig {
            economics: EconomicsConfig {
                total_supply: 22_000_000,
                halving_period_years: 2,
                halving_duration_years: 66,
                block_time_target_sec: 600,
                genesis_premine_qtc: 1_250_000,
                dev_fund_qtc: 250_000,
            },
            network: NetworkConfig {
                chain_id: "quantumcoin-mainnet-v2".to_string(),
                network_magic: 0x51434D4E, // "QTCM" in hex
                p2p_port: 9333,
                rpc_port: 9332,
                explorer_port: 8080,
            },
            pow: PowConfig {
                difficulty_adjustment_blocks: 2016,
                target_timespan_seconds: 1_209_600, // 2 weeks
            },
            features: FeaturesConfig {
                revstop_enabled: true,
                revstop_default_on: false, // CRITICAL: must be false for exchanges
            },
        }
    }
    
    /// Validate the entire configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.economics.validate()?;
        
        // Validate network config
        if self.network.chain_id.is_empty() {
            return Err(ConfigError::Invalid("chain_id cannot be empty".to_string()));
        }
        
        // Validate PoW config
        if self.pow.difficulty_adjustment_blocks == 0 {
            return Err(ConfigError::Invalid("difficulty_adjustment_blocks cannot be zero".to_string()));
        }
        
        if self.pow.target_timespan_seconds == 0 {
            return Err(ConfigError::Invalid("target_timespan_seconds cannot be zero".to_string()));
        }
        
        // Validate features - enforce RevStop safety
        if self.features.revstop_enabled && self.features.revstop_default_on {
            tracing::warn!(
                "RevStop is enabled by default! This should be FALSE for exchange integrations."
            );
        }
        
        Ok(())
    }
    
    /// Get thread-safe configuration handle
    pub fn shared(self) -> SharedConfig {
        SharedConfig(Arc::new(self))
    }
}

/// Thread-safe shared configuration
#[derive(Debug, Clone)]
pub struct SharedConfig(Arc<ChainConfig>);

impl std::ops::Deref for SharedConfig {
    type Target = ChainConfig;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SharedConfig {
    /// Get reference to economics configuration
    pub fn economics(&self) -> &EconomicsConfig {
        &self.0.economics
    }
    
    /// Get reference to network configuration  
    pub fn network(&self) -> &NetworkConfig {
        &self.0.network
    }
    
    /// Get reference to PoW configuration
    pub fn pow(&self) -> &PowConfig {
        &self.0.pow
    }
    
    /// Get reference to features configuration
    pub fn features(&self) -> &FeaturesConfig {
        &self.0.features
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_validation() {
        let config = ChainConfig::default();
        config.validate().expect("Default config should be valid");
        
        // Verify critical economics constants
        assert_eq!(config.economics.total_supply, 22_000_000);
        assert_eq!(config.economics.halving_period_years, 2);
        assert_eq!(config.economics.halving_duration_years, 66);
        assert_eq!(config.economics.block_time_target_sec, 600);
        
        // Verify RevStop safety
        assert!(!config.features.revstop_default_on, "RevStop MUST be off by default");
    }
    
    #[test]
    fn test_invalid_config_detection() {
        let mut config = ChainConfig::default();
        
        // Test zero total supply
        config.economics.total_supply = 0;
        assert!(config.validate().is_err());
        
        // Reset and test premine overflow
        config.economics.total_supply = 1000;
        config.economics.genesis_premine_qtc = 800;
        config.economics.dev_fund_qtc = 300; // 800 + 300 > 1000
        assert!(config.validate().is_err());
    }
}
