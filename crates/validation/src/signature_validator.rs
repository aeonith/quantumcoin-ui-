//! Post-quantum signature validator using Dilithium2

use anyhow::Result;
use pqcrypto_dilithium::dilithium2::*;
use pqcrypto_traits::sign::{PublicKey, DetachedSignature};
use blake3::Hasher;
use std::collections::HashMap;
use parking_lot::RwLock;
use tracing::{debug, error};

#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("Invalid public key format")]
    InvalidPublicKey,
    
    #[error("Invalid signature format")]
    InvalidSignature,
    
    #[error("Signature verification failed")]
    VerificationFailed,
    
    #[error("Missing signature data")]
    MissingSignature,
    
    #[error("Unsupported signature algorithm")]
    UnsupportedAlgorithm,
}

/// Production post-quantum signature validator
pub struct SignatureValidator {
    // Performance cache for public keys
    pubkey_cache: RwLock<HashMap<Vec<u8>, PublicKey>>,
    verification_stats: RwLock<SignatureStats>,
}

#[derive(Debug, Clone, Default)]
struct SignatureStats {
    total_verifications: u64,
    successful_verifications: u64,
    failed_verifications: u64,
    cache_hits: u64,
    avg_verification_time_us: f64,
}

impl SignatureValidator {
    pub fn new() -> Self {
        Self {
            pubkey_cache: RwLock::new(HashMap::with_capacity(1000)),
            verification_stats: RwLock::new(SignatureStats::default()),
        }
    }

    /// Verify Dilithium2 signature for transaction input
    pub fn verify_dilithium_signature(
        &self,
        script_sig: &[u8],
        signature_hash: &[u8; 32],
        input_index: usize,
    ) -> Result<bool, SignatureError> {
        let start_time = std::time::Instant::now();

        // Parse script_sig to extract signature and public key
        let (signature, public_key) = self.parse_script_sig(script_sig)?;

        // Verify the signature
        let is_valid = self.verify_signature(&signature, &public_key, signature_hash)?;

        // Update statistics
        let verification_time = start_time.elapsed().as_micros() as f64;
        {
            let mut stats = self.verification_stats.write();
            stats.total_verifications += 1;
            if is_valid {
                stats.successful_verifications += 1;
            } else {
                stats.failed_verifications += 1;
            }
            
            let total_time = stats.avg_verification_time_us * (stats.total_verifications - 1) as f64;
            stats.avg_verification_time_us = (total_time + verification_time) / stats.total_verifications as f64;
        }

        debug!("Signature verification for input {} completed in {:.2}μs: {}", 
               input_index, verification_time, if is_valid { "VALID" } else { "INVALID" });

        Ok(is_valid)
    }

    /// Parse script signature to extract signature and public key components
    fn parse_script_sig(&self, script_sig: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SignatureError> {
        // Minimum size check: 2 bytes (sig_len) + minimum sig + 2 bytes (pubkey_len) + minimum pubkey
        if script_sig.len() < 8 {
            return Err(SignatureError::MissingSignature);
        }

        // QuantumCoin script format: [sig_len][signature][pubkey_len][public_key]  
        let mut offset = 0;

        // Read signature length and signature - with bounds checking
        if offset + 2 > script_sig.len() {
            return Err(SignatureError::InvalidSignature);
        }
        
        let sig_len = u16::from_le_bytes([script_sig[offset], script_sig[offset + 1]]) as usize;
        offset += 2;

        // Validate signature length is reasonable (prevent memory exhaustion)
        if sig_len == 0 || sig_len > 10000 {
            return Err(SignatureError::InvalidSignature);
        }

        if offset + sig_len > script_sig.len() {
            return Err(SignatureError::InvalidSignature);
        }
        
        let signature = script_sig[offset..offset + sig_len].to_vec();
        offset += sig_len;

        // Read public key length and public key - with bounds checking
        if offset + 2 > script_sig.len() {
            return Err(SignatureError::InvalidPublicKey);
        }
        
        let pubkey_len = u16::from_le_bytes([script_sig[offset], script_sig[offset + 1]]) as usize;
        offset += 2;

        // Validate public key length is reasonable (prevent memory exhaustion)
        if pubkey_len == 0 || pubkey_len > 10000 {
            return Err(SignatureError::InvalidPublicKey);
        }

        if offset + pubkey_len > script_sig.len() {
            return Err(SignatureError::InvalidPublicKey);
        }
        
        let public_key = script_sig[offset..offset + pubkey_len].to_vec();

        // Validate expected sizes for Dilithium2 (security requirement)
        if signature.len() != SIGNATUREBYTES {
            return Err(SignatureError::InvalidSignature);
        }
        if public_key.len() != PUBLICKEYBYTES {
            return Err(SignatureError::InvalidPublicKey);
        }

        Ok((signature, public_key))
    }

    /// Verify signature using Dilithium2
    fn verify_signature(
        &self,
        signature: &[u8],
        public_key_bytes: &[u8],
        message: &[u8; 32],
    ) -> Result<bool, SignatureError> {
        // Check cache first
        let cache_key = public_key_bytes.to_vec();
        let public_key = {
            let mut cache = self.pubkey_cache.write();
            if let Some(pk) = cache.get(&cache_key) {
                let mut stats = self.verification_stats.write();
                stats.cache_hits += 1;
                pk.clone()
            } else {
                // Parse public key
                match PublicKey::from_bytes(public_key_bytes) {
                    Ok(pk) => {
                        // Cache the parsed public key
                        if cache.len() < 1000 {
                            cache.insert(cache_key, pk.clone());
                        }
                        pk
                    }
                    Err(_) => return Err(SignatureError::InvalidPublicKey),
                }
            }
        };

        // Create detached signature
        let detached_sig = match DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => return Err(SignatureError::InvalidSignature),
        };

        // Verify the signature
        match detached_sig.verify(message, &public_key) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false), // Signature verification failed, but not an error condition
        }
    }

    /// Create a script signature for a transaction input
    pub fn create_script_sig(
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<Vec<u8>, SignatureError> {
        if signature.len() != SIGNATUREBYTES {
            return Err(SignatureError::InvalidSignature);
        }
        if public_key.len() != PUBLICKEYBYTES {
            return Err(SignatureError::InvalidPublicKey);
        }

        let mut script_sig = Vec::with_capacity(4 + signature.len() + public_key.len());
        
        // Add signature length and signature
        script_sig.extend_from_slice(&(signature.len() as u16).to_le_bytes());
        script_sig.extend_from_slice(signature);
        
        // Add public key length and public key
        script_sig.extend_from_slice(&(public_key.len() as u16).to_le_bytes());
        script_sig.extend_from_slice(public_key);
        
        Ok(script_sig)
    }

    /// Get signature verification statistics
    pub fn get_statistics(&self) -> SignatureStats {
        self.verification_stats.read().clone()
    }

    /// Clear caches (for testing)
    pub fn clear_cache(&self) {
        self.pubkey_cache.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pqcrypto_dilithium::dilithium2::*;

    #[test]
    fn test_signature_creation_and_verification() {
        let validator = SignatureValidator::new();
        
        // Generate test key pair
        let (pk, sk) = keypair();
        let message = b"test message for signature verification";
        
        // Create signature
        let signature = detached_sign(message, &sk);
        
        // Create script signature
        let script_sig = SignatureValidator::create_script_sig(
            signature.as_bytes(),
            pk.as_bytes(),
        ).unwrap();
        
        // Create message hash
        let mut hasher = Hasher::new();
        hasher.update(message);
        let message_hash = *hasher.finalize().as_bytes();
        
        // Verify signature
        let is_valid = validator.verify_dilithium_signature(
            &script_sig,
            &message_hash,
            0,
        ).unwrap();
        
        assert!(is_valid, "Valid signature should verify successfully");
    }

    #[test]
    fn test_invalid_signature_rejection() {
        let validator = SignatureValidator::new();
        
        // Generate test key pair
        let (pk, sk) = keypair();
        let message1 = b"original message";
        let message2 = b"different message";
        
        // Create signature for message1
        let signature = detached_sign(message1, &sk);
        let script_sig = SignatureValidator::create_script_sig(
            signature.as_bytes(),
            pk.as_bytes(),
        ).unwrap();
        
        // Try to verify against message2 (should fail)
        let mut hasher = Hasher::new();
        hasher.update(message2);
        let message_hash = *hasher.finalize().as_bytes();
        
        let is_valid = validator.verify_dilithium_signature(
            &script_sig,
            &message_hash,
            0,
        ).unwrap();
        
        assert!(!is_valid, "Invalid signature should be rejected");
    }

    #[test]
    fn test_malformed_script_sig() {
        let validator = SignatureValidator::new();
        let malformed_script = vec![0x01, 0x02, 0x03]; // Too short
        let message_hash = [0u8; 32];
        
        let result = validator.verify_dilithium_signature(
            &malformed_script,
            &message_hash,
            0,
        );
        
        assert!(result.is_err(), "Malformed script should return error");
    }

    #[test]
    fn test_signature_caching() {
        let validator = SignatureValidator::new();
        
        // Generate test data
        let (pk, sk) = keypair();
        let message = b"test message";
        let signature = detached_sign(message, &sk);
        let script_sig = SignatureValidator::create_script_sig(
            signature.as_bytes(),
            pk.as_bytes(),
        ).unwrap();
        
        let mut hasher = Hasher::new();
        hasher.update(message);
        let message_hash = *hasher.finalize().as_bytes();
        
        // First verification (should cache the public key)
        let is_valid1 = validator.verify_dilithium_signature(
            &script_sig,
            &message_hash,
            0,
        ).unwrap();
        
        // Second verification (should hit cache)
        let is_valid2 = validator.verify_dilithium_signature(
            &script_sig,
            &message_hash,
            0,
        ).unwrap();
        
        assert!(is_valid1 && is_valid2);
        
        let stats = validator.get_statistics();
        assert!(stats.cache_hits > 0, "Should have cache hits");
        assert_eq!(stats.total_verifications, 2);
    }

    #[test]
    fn test_performance_statistics() {
        let validator = SignatureValidator::new();
        
        // Generate test data
        let (pk, sk) = keypair();
        let signature = detached_sign(b"test", &sk);
        let script_sig = SignatureValidator::create_script_sig(
            signature.as_bytes(),
            pk.as_bytes(),
        ).unwrap();
        
        let message_hash = [0u8; 32];
        
        // Perform multiple verifications
        for _ in 0..5 {
            validator.verify_dilithium_signature(&script_sig, &message_hash, 0).unwrap();
        }
        
        let stats = validator.get_statistics();
        assert_eq!(stats.total_verifications, 5);
        assert!(stats.avg_verification_time_us > 0.0);
        assert_eq!(stats.successful_verifications, 5);
        assert_eq!(stats.failed_verifications, 0);
    }
}
