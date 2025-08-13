//! Key management for QuantumCoin wallet

use crate::crypto::{self, CryptoError};

/// Key management errors
#[derive(thiserror::Error, Debug)]
pub enum KeyError {
    /// Invalid key data
    #[error("Invalid key data: {0}")]
    InvalidKey(String),
}

/// Public key wrapper
#[derive(Debug, Clone)]
pub struct PublicKey(Vec<u8>);

/// Secret key wrapper  
#[derive(Debug, Clone)]
pub struct SecretKey(Vec<u8>);

/// Key pair containing both public and secret keys
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// Public key
    pub public: PublicKey,
    /// Secret key (sensitive)
    pub secret: SecretKey,
}

impl KeyPair {
    /// Generate new key pair
    pub fn generate() -> Result<Self, KeyError> {
        let (pk_bytes, sk_bytes) = crypto::generate_keypair()
            .map_err(|e| KeyError::InvalidKey(e.to_string()))?;
        
        Ok(KeyPair {
            public: PublicKey(pk_bytes),
            secret: SecretKey(sk_bytes),
        })
    }
    
    /// Get address from public key
    pub fn address(&self) -> String {
        crypto::public_key_to_address(&self.public.0)
    }
}

impl PublicKey {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        PublicKey(bytes)
    }
    
    /// Get bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl SecretKey {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        SecretKey(bytes)
    }
    
    /// Get bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
