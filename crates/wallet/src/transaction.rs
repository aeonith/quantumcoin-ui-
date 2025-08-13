//! Transaction building for wallet

/// Transaction building errors
#[derive(thiserror::Error, Debug)]
pub enum TransactionError {
    /// Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,
}
