//! Wallet storage management

/// Storage errors
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    /// File I/O error
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
