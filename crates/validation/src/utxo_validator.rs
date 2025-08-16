//! UTXO validator with double-spend prevention

use crate::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum SpendError {
    #[error("UTXO not found: {txid}:{vout}")]
    UtxoNotFound { txid: String, vout: u32 },
    
    #[error("UTXO already spent")]
    AlreadySpent,
    
    #[error("Coinbase not mature")]
    CoinbaseNotMature,
}

/// UTXO set for validation
#[derive(Debug, Clone)]
pub struct UtxoSet {
    utxos: Arc<RwLock<HashMap<OutPoint, UtxoEntry>>>,
    spent_utxos: Arc<RwLock<HashSet<OutPoint>>>,
}

impl UtxoSet {
    pub fn new() -> Self {
        Self {
            utxos: Arc::new(RwLock::new(HashMap::new())),
            spent_utxos: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<UtxoEntry> {
        self.utxos.read().get(outpoint).cloned()
    }

    pub fn is_spent(&self, outpoint: &OutPoint) -> bool {
        self.spent_utxos.read().contains(outpoint)
    }

    pub fn add_utxo(&self, outpoint: OutPoint, entry: UtxoEntry) {
        self.utxos.write().insert(outpoint, entry);
    }

    pub fn mark_spent(&self, outpoint: &OutPoint) {
        self.spent_utxos.write().insert(*outpoint);
    }

    pub fn remove_utxo(&self, outpoint: &OutPoint) -> Option<UtxoEntry> {
        self.utxos.write().remove(outpoint)
    }

    pub fn len(&self) -> usize {
        self.utxos.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.utxos.read().is_empty()
    }
}

pub struct UtxoValidator {
    config: ValidationConfig,
}

impl UtxoValidator {
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    pub fn new_with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub async fn validate_inputs(
        &self,
        transaction: &Transaction,
        utxo_set: &UtxoSet,
        current_height: u32,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut total_input_value = 0u64;
        
        // Check each input
        for (input_index, input) in transaction.inputs.iter().enumerate() {
            match utxo_set.get_utxo(&input.previous_output) {
                Some(utxo_entry) => {
                    // Check if already spent (double-spend protection)
                    if utxo_set.is_spent(&input.previous_output) {
                        errors.push(ValidationError::DoubleSpend {
                            txid: hex::encode(input.previous_output.txid),
                            vout: input.previous_output.vout,
                        });
                        continue;
                    }

                    // Check coinbase maturity
                    if utxo_entry.is_coinbase {
                        let confirmations = current_height.saturating_sub(utxo_entry.block_height);
                        if confirmations < self.config.coinbase_maturity {
                            errors.push(ValidationError::CoinbaseNotMature {
                                height: utxo_entry.block_height,
                                required: self.config.coinbase_maturity,
                            });
                            continue;
                        }
                    }

                    // Add to total input value with overflow protection
                    match total_input_value.checked_add(utxo_entry.output.value) {
                        Some(new_total) => total_input_value = new_total,
                        None => {
                            errors.push(ValidationError::InputValueOverflow);
                            continue;
                        }
                    }
                }
                None => {
                    errors.push(ValidationError::UtxoNotFound {
                        txid: hex::encode(input.previous_output.txid),
                        vout: input.previous_output.vout,
                    });
                }
            }
        }

        // Calculate total output value with overflow protection
        let mut total_output_value = 0u64;
        for output in &transaction.outputs {
            match total_output_value.checked_add(output.value) {
                Some(new_total) => total_output_value = new_total,
                None => {
                    errors.push(ValidationError::OutputValueOverflow);
                    break;
                }
            }
        }

        // Calculate fee
        let fee = if total_input_value >= total_output_value {
            total_input_value - total_output_value
        } else {
            errors.push(ValidationError::NegativeFee);
            0
        };

        if errors.is_empty() {
            Ok(ValidationResult::valid(
                total_input_value,
                total_output_value,
                fee,
                0, // size will be calculated elsewhere
                0, // weight will be calculated elsewhere
            ))
        } else {
            Ok(ValidationResult::invalid(errors))
        }
    }

    /// Validate that a transaction doesn't create any invalid UTXOs
    pub fn validate_outputs(&self, transaction: &Transaction) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        for (output_index, output) in transaction.outputs.iter().enumerate() {
            // Check for zero-value outputs (except OP_RETURN)
            if output.value == 0 && !self.is_op_return(&output.script_pubkey) {
                errors.push(ValidationError::DustOutput {
                    value: output.value,
                    threshold: self.config.dust_threshold,
                });
            }

            // Check for dust outputs
            if output.value > 0 && output.value < self.config.dust_threshold {
                errors.push(ValidationError::DustOutput {
                    value: output.value,
                    threshold: self.config.dust_threshold,
                });
            }

            // Validate script format
            if output.script_pubkey.len() > 10000 {
                errors.push(ValidationError::InvalidFormat(
                    format!("Output {} script too large: {} bytes", output_index, output.script_pubkey.len())
                ));
            }
        }

        if errors.is_empty() {
            Ok(ValidationResult::valid(0, 0, 0, 0, 0))
        } else {
            Ok(ValidationResult::invalid(errors))
        }
    }

    /// Check if script is OP_RETURN (data carrying transaction)
    fn is_op_return(&self, script: &[u8]) -> bool {
        !script.is_empty() && script[0] == 0x6a // OP_RETURN opcode
    }
}

impl Default for UtxoSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for UtxoValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_set_operations() {
        let utxo_set = UtxoSet::new();
        let outpoint = OutPoint {
            txid: [1u8; 32],
            vout: 0,
        };
        
        let utxo_entry = UtxoEntry {
            output: TxOutput {
                value: 1000000000,
                script_pubkey: vec![0x76, 0xa9, 0x14],
            },
            block_height: 100,
            is_coinbase: false,
            confirmations: 6,
        };

        // Test add and get
        utxo_set.add_utxo(outpoint, utxo_entry.clone());
        assert!(utxo_set.get_utxo(&outpoint).is_some());
        
        // Test spending
        utxo_set.mark_spent(&outpoint);
        assert!(utxo_set.is_spent(&outpoint));
    }

    #[tokio::test]
    async fn test_utxo_validation() {
        let validator = UtxoValidator::new();
        let utxo_set = UtxoSet::new();
        
        let transaction = Transaction {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            lock_time: 0,
            witness_flag: false,
        };

        let result = validator.validate_inputs(&transaction, &utxo_set, 1000).await.unwrap();
        assert!(result.is_valid);
    }
}
