//! Transaction format validator

use crate::{ValidationConfig, ValidationResult, ValidationError, Transaction};
use anyhow::Result;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("Empty transaction")]
    EmptyTransaction,
    
    #[error("Too many inputs: {count} > {max}")]
    TooManyInputs { count: usize, max: usize },
    
    #[error("Too many outputs: {count} > {max}")]
    TooManyOutputs { count: usize, max: usize },
    
    #[error("Transaction too large: {size} > {max}")]
    TransactionTooLarge { size: usize, max: usize },
}

pub struct FormatValidator {
    config: ValidationConfig,
}

impl FormatValidator {
    pub fn new(config: &ValidationConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn validate_transaction_format(&self, transaction: &Transaction) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        // Check for empty transaction
        if transaction.inputs.is_empty() && transaction.outputs.is_empty() {
            errors.push(ValidationError::EmptyTransaction);
        }

        // Check input count
        if transaction.inputs.len() > self.config.max_inputs_per_tx {
            errors.push(ValidationError::TooManyInputs {
                count: transaction.inputs.len(),
                max: self.config.max_inputs_per_tx,
            });
        }

        // Check output count  
        if transaction.outputs.len() > self.config.max_outputs_per_tx {
            errors.push(ValidationError::TooManyOutputs {
                count: transaction.outputs.len(),
                max: self.config.max_outputs_per_tx,
            });
        }

        // Check transaction size
        if let Ok(serialized) = bincode::serialize(transaction) {
            if serialized.len() > self.config.max_tx_size {
                errors.push(ValidationError::TransactionTooLarge {
                    size: serialized.len(),
                    max: self.config.max_tx_size,
                });
            }
        }

        if errors.is_empty() {
            Ok(ValidationResult::valid(0, 0, 0, 0, 0))
        } else {
            Ok(ValidationResult::invalid(errors))
        }
    }
}
