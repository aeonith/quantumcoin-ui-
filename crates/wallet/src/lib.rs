//! # QuantumCoin Wallet
//!
//! Post-quantum cryptographic wallet implementation for QuantumCoin.
//! Uses Dilithium2 signatures for quantum resistance.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]

pub mod crypto;
pub mod keys;
pub mod revstop;
pub mod storage;
pub mod transaction;
pub mod wallet;

pub use wallet::{Wallet, WalletConfig, WalletError};
pub use keys::{KeyPair, PublicKey, SecretKey};
pub use revstop::RevStopManager;

/// Wallet-specific error types
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Cryptographic error
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] crypto::CryptoError),
    
    /// Key management error
    #[error("Key error: {0}")]
    Key(#[from] keys::KeyError),
    
    /// RevStop error
    #[error("RevStop error: {0}")]
    RevStop(#[from] revstop::RevStopError),
    
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),
    
    /// Transaction building error
    #[error("Transaction error: {0}")]
    Transaction(#[from] transaction::TransactionError),
}

/// Result type for wallet operations
pub type Result<T> = std::result::Result<T, Error>;
