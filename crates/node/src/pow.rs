use sha2::{Digest, Sha256};
use qc_types::BlockHeader;

/// Double SHA256 hash for block headers (Bitcoin-style)
pub fn sha256d(header: &BlockHeader) -> [u8; 32] {
    let bytes = bincode::serialize(header).unwrap();
    let h1 = Sha256::digest(&bytes);
    let h2 = Sha256::digest(&h1);
    let mut out = [0u8; 32]; 
    out.copy_from_slice(&h2); 
    out
}

/// Check if block hash meets difficulty target
pub fn check_proof_of_work(hash: &[u8; 32], target: u128) -> bool {
    // Interpret hash as big-endian integer; valid if <= target
    let mut val: u128 = 0;
    for b in hash {
        val = (val << 8) | (*b as u128);
        if val > target {
            return false; // Early exit for efficiency
        }
    }
    val <= target
}

#[cfg(test)]
mod tests {
    use super::*;
    use qc_types::*;

    #[test]
    fn test_sha256d() {
        let header = BlockHeader::new(1, Hash32::zero(), Hash32::zero(), 1700000000, 0x1d00ffff, 0);
        let hash = sha256d(&header);
        assert_eq!(hash.len(), 32);
        
        // Same header should produce same hash
        let hash2 = sha256d(&header);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_proof_of_work() {
        // Easy target (high value)
        let easy_target = u128::MAX;
        let any_hash = [0u8; 32];
        assert!(check_proof_of_work(&any_hash, easy_target));
        
        // Impossible target
        let impossible_target = 0u128;
        assert!(!check_proof_of_work(&any_hash, impossible_target));
        
        // Test with actual values
        let low_hash = [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let high_target = 0x00010000u128 << (8 * 28);
        assert!(check_proof_of_work(&low_hash, high_target));
    }
}
