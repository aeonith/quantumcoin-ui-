use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Amount = i64;      // sats (8 decimals)
pub type Height = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub fn zero() -> Self { Self([0u8; 32]) }
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
    
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash32(arr))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutPoint { 
    pub txid: Hash32, 
    pub vout: u32 
}

impl OutPoint {
    pub fn new(txid: Hash32, vout: u32) -> Self {
        Self { txid, vout }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputType {
    /// PQ pay-to-pubkey (P2PQ)
    P2PQ { pubkey: Vec<u8> },

    /// Revocable output (RevStop): owner may cancel spends for window_blocks after creation.
    P2PQRevocable { pubkey: Vec<u8>, window_blocks: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOut { 
    pub value: Amount, 
    pub kind: OutputType 
}

impl TxOut {
    pub fn new_p2pq(value: Amount, pubkey: Vec<u8>) -> Self {
        Self {
            value,
            kind: OutputType::P2PQ { pubkey }
        }
    }
    
    pub fn new_revocable(value: Amount, pubkey: Vec<u8>, window_blocks: u32) -> Self {
        Self {
            value,
            kind: OutputType::P2PQRevocable { pubkey, window_blocks }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxIn {
    pub prevout: OutPoint,
    pub pq_signature: Vec<u8>,
    pub cancel: bool, // true means this is a RevStop cancel-intent spend
}

impl TxIn {
    pub fn new(prevout: OutPoint, pq_signature: Vec<u8>, cancel: bool) -> Self {
        Self { prevout, pq_signature, cancel }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u32,
    pub vin: Vec<TxIn>,
    pub vout: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn new(version: u32, vin: Vec<TxIn>, vout: Vec<TxOut>, lock_time: u32) -> Self {
        Self { version, vin, vout, lock_time }
    }
    
    pub fn is_coinbase(&self) -> bool {
        self.vin.is_empty()
    }
    
    pub fn total_output_value(&self) -> Amount {
        self.vout.iter().map(|o| o.value).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block: Hash32,
    pub merkle_root: Hash32,
    pub time: u64,
    pub bits: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn new(
        version: u32,
        prev_block: Hash32,
        merkle_root: Hash32,
        time: u64,
        bits: u32,
        nonce: u32,
    ) -> Self {
        Self {
            version,
            prev_block,
            merkle_root,
            time,
            bits,
            nonce,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block { 
    pub header: BlockHeader, 
    pub txs: Vec<Transaction> 
}

impl Block {
    pub fn new(header: BlockHeader, txs: Vec<Transaction>) -> Self {
        Self { header, txs }
    }
    
    pub fn transaction_count(&self) -> usize {
        self.txs.len()
    }
    
    pub fn total_fees(&self) -> Amount {
        // Calculate fees as input_sum - output_sum for each non-coinbase tx
        // This is a simplified calculation
        0 // TODO: Implement proper fee calculation with UTXO lookup
    }
}

#[derive(Debug, Error)]
pub enum TypesError {
    #[error("serialization error")]
    Serialization,
    #[error("invalid hash format")]
    InvalidHash,
    #[error("invalid transaction")]
    InvalidTransaction,
    #[error("invalid block")]
    InvalidBlock,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash32_creation() {
        let hash = Hash32::zero();
        assert_eq!(hash.0, [0u8; 32]);
        assert_eq!(hash.to_hex(), "0".repeat(64));
    }

    #[test]
    fn test_outpoint_creation() {
        let txid = Hash32::zero();
        let outpoint = OutPoint::new(txid, 0);
        assert_eq!(outpoint.txid, txid);
        assert_eq!(outpoint.vout, 0);
    }

    #[test]
    fn test_transaction_coinbase() {
        let coinbase = Transaction::new(1, vec![], vec![], 0);
        assert!(coinbase.is_coinbase());
        
        let regular_tx = Transaction::new(
            1,
            vec![TxIn::new(OutPoint::new(Hash32::zero(), 0), vec![], false)],
            vec![],
            0
        );
        assert!(!regular_tx.is_coinbase());
    }
}
