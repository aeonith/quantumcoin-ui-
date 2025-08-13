//! RevStop functionality for QuantumCoin wallet

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use std::path::Path;

/// RevStop errors
#[derive(thiserror::Error, Debug)]
pub enum RevStopError {
    /// Password verification failed
    #[error("Password verification failed")]
    InvalidPassword,
    
    /// RevStop file error
    #[error("RevStop file error: {0}")]
    FileError(String),
}

/// RevStop manager handles wallet freeze functionality
pub struct RevStopManager {
    revstop_file: std::path::PathBuf,
}

impl RevStopManager {
    /// Create new RevStop manager
    pub fn new(wallet_dir: &Path) -> Self {
        let revstop_file = wallet_dir.join("revstop.flag");
        Self { revstop_file }
    }
    
    /// Enable RevStop with password
    pub fn enable(&self, password: &str) -> Result<(), RevStopError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| RevStopError::FileError(e.to_string()))?;
        
        std::fs::write(&self.revstop_file, password_hash.to_string())
            .map_err(|e| RevStopError::FileError(e.to_string()))?;
        
        tracing::info!("RevStop enabled for wallet");
        Ok(())
    }
    
    /// Disable RevStop with password verification
    pub fn disable(&self, password: &str) -> Result<(), RevStopError> {
        if !self.is_enabled() {
            return Ok(()); // Already disabled
        }
        
        self.verify_password(password)?;
        
        std::fs::remove_file(&self.revstop_file)
            .map_err(|e| RevStopError::FileError(e.to_string()))?;
        
        tracing::info!("RevStop disabled for wallet");
        Ok(())
    }
    
    /// Check if RevStop is enabled
    pub fn is_enabled(&self) -> bool {
        self.revstop_file.exists()
    }
    
    /// Verify password against stored hash
    pub fn verify_password(&self, password: &str) -> Result<(), RevStopError> {
        if !self.is_enabled() {
            return Err(RevStopError::InvalidPassword);
        }
        
        let hash_string = std::fs::read_to_string(&self.revstop_file)
            .map_err(|e| RevStopError::FileError(e.to_string()))?;
        
        let parsed_hash = PasswordHash::new(&hash_string)
            .map_err(|e| RevStopError::FileError(e.to_string()))?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| RevStopError::InvalidPassword)?;
        
        Ok(())
    }
    
    /// Check if transaction is allowed (RevStop not enabled or password provided)
    pub fn allow_transaction(&self, password: Option<&str>) -> Result<(), RevStopError> {
        if !self.is_enabled() {
            return Ok(()); // RevStop not enabled, allow transaction
        }
        
        if let Some(pwd) = password {
            self.verify_password(pwd)?;
            Ok(())
        } else {
            Err(RevStopError::InvalidPassword)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_revstop_enable_disable() {
        let temp_dir = TempDir::new().unwrap();
        let manager = RevStopManager::new(temp_dir.path());
        
        assert!(!manager.is_enabled());
        
        let password = "test_password_123";
        manager.enable(password).unwrap();
        
        assert!(manager.is_enabled());
        
        // Should verify correct password
        assert!(manager.verify_password(password).is_ok());
        
        // Should reject wrong password
        assert!(manager.verify_password("wrong_password").is_err());
        
        // Disable with correct password
        manager.disable(password).unwrap();
        assert!(!manager.is_enabled());
    }
    
    #[test]
    fn test_transaction_allowance() {
        let temp_dir = TempDir::new().unwrap();
        let manager = RevStopManager::new(temp_dir.path());
        
        // Should allow when RevStop disabled
        assert!(manager.allow_transaction(None).is_ok());
        
        let password = "test_password_123";
        manager.enable(password).unwrap();
        
        // Should block when RevStop enabled without password
        assert!(manager.allow_transaction(None).is_err());
        
        // Should allow with correct password
        assert!(manager.allow_transaction(Some(password)).is_ok());
        
        // Should block with wrong password
        assert!(manager.allow_transaction(Some("wrong")).is_err());
    }
}
