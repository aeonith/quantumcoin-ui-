use bech32::{ToBase32, Variant, encode};
use pqcrypto_dilithium::dilithium2;
use pqcrypto_dilithium::dilithium2::{PublicKey, SecretKey, DetachedSignature, sign_detached, verify_detached};
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;

/// Post-quantum sign using Dilithium2
pub fn pq_sign(sk: &SecretKey, msg: &[u8]) -> Vec<u8> {
    sign_detached(msg, sk).as_bytes().to_vec()
}

/// Post-quantum verify using Dilithium2
pub fn pq_verify(pk: &PublicKey, msg: &[u8], sig: &[u8]) -> bool {
    if let Ok(det_sig) = DetachedSignature::from_bytes(sig.to_vec()) {
        verify_detached(&det_sig, msg, pk).is_ok()
    } else {
        false
    }
}

/// Generate QuantumCoin address from public key
pub fn address_from_pubkey(pubkey: &[u8]) -> String {
    let sha = Sha256::digest(pubkey);
    let rip = Ripemd160::digest(&sha);
    encode("qc", rip.to_base32(), Variant::Bech32).expect("bech32 encoding")
}

/// Generate keypair for QuantumCoin
pub fn generate_keypair() -> (PublicKey, SecretKey) {
    dilithium2::keypair()
}

/// Create transaction signature hash
pub fn tx_sighash(canonical_payload: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(canonical_payload);
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// Double SHA256 hash (for block headers)
pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    let mut result = [0u8; 32];
    result.copy_from_slice(&second);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let (pk, sk) = generate_keypair();
        assert_eq!(pk.as_bytes().len(), 1312); // Dilithium2 public key size
        assert_eq!(sk.as_bytes().len(), 2528); // Dilithium2 secret key size
    }

    #[test]
    fn test_sign_verify() {
        let (pk, sk) = generate_keypair();
        let message = b"Hello, QuantumCoin!";
        
        let signature = pq_sign(&sk, message);
        assert!(pq_verify(&pk, message, &signature));
        
        // Test with wrong message
        let wrong_message = b"Wrong message";
        assert!(!pq_verify(&pk, wrong_message, &signature));
    }

    #[test]
    fn test_address_generation() {
        let (pk, _) = generate_keypair();
        let address = address_from_pubkey(pk.as_bytes());
        
        assert!(address.starts_with("qc1"));
        assert!(address.len() > 10);
    }

    #[test]
    fn test_tx_sighash() {
        let data = b"test transaction data";
        let hash = tx_sighash(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_double_sha256() {
        let data = b"test block header";
        let hash = double_sha256(data);
        assert_eq!(hash.len(), 32);
    }
}
