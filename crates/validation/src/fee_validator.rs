//! Fee validation with market-based policies

use crate::*;
use anyhow::Result;

#[derive(Debug, thiserror::Error)]
pub enum FeeError {
    #[error("Insufficient fee: {actual} < {required}")]
    InsufficientFee { actual: u64, required: u64 },
    
    #[error("Fee too high: {actual} > {max}")]
    FeeTooHigh { actual: u64, max: u64 },
}

#[derive(Debug, Clone)]
pub struct FeePolicy {
    pub min_relay_fee: u64,
    pub dust_threshold: u64, 
    pub max_fee_rate: u64,
}

impl FeePolicy {
    pub fn from_config(config: &ValidationConfig) -> Self {
        Self {
            min_relay_fee: config.min_tx_fee,
            dust_threshold: config.dust_threshold,
            max_fee_rate: config.max_fee_rate,
        }
    }
}

pub struct FeeValidator {
    policy: FeePolicy,
}

impl FeeValidator {
    pub fn new(policy: FeePolicy) -> Self {
        Self { policy }
    }

    pub async fn validate_transaction_fees(
        &self,
        transaction: &Transaction,
        utxo_set: &UtxoSet,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        
        // Calculate total input value
        let mut total_input_value = 0u64;
        for input in &transaction.inputs {
            if let Some(utxo) = utxo_set.get_utxo(&input.previous_output) {
                total_input_value = total_input_value.saturating_add(utxo.output.value);
            }
        }

        // Calculate total output value
        let total_output_value: u64 = transaction.outputs.iter().map(|o| o.value).sum();
        
        // Calculate fee
        if total_input_value < total_output_value {
            errors.push(ValidationError::NegativeFee);
        } else {
            let fee = total_input_value - total_output_value;
            
            // Check minimum fee
            if fee < self.policy.min_relay_fee {
                errors.push(ValidationError::InsufficientFee {
                    actual: fee,
                    required: self.policy.min_relay_fee,
                });
            }

            // Check dust outputs
            for output in &transaction.outputs {
                if output.value < self.policy.dust_threshold {
                    errors.push(ValidationError::DustOutput {
                        value: output.value,
                        threshold: self.policy.dust_threshold,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(ValidationResult::valid(total_input_value, total_output_value, 0, 0, 0))
        } else {
            Ok(ValidationResult::invalid(errors))
        }
    }
}
