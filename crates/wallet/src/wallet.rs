//! Main wallet implementation

use crate::{KeyPair, RevStopManager};
use std::path::PathBuf;

/// Wallet errors
#[derive(thiserror::Error, Debug)]
pub enum WalletError {
    /// Key error
    #[error("Key error: {0}")]
    KeyError(String),
}

/// Wallet configuration
#[derive(Debug, Clone)]
pub struct WalletConfig {
    /// Wallet data directory
    pub data_dir: PathBuf,
}

/// Main wallet struct
pub struct Wallet {
    /// Wallet configuration
    pub config: WalletConfig,
    
    /// Key pair
    pub keypair: KeyPair,
    
    /// RevStop manager
    pub revstop: RevStopManager,
}

impl Wallet {
    /// Create new wallet
    pub fn new(config: WalletConfig) -> Result<Self, WalletError> {
        let keypair = KeyPair::generate()
            .map_err(|e| WalletError::KeyError(e.to_string()))?;
        
        let revstop = RevStopManager::new(&config.data_dir);
        
        Ok(Wallet {
            config,
            keypair,
            revstop,
        })
    }
    
    /// Get wallet address
    pub fn address(&self) -> String {
        self.keypair.address()
    }
}
