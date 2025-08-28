use crate::storage::Storage;
use crate::pow::{sha256d, check_proof_of_work};
use crate::target::bits_to_target;
use qc_types::*;
use qc_validation::{ChainSpec, validate_transaction, block_subsidy, merkle_root};
use anyhow::{Result, bail};
use rocksdb::WriteBatch;
use sha2::{Digest, Sha256};

pub struct ChainState<'a> {
    pub spec: &'a ChainSpec,
    pub store: &'a Storage,
}

impl<'a> ChainState<'a> {
    pub fn apply_block(&self, height: u64, block: &Block) -> Result<()> {
        // Verify proof of work
        let target = bits_to_target(block.header.bits);
        let block_hash = sha256d(&block.header);
        if !check_proof_of_work(&block_hash, target) {
            bail!("Invalid proof of work");
        }

        // Verify merkle root
        let calculated_merkle = merkle_root(&block.txs);
        if calculated_merkle.0 != block.header.merkle_root.0 {
            bail!("Merkle root mismatch");
        }

        // TODO: Verify timestamp, previous block linkage, etc.

        let mut wb = WriteBatch::default();
        
        // Build UTXO lookup function
        let lookup = |op: &OutPoint| self.store.get_utxo(op).ok().flatten();

        // Validate and apply transactions
        for (i, tx) in block.txs.iter().enumerate() {
            if i == 0 {
                // Coinbase transaction validation
                let subsidy = block_subsidy(self.spec, height);
                let total_out: i128 = tx.vout.iter().map(|o| o.value as i128).sum();
                
                // TODO: Add transaction fees to subsidy calculation
                if total_out > subsidy as i128 { 
                    bail!("Coinbase output exceeds subsidy + fees"); 
                }
            } else {
                // Regular transaction validation
                validate_transaction(self.spec, height, tx, false, &lookup)
                    .map_err(|e| anyhow::anyhow!("Transaction validation failed: {}", e))?;
                
                // Remove spent UTXOs
                for input in &tx.vin {
                    self.store.del_utxo_batch(&mut wb, &input.prevout);
                }
            }

            // Add new UTXOs from outputs
            for (vout, output) in tx.vout.iter().enumerate() {
                let txid = self.calculate_txid(tx);
                let outpoint = OutPoint::new(txid, vout as u32);
                self.store.put_utxo_batch(
                    &mut wb, 
                    &outpoint, 
                    &(output.value, output.kind.clone(), height, i == 0)
                );
            }
        }

        // Write block and update tip
        self.store.db.write(wb)?;
        let block_hash = self.block_hash(&block.header);
        self.store.write_block(&block_hash, block, height)?;
        
        info!("✅ Applied block at height {} with {} transactions", height, block.txs.len());
        Ok(())
    }

    pub fn block_hash(&self, header: &BlockHeader) -> Hash32 {
        let hash = sha256d(header);
        Hash32(hash)
    }
    
    pub fn calculate_txid(&self, tx: &Transaction) -> Hash32 {
        let serialized = bincode::serialize(tx).expect("serialize transaction");
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }
    
    pub fn get_chain_tip(&self) -> Result<Option<(Hash32, u64)>> {
        if let Some(tip_hash) = self.store.get_tip()? {
            // TODO: Store height with tip for efficiency
            // For now, return height 0 as placeholder
            Ok(Some((tip_hash, 0)))
        } else {
            Ok(None)
        }
    }
    
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<(Amount, OutputType, u64, bool)>> {
        self.store.get_utxo(outpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_chainstate_operations() -> Result<()> {
        let temp_dir = tempdir()?;
        let storage = Storage::open(temp_dir.path())?;
        
        // Load test spec
        let spec_content = r#"
[network]
name = "QuantumCoin"
symbol = "QC"
decimals = 8
version = "1.0.0"

[consensus]
hash_function = "sha256d"
target_block_time_secs = 600
difficulty_adjustment = "ASERT"
asert_half_life_secs = 2592000

[supply]
max_supply_sats = 22000000_00000000
halving_interval_blocks = 105120
premine_sats = 0

[txpolicy]
max_tx_size = 100000
min_fee_per_kb_sats = 1000
dust_threshold_sats = 546
max_inputs = 32
max_outputs = 32
coinbase_maturity = 100

[revstop]
window_blocks = 30
        "#;
        
        let spec: ChainSpec = toml::from_str(spec_content)?;
        let cs = ChainState { spec: &spec, store: &storage };
        
        // Test block hash calculation
        let header = BlockHeader::new(1, Hash32::zero(), Hash32::zero(), 1700000000, 0x1d00ffff, 12345);
        let hash = cs.block_hash(&header);
        assert_ne!(hash, Hash32::zero());
        
        Ok(())
    }
}
