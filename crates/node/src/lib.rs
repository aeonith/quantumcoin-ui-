//! # QuantumCoin Node
//!
//! Core blockchain node implementation for QuantumCoin.
//! This crate provides the fundamental building blocks for the QuantumCoin blockchain,
//! including consensus logic, transaction validation, and block production.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]

pub mod block;
pub mod config;
pub mod consensus;
pub mod consensus_engine;
pub mod chain_spec_loader;
pub mod economics;
pub mod mempool;
pub mod network;
pub mod storage;
pub mod transaction;
pub mod validation;

// Include comprehensive tests
#[cfg(test)]
pub mod consensus_tests;

pub use config::ChainConfig;
pub use economics::{Economics, IssuanceSchedule};

/// Node-specific error types
#[derive(thiserror::Error, Debug)]
pub enum NodeError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    /// Block validation error
    #[error("Block validation error: {0}")]
    Block(#[from] block::BlockError),
    
    /// Transaction validation error
    #[error("Transaction validation error: {0}")]  
    Transaction(#[from] transaction::TransactionError),
    
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),
    
    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] network::NetworkError),
}

/// Result type for node operations
pub type NodeResult<T> = Result<T, NodeError>;
