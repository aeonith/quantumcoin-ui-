use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use crate::transaction::{SignedTransaction, TransactionOutput};

/// Unspent Transaction Output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXO {
    pub tx_id: String,
    pub output_index: u32,
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
    pub address: String,
    pub block_height: u64,
    pub is_coinbase: bool,
    pub confirmations: u64,
}

impl UTXO {
    pub fn new(
        tx_id: String,
        output_index: u32,
        output: &TransactionOutput,
        block_height: u64,
        is_coinbase: bool,
    ) -> Self {
        Self {
            tx_id,
            output_index,
            amount: output.value,
            script_pubkey: output.script_pubkey.clone(),
            address: output.address.clone(),
            block_height,
            is_coinbase,
            confirmations: 0,
        }
    }

    pub fn get_outpoint(&self) -> String {
        format!("{}:{}", self.tx_id, self.output_index)
    }

    pub fn update_confirmations(&mut self, current_height: u64) {
        if current_height >= self.block_height {
            self.confirmations = current_height - self.block_height + 1;
        }
    }

    pub fn is_mature(&self, coinbase_maturity: u64) -> bool {
        if !self.is_coinbase {
            return true;
        }
        self.confirmations >= coinbase_maturity
    }
}

/// UTXO Set manager for tracking unspent outputs
#[derive(Debug, Clone, Default)]
pub struct UTXOSet {
    /// Map from outpoint (tx_id:output_index) to UTXO
    utxos: HashMap<String, UTXO>,
    /// Total value in the UTXO set
    total_value: u64,
    /// Current blockchain height
    current_height: u64,
}

impl UTXOSet {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
            total_value: 0,
            current_height: 0,
        }
    }

    /// Add a new UTXO to the set
    pub fn add_utxo(&mut self, utxo: UTXO) -> Result<()> {
        let outpoint = utxo.get_outpoint();
        
        if self.utxos.contains_key(&outpoint) {
            return Err(anyhow!("UTXO already exists: {}", outpoint));
        }

        self.total_value = self.total_value
            .checked_add(utxo.amount)
            .ok_or_else(|| anyhow!("UTXO set value overflow"))?;

        self.utxos.insert(outpoint, utxo);
        Ok(())
    }

    /// Remove a UTXO from the set
    pub fn remove_utxo(&mut self, outpoint: &str) -> Result<UTXO> {
        let utxo = self.utxos.remove(outpoint)
            .ok_or_else(|| anyhow!("UTXO not found: {}", outpoint))?;

        self.total_value = self.total_value
            .checked_sub(utxo.amount)
            .ok_or_else(|| anyhow!("UTXO set value underflow"))?;

        Ok(utxo)
    }

    /// Get a UTXO by outpoint
    pub fn get_utxo(&self, outpoint: &str) -> Option<&UTXO> {
        self.utxos.get(outpoint)
    }

    /// Check if a UTXO exists
    pub fn contains_utxo(&self, outpoint: &str) -> bool {
        self.utxos.contains_key(outpoint)
    }

    /// Get all UTXOs for an address
    pub fn get_utxos_for_address(&self, address: &str) -> Vec<&UTXO> {
        self.utxos.values()
            .filter(|utxo| utxo.address == address)
            .collect()
    }

    /// Get spendable UTXOs for an address (excluding immature coinbase)
    pub fn get_spendable_utxos(&self, address: &str, coinbase_maturity: u64) -> Vec<&UTXO> {
        self.get_utxos_for_address(address)
            .into_iter()
            .filter(|utxo| utxo.is_mature(coinbase_maturity))
            .collect()
    }

    /// Calculate balance for an address
    pub fn get_balance(&self, address: &str) -> u64 {
        self.utxos.values()
            .filter(|utxo| utxo.address == address)
            .map(|utxo| utxo.amount)
            .sum()
    }

    /// Calculate spendable balance for an address
    pub fn get_spendable_balance(&self, address: &str, coinbase_maturity: u64) -> u64 {
        self.get_spendable_utxos(address, coinbase_maturity)
            .iter()
            .map(|utxo| utxo.amount)
            .sum()
    }

    /// Apply a transaction to the UTXO set
    pub fn apply_transaction(
        &mut self, 
        tx: &SignedTransaction, 
        block_height: u64,
        is_coinbase: bool
    ) -> Result<()> {
        // Remove spent outputs (inputs)
        if !is_coinbase {
            for input in &tx.inputs {
                self.remove_utxo(&input.previous_output)?;
            }
        }

        // Add new outputs
        for (output_index, output) in tx.outputs.iter().enumerate() {
            let utxo = UTXO::new(
                tx.id.clone(),
                output_index as u32,
                output,
                block_height,
                is_coinbase,
            );
            self.add_utxo(utxo)?;
        }

        Ok(())
    }

    /// Reverse a transaction from the UTXO set (for reorganizations)
    pub fn reverse_transaction(
        &mut self, 
        tx: &SignedTransaction, 
        spent_utxos: &[UTXO]
    ) -> Result<()> {
        // Remove outputs that were added
        for (output_index, _) in tx.outputs.iter().enumerate() {
            let outpoint = format!("{}:{}", tx.id, output_index);
            self.remove_utxo(&outpoint)?;
        }

        // Restore spent outputs (inputs)
        for utxo in spent_utxos {
            self.add_utxo(utxo.clone())?;
        }

        Ok(())
    }

    /// Update the current blockchain height
    pub fn set_height(&mut self, height: u64) {
        self.current_height = height;
        
        // Update confirmations for all UTXOs
        for utxo in self.utxos.values_mut() {
            utxo.update_confirmations(height);
        }
    }

    /// Get the total value in the UTXO set
    pub fn total_value(&self) -> u64 {
        self.total_value
    }

    /// Get the number of UTXOs
    pub fn size(&self) -> usize {
        self.utxos.len()
    }

    /// Check if the UTXO set is consistent
    pub fn verify_consistency(&self) -> Result<()> {
        let calculated_value: u64 = self.utxos.values().map(|utxo| utxo.amount).sum();
        
        if calculated_value != self.total_value {
            return Err(anyhow!(
                "UTXO set value inconsistency: calculated={}, stored={}",
                calculated_value,
                self.total_value
            ));
        }

        Ok(())
    }

    /// Get statistics about the UTXO set
    pub fn get_stats(&self) -> UTXOStats {
        let mut stats = UTXOStats::default();
        
        stats.total_utxos = self.utxos.len();
        stats.total_value = self.total_value;
        
        for utxo in self.utxos.values() {
            if utxo.is_coinbase {
                stats.coinbase_utxos += 1;
                stats.coinbase_value += utxo.amount;
            }
            
            if utxo.amount >= 100_000_000 { // 1 QTC
                stats.large_utxos += 1;
            }
            
            stats.addresses.insert(utxo.address.clone());
        }
        
        stats.unique_addresses = stats.addresses.len();
        stats
    }

    /// Prune old UTXOs (for space efficiency)
    pub fn prune_old_utxos(&mut self, max_age_blocks: u64) -> usize {
        if self.current_height <= max_age_blocks {
            return 0;
        }

        let min_height = self.current_height - max_age_blocks;
        let mut pruned = 0;
        
        self.utxos.retain(|_, utxo| {
            if utxo.block_height < min_height && utxo.amount < 1000 { // Prune dust
                self.total_value -= utxo.amount;
                pruned += 1;
                false
            } else {
                true
            }
        });
        
        pruned
    }
}

/// Statistics about the UTXO set
#[derive(Debug, Default)]
pub struct UTXOStats {
    pub total_utxos: usize,
    pub total_value: u64,
    pub coinbase_utxos: usize,
    pub coinbase_value: u64,
    pub large_utxos: usize,
    pub unique_addresses: usize,
    pub addresses: std::collections::HashSet<String>,
}

impl UTXOStats {
    pub fn average_utxo_value(&self) -> f64 {
        if self.total_utxos > 0 {
            self.total_value as f64 / self.total_utxos as f64
        } else {
            0.0
        }
    }

    pub fn coinbase_percentage(&self) -> f64 {
        if self.total_utxos > 0 {
            (self.coinbase_utxos as f64 / self.total_utxos as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TransactionOutput, SignedTransaction, TransactionInput};
    use chrono::Utc;

    fn create_test_utxo() -> UTXO {
        let output = TransactionOutput {
            value: 5000000000, // 50 QTC
            script_pubkey: vec![0x76, 0xa9, 0x14], // Example script
            address: "qtc1qtest000000000000000000000000000".to_string(),
        };
        
        UTXO::new("test_tx_1".to_string(), 0, &output, 100, false)
    }

    #[test]
    fn test_utxo_creation() {
        let utxo = create_test_utxo();
        assert_eq!(utxo.amount, 5000000000);
        assert_eq!(utxo.get_outpoint(), "test_tx_1:0");
        assert!(!utxo.is_coinbase);
    }

    #[test]
    fn test_utxo_set_operations() {
        let mut utxo_set = UTXOSet::new();
        let utxo = create_test_utxo();
        
        // Add UTXO
        assert!(utxo_set.add_utxo(utxo.clone()).is_ok());
        assert_eq!(utxo_set.total_value(), 5000000000);
        assert_eq!(utxo_set.size(), 1);
        
        // Get UTXO
        let retrieved = utxo_set.get_utxo(&utxo.get_outpoint());
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().amount, 5000000000);
        
        // Remove UTXO
        let removed = utxo_set.remove_utxo(&utxo.get_outpoint());
        assert!(removed.is_ok());
        assert_eq!(utxo_set.total_value(), 0);
        assert_eq!(utxo_set.size(), 0);
    }

    #[test]
    fn test_address_balance() {
        let mut utxo_set = UTXOSet::new();
        let address = "qtc1qtest000000000000000000000000000";
        
        // Add multiple UTXOs for the same address
        for i in 0..3 {
            let output = TransactionOutput {
                value: 1000000000, // 10 QTC each
                script_pubkey: vec![],
                address: address.to_string(),
            };
            
            let utxo = UTXO::new(format!("tx_{}", i), 0, &output, 100, false);
            utxo_set.add_utxo(utxo).unwrap();
        }
        
        assert_eq!(utxo_set.get_balance(address), 3000000000); // 30 QTC total
        assert_eq!(utxo_set.get_utxos_for_address(address).len(), 3);
    }

    #[test]
    fn test_coinbase_maturity() {
        let mut utxo_set = UTXOSet::new();
        
        // Add coinbase UTXO
        let output = TransactionOutput {
            value: 5000000000,
            script_pubkey: vec![],
            address: "qtc1qminer000000000000000000000000".to_string(),
        };
        
        let mut utxo = UTXO::new("coinbase_tx".to_string(), 0, &output, 100, true);
        utxo.confirmations = 50; // Not mature yet (assuming 100 block maturity)
        
        assert!(!utxo.is_mature(100));
        assert_eq!(utxo_set.get_spendable_balance(&utxo.address, 100), 0);
        
        utxo.confirmations = 100; // Now mature
        assert!(utxo.is_mature(100));
    }

    #[test]
    fn test_transaction_application() {
        let mut utxo_set = UTXOSet::new();
        
        // Create initial UTXO
        let initial_utxo = create_test_utxo();
        utxo_set.add_utxo(initial_utxo.clone()).unwrap();
        
        // Create transaction that spends the UTXO
        let tx = SignedTransaction {
            id: "spending_tx".to_string(),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: initial_utxo.get_outpoint(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 2500000000, // 25 QTC to recipient
                    script_pubkey: vec![],
                    address: "qtc1qrecipient000000000000000000000".to_string(),
                },
                TransactionOutput {
                    value: 2400000000, // 24 QTC change
                    script_pubkey: vec![],
                    address: initial_utxo.address.clone(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: "test_signature".to_string(),
            public_key: "test_public_key".to_string(),
        };
        
        // Apply transaction
        assert!(utxo_set.apply_transaction(&tx, 200, false).is_ok());
        
        // Check results
        assert_eq!(utxo_set.size(), 2); // Two new outputs
        assert!(!utxo_set.contains_utxo(&initial_utxo.get_outpoint())); // Old UTXO spent
        assert!(utxo_set.contains_utxo(&format!("{}:0", tx.id))); // New output 0
        assert!(utxo_set.contains_utxo(&format!("{}:1", tx.id))); // New output 1
        
        // Check balances
        assert_eq!(utxo_set.get_balance("qtc1qrecipient000000000000000000000"), 2500000000);
        assert_eq!(utxo_set.get_balance(&initial_utxo.address), 2400000000);
        
        // Total should remain the same minus fee
        assert_eq!(utxo_set.total_value(), 4900000000); // 49 QTC (1 QTC fee)
    }

    #[test]
    fn test_utxo_set_consistency() {
        let mut utxo_set = UTXOSet::new();
        
        for i in 0..10 {
            let output = TransactionOutput {
                value: 1000000000 + i * 100000, // Varying amounts
                script_pubkey: vec![],
                address: format!("qtc1qaddr{:010}", i),
            };
            
            let utxo = UTXO::new(format!("tx_{}", i), 0, &output, 100 + i, false);
            utxo_set.add_utxo(utxo).unwrap();
        }
        
        // Verify consistency
        assert!(utxo_set.verify_consistency().is_ok());
        
        // Get stats
        let stats = utxo_set.get_stats();
        assert_eq!(stats.total_utxos, 10);
        assert_eq!(stats.unique_addresses, 10);
        assert_eq!(stats.coinbase_utxos, 0);
    }
}
