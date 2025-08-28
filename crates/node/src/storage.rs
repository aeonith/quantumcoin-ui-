use qc_types::*;
use anyhow::Result;
use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;

pub struct Storage { 
    pub db: DB 
}

impl Storage {
    pub fn open<P: AsRef<Path>>(p: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.optimize_level_style_compaction();
        opts.set_max_open_files(1000);
        
        Ok(Self { 
            db: DB::open(&opts, p)? 
        })
    }

    // Key prefixes for different data types
    fn k_utxo(op: &OutPoint) -> Vec<u8> {
        let mut k = b"U".to_vec();
        k.extend_from_slice(&op.txid.0);
        k.extend_from_slice(&op.vout.to_le_bytes());
        k
    }
    
    fn k_block(h: &Hash32) -> Vec<u8> { 
        let mut k = b"B".to_vec(); 
        k.extend_from_slice(&h.0); 
        k 
    }
    
    fn k_height(hh: u64) -> Vec<u8> { 
        let mut k = b"H".to_vec(); 
        k.extend_from_slice(&hh.to_le_bytes()); 
        k 
    }
    
    fn k_tip() -> Vec<u8> { 
        b"T:tip".to_vec() 
    }
    
    fn k_tx(txid: &Hash32) -> Vec<u8> {
        let mut k = b"X".to_vec();
        k.extend_from_slice(&txid.0);
        k
    }

    /// Get UTXO data
    pub fn get_utxo(&self, op: &OutPoint) -> Result<Option<(Amount, OutputType, u64, bool)>> {
        if let Some(v) = self.db.get(Self::k_utxo(op))? {
            Ok(Some(bincode::deserialize(&v)?))
        } else { 
            Ok(None) 
        }
    }

    /// Add UTXO to batch write
    pub fn put_utxo_batch(&self, wb: &mut WriteBatch, op: &OutPoint, val: &(Amount, OutputType, u64, bool)) {
        wb.put(Self::k_utxo(op), bincode::serialize(val).unwrap());
    }
    
    /// Delete UTXO from batch write
    pub fn del_utxo_batch(&self, wb: &mut WriteBatch, op: &OutPoint) {
        wb.delete(Self::k_utxo(op));
    }

    /// Write block to storage
    pub fn write_block(&self, hash: &Hash32, blk: &Block, height: u64) -> Result<()> {
        let mut wb = WriteBatch::default();
        wb.put(Self::k_block(hash), bincode::serialize(blk)?);
        wb.put(Self::k_height(height), hash.0);
        wb.put(Self::k_tip(), hash.0);
        
        // Index transactions
        for tx in &blk.txs {
            let txid = self.calculate_txid(tx);
            wb.put(Self::k_tx(&txid), bincode::serialize(&(height, tx))?);
        }
        
        self.db.write(wb)?;
        Ok(())
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &Hash32) -> Result<Option<Block>> {
        if let Some(v) = self.db.get(Self::k_block(hash))? {
            Ok(Some(bincode::deserialize(&v)?))
        } else {
            Ok(None)
        }
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        if let Some(hash_bytes) = self.db.get(Self::k_height(height))? {
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&hash_bytes);
            let hash = Hash32(hash_array);
            self.get_block(&hash)
        } else {
            Ok(None)
        }
    }

    /// Get current tip
    pub fn get_tip(&self) -> Result<Option<Hash32>> {
        if let Some(hash_bytes) = self.db.get(Self::k_tip())? {
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&hash_bytes);
            Ok(Some(Hash32(hash_array)))
        } else {
            Ok(None)
        }
    }

    /// Calculate transaction ID
    pub fn calculate_txid(&self, tx: &Transaction) -> Hash32 {
        use sha2::{Digest, Sha256};
        let serialized = bincode::serialize(tx).unwrap();
        let hash = Sha256::digest(&serialized);
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash);
        Hash32(arr)
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, txid: &Hash32) -> Result<Option<(u64, Transaction)>> {
        if let Some(v) = self.db.get(Self::k_tx(txid))? {
            Ok(Some(bincode::deserialize(&v)?))
        } else {
            Ok(None)
        }
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> Result<StorageStats> {
        let mut block_count = 0u64;
        let mut utxo_count = 0u64;
        
        // Count blocks (approximate)
        let iter = self.db.prefix_iterator(b"H");
        for _item in iter {
            block_count += 1;
        }
        
        // Count UTXOs (approximate)
        let iter = self.db.prefix_iterator(b"U");
        for _item in iter {
            utxo_count += 1;
        }
        
        Ok(StorageStats {
            block_count,
            utxo_count,
            database_size: 0, // TODO: Get actual DB size
        })
    }
}

#[derive(Debug)]
pub struct StorageStats {
    pub block_count: u64,
    pub utxo_count: u64,
    pub database_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_operations() -> Result<()> {
        let dir = tempdir()?;
        let storage = Storage::open(dir.path())?;
        
        // Test UTXO operations
        let outpoint = OutPoint { txid: Hash32::zero(), vout: 0 };
        let utxo_data = (100_000i64, OutputType::P2PQ { pubkey: vec![1u8; 1312] }, 0u64, false);
        
        let mut wb = WriteBatch::default();
        storage.put_utxo_batch(&mut wb, &outpoint, &utxo_data);
        storage.db.write(wb)?;
        
        let retrieved = storage.get_utxo(&outpoint)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().0, 100_000);
        
        Ok(())
    }
    
    #[test]
    fn test_block_storage() -> Result<()> {
        let dir = tempdir()?;
        let storage = Storage::open(dir.path())?;
        
        // Create test block
        let header = BlockHeader::new(1, Hash32::zero(), Hash32::zero(), 1000, 0x1d00ffff, 0);
        let block = Block::new(header, vec![]);
        let hash = Hash32([1u8; 32]);
        
        // Store block
        storage.write_block(&hash, &block, 0)?;
        
        // Retrieve block
        let retrieved = storage.get_block(&hash)?;
        assert!(retrieved.is_some());
        
        let retrieved_by_height = storage.get_block_by_height(0)?;
        assert!(retrieved_by_height.is_some());
        
        Ok(())
    }
}
