use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use argon2::{Argon2, password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}};
use rand_core::OsRng;
use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, DetachedSignature, PublicKey, SecretKey};
use pqcrypto_traits::sign::Verifier;
use base64::{encode, decode};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum RevStopError {
    #[error("Invalid passphrase")]
    InvalidPassphrase,
    #[error("Wallet already locked")]
    AlreadyLocked,
    #[error("Wallet not locked")]
    NotLocked,
    #[error("Lock expired")]
    LockExpired,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevStopLock {
    pub wallet_address: String,
    pub lock_hash: String, // Argon2 hash of passphrase
    pub locked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub lock_signature: String, // Dilithium signature
    pub emergency_unlock_hash: Option<String>, // Emergency passphrase hash
    pub unlock_attempts: u32,
    pub max_attempts: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevStopRegistry {
    pub locks: HashMap<String, RevStopLock>,
    pub global_settings: RevStopSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevStopSettings {
    pub max_lock_duration_hours: u64,
    pub default_lock_duration_hours: u64,
    pub max_unlock_attempts: u32,
    pub rate_limit_window_minutes: u64,
    pub quantum_resistance_level: u8,
}

impl Default for RevStopSettings {
    fn default() -> Self {
        Self {
            max_lock_duration_hours: 168, // 1 week
            default_lock_duration_hours: 24, // 1 day
            max_unlock_attempts: 5,
            rate_limit_window_minutes: 60,
            quantum_resistance_level: 3, // High security
        }
    }
}

pub struct RevStop {
    registry: RevStopRegistry,
    argon2: Argon2<'static>,
}

impl RevStop {
    pub fn new() -> Self {
        Self {
            registry: RevStopRegistry {
                locks: HashMap::new(),
                global_settings: RevStopSettings::default(),
            },
            argon2: Argon2::default(),
        }
    }

    pub fn create_lock(
        &mut self,
        wallet_address: &str,
        passphrase: &str,
        emergency_passphrase: Option<&str>,
        duration_hours: Option<u64>,
        private_key: &SecretKey,
    ) -> Result<String, RevStopError> {
        // Check if already locked
        if self.is_locked(wallet_address) {
            return Err(RevStopError::AlreadyLocked);
        }

        // Hash the passphrase with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let lock_hash = self.argon2
            .hash_password(passphrase.as_bytes(), &salt)
            .map_err(|_| RevStopError::InvalidPassphrase)?
            .to_string();

        // Hash emergency passphrase if provided
        let emergency_unlock_hash = if let Some(emergency) = emergency_passphrase {
            let emergency_salt = SaltString::generate(&mut OsRng);
            Some(self.argon2
                .hash_password(emergency.as_bytes(), &emergency_salt)
                .map_err(|_| RevStopError::InvalidPassphrase)?
                .to_string())
        } else {
            None
        };

        let duration = duration_hours.unwrap_or(self.registry.global_settings.default_lock_duration_hours);
        let locked_at = Utc::now();
        let expires_at = if duration > 0 {
            Some(locked_at + Duration::hours(duration as i64))
        } else {
            None
        };

        // Create signature for lock authenticity
        let lock_message = format!("REVSTOP_LOCK:{}:{}:{:?}", 
            wallet_address, locked_at.timestamp(), expires_at.map(|e| e.timestamp()));
        let signature = sign_detached(lock_message.as_bytes(), private_key);
        let lock_signature = encode(signature.as_bytes());

        let lock = RevStopLock {
            wallet_address: wallet_address.to_string(),
            lock_hash,
            locked_at,
            expires_at,
            lock_signature,
            emergency_unlock_hash,
            unlock_attempts: 0,
            max_attempts: self.registry.global_settings.max_unlock_attempts,
        };

        let lock_id = Uuid::new_v4().to_string();
        self.registry.locks.insert(wallet_address.to_string(), lock);
        
        Ok(lock_id)
    }

    pub fn unlock(
        &mut self,
        wallet_address: &str,
        passphrase: &str,
        public_key: &PublicKey,
    ) -> Result<bool, RevStopError> {
        let mut lock = self.registry.locks
            .get(wallet_address)
            .ok_or(RevStopError::NotLocked)?
            .clone();

        // Check if lock has expired
        if let Some(expires_at) = lock.expires_at {
            if Utc::now() > expires_at {
                self.registry.locks.remove(wallet_address);
                return Err(RevStopError::LockExpired);
            }
        }

        // Check unlock attempts
        if lock.unlock_attempts >= lock.max_attempts {
            return Err(RevStopError::RateLimitExceeded);
        }

        // Verify lock signature
        let lock_message = format!("REVSTOP_LOCK:{}:{}:{:?}", 
            wallet_address, lock.locked_at.timestamp(), 
            lock.expires_at.map(|e| e.timestamp()));
        
        let signature_bytes = decode(&lock.lock_signature)
            .map_err(|_| RevStopError::InvalidSignature)?;
        let signature = DetachedSignature::from_bytes(&signature_bytes)
            .map_err(|_| RevStopError::InvalidSignature)?;

        if signature.verify_detached(lock_message.as_bytes(), public_key).is_err() {
            return Err(RevStopError::InvalidSignature);
        }

        // Verify passphrase
        let parsed_hash = PasswordHash::new(&lock.lock_hash)
            .map_err(|_| RevStopError::InvalidPassphrase)?;
        
        if self.argon2.verify_password(passphrase.as_bytes(), &parsed_hash).is_ok() {
            // Successful unlock
            self.registry.locks.remove(wallet_address);
            Ok(true)
        } else {
            // Try emergency passphrase if available
            if let Some(emergency_hash) = &lock.emergency_unlock_hash {
                let emergency_parsed = PasswordHash::new(emergency_hash)
                    .map_err(|_| RevStopError::InvalidPassphrase)?;
                
                if self.argon2.verify_password(passphrase.as_bytes(), &emergency_parsed).is_ok() {
                    self.registry.locks.remove(wallet_address);
                    return Ok(true);
                }
            }

            // Failed attempt
            lock.unlock_attempts += 1;
            self.registry.locks.insert(wallet_address.to_string(), lock);
            Err(RevStopError::InvalidPassphrase)
        }
    }

    pub fn is_locked(&self, wallet_address: &str) -> bool {
        if let Some(lock) = self.registry.locks.get(wallet_address) {
            // Check if expired
            if let Some(expires_at) = lock.expires_at {
                Utc::now() <= expires_at
            } else {
                true // No expiration
            }
        } else {
            false
        }
    }

    pub fn get_lock_info(&self, wallet_address: &str) -> Option<&RevStopLock> {
        self.registry.locks.get(wallet_address)
    }

    pub fn force_unlock_emergency(&mut self, wallet_address: &str, admin_signature: &str) -> Result<bool, RevStopError> {
        // This would require admin-level access with proper cryptographic verification
        // Implementation would include multi-signature requirements for emergency unlocks
        if self.verify_admin_signature(admin_signature) {
            self.registry.locks.remove(wallet_address);
            Ok(true)
        } else {
            Err(RevStopError::InvalidSignature)
        }
    }

    fn verify_admin_signature(&self, _signature: &str) -> bool {
        // Placeholder for admin signature verification
        // Would require proper multi-sig implementation
        false
    }

    pub fn cleanup_expired_locks(&mut self) {
        let now = Utc::now();
        self.registry.locks.retain(|_, lock| {
            if let Some(expires_at) = lock.expires_at {
                now <= expires_at
            } else {
                true
            }
        });
    }

    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_locks".to_string(), self.registry.locks.len() as u64);
        stats.insert("active_locks".to_string(), 
            self.registry.locks.values()
                .filter(|lock| self.is_locked(&lock.wallet_address))
                .count() as u64);
        stats
    }
}