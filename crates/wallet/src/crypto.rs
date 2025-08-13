//! Post-quantum cryptography using Dilithium2

use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{SignedMessage, PublicKey as PQPublicKey, SecretKey as PQSecretKey, DetachedSignature};
use rand::rngs::OsRng;

/// Cryptographic errors
#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    /// Signature verification failed
    #[error("Signature verification failed")]
    InvalidSignature,
    
    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    
    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
}

/// Generate new Dilithium2 key pair
pub fn generate_keypair() -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    let (pk, sk) = dilithium2::keypair(&mut OsRng);
    Ok((pk.as_bytes().to_vec(), sk.as_bytes().to_vec()))
}

/// Sign message with Dilithium2
pub fn sign_detached(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let sk = dilithium2::SecretKey::from_bytes(secret_key)
        .map_err(|_| CryptoError::InvalidKeyFormat("Invalid secret key".to_string()))?;
    
    // Add domain separation for QuantumCoin transactions
    let domain_separated = format!("QTC-TX-V1|{}", hex::encode(message));
    
    let sig = dilithium2::detached_sign(domain_separated.as_bytes(), &sk);
    Ok(sig.as_bytes().to_vec())
}

/// Verify detached signature
pub fn verify_detached(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8]
) -> Result<(), CryptoError> {
    let pk = dilithium2::PublicKey::from_bytes(public_key)
        .map_err(|_| CryptoError::InvalidKeyFormat("Invalid public key".to_string()))?;
    
    let sig = dilithium2::DetachedSignature::from_bytes(signature)
        .map_err(|_| CryptoError::InvalidSignature)?;
    
    // Use same domain separation as signing
    let domain_separated = format!("QTC-TX-V1|{}", hex::encode(message));
    
    dilithium2::verify_detached_signature(&sig, domain_separated.as_bytes(), &pk)
        .map_err(|_| CryptoError::InvalidSignature)
}

/// Create address from public key (base64 encoded)
pub fn public_key_to_address(public_key: &[u8]) -> String {
    base64::encode(public_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keypair_generation() {
        let (pk, sk) = generate_keypair().expect("Key generation should succeed");
        
        assert!(!pk.is_empty());
        assert!(!sk.is_empty());
        assert_ne!(pk, sk);
    }
    
    #[test]
    fn test_sign_and_verify() {
        let (pk, sk) = generate_keypair().expect("Key generation should succeed");
        let message = b"test message";
        
        let signature = sign_detached(&sk, message).expect("Signing should succeed");
        assert!(!signature.is_empty());
        
        verify_detached(&pk, message, &signature).expect("Verification should succeed");
    }
    
    #[test]
    fn test_invalid_signature() {
        let (pk, sk) = generate_keypair().expect("Key generation should succeed");
        let message = b"test message";
        let wrong_message = b"wrong message";
        
        let signature = sign_detached(&sk, message).expect("Signing should succeed");
        
        // Should fail with wrong message
        assert!(verify_detached(&pk, wrong_message, &signature).is_err());
        
        // Should fail with corrupted signature
        let mut bad_signature = signature.clone();
        bad_signature[0] ^= 1;
        assert!(verify_detached(&pk, message, &bad_signature).is_err());
    }
    
    #[test]
    fn test_address_generation() {
        let (pk, _sk) = generate_keypair().expect("Key generation should succeed");
        
        let address = public_key_to_address(&pk);
        assert!(!address.is_empty());
        
        // Should be valid base64
        assert!(base64::decode(&address).is_ok());
    }
}
