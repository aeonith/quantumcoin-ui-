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
pub struct UtxoSet {
    utxos: RwLock<HashMap<OutPoint, UtxoEntry>>,
    spent_utxos: RwLock<HashSet<OutPoint>>,
}

impl UtxoSet {
    pub fn new() -> Self {
        Self {
            utxos: RwLock::new(HashMap::new()),
            spent_utxos: RwLock::new(HashSet::new()),
        }
    }

    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<UtxoEntry> {
        self.utxos.read().get(outpoint).cloned()
    }

    pub fn is_spent(&self, outpoint: &OutPoint) -> bool {
        self.spent_utxos.read().contains(outpoint)
    }
}

pub struct UtxoValidator;

impl UtxoValidator {
    pub fn new() -> Self {
        Self
    }

    pub async fn validate_inputs(
        &self,
        transaction: &Transaction,
        utxo_set: &UtxoSet,
        current_height: u32,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut total_input_value = 0u64;
        
        for input in &transaction.inputs {
            match utxo_set.get_utxo(&input.previous_output) {
                Some(utxo_entry) => {
                    // Check if already spent
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
                        if confirmations < 100 { // 100 block maturity
                            errors.push(ValidationError::CoinbaseNotMature {
                                height: utxo_entry.block_height,
                                required: 100,
                            });
                            continue;
                        }
                    }

                    total_input_value = total_input_value.saturating_add(utxo_entry.output.value);
                }
                None => {
                    errors.push(ValidationError::UtxoNotFound {
                        txid: hex::encode(input.previous_output.txid),
                        vout: input.previous_output.vout,
                    });
                }
            }
        }

        let total_output_value: u64 = transaction.outputs.iter().map(|o| o.value).sum();

        if errors.is_empty() {
            Ok(ValidationResult::valid(total_input_value, total_output_value, 0, 0, 0))
        } else {
            Ok(ValidationResult::invalid(errors))
        }
    }
}
