//! QuantumCoin Genesis Block System
//! 
//! This crate provides a production-grade, deterministic genesis block creation
//! system for QuantumCoin with post-quantum cryptographic security.

pub mod config;
pub mod block;
pub mod merkle;
pub mod crypto;
pub mod builder;
pub mod verification;

pub use builder::GenesisBuilder;
pub use config::ChainSpec;
pub use block::{GenesisBlock, BlockHeader};
pub use verification::GenesisVerifier;

use anyhow::Result;

/// Create the official mainnet genesis block
pub fn create_mainnet_genesis() -> Result<GenesisBlock> {
    let chain_spec = ChainSpec::load_mainnet()?;
    let builder = GenesisBuilder::new(chain_spec);
    builder.build()
}

/// Create a testnet genesis block
pub fn create_testnet_genesis() -> Result<GenesisBlock> {
    let chain_spec = ChainSpec::load_testnet()?;
    let builder = GenesisBuilder::new(chain_spec);
    builder.build()
}

/// Verify a genesis block against the chain specification
pub fn verify_genesis_block(block: &GenesisBlock, chain_spec: &ChainSpec) -> Result<bool> {
    let verifier = GenesisVerifier::new(chain_spec);
    verifier.verify(block)
}
