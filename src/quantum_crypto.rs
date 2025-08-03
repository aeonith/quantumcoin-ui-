use pqcrypto_dilithium::dilithium2::{keypair, sign, verify, DetachedSignature, PublicKey, SecretKey};
use pqcrypto_traits::sign::{SignedMessage, Signer, Verifier};
use sha3::{Sha3_256, Digest};
use base64::{encode, decode};
use rand::{thread_rng, RngCore};

pub fn generate_keypair() -> (String, String) {
    let (public_key, secret_key) = keypair();
    
    let public_key_base64 = encode(public_key.as_bytes());
    let secret_key_base64 = encode(secret_key.as_bytes());
    
    (public_key_base64, secret_key_base64)
}

pub fn sign_message(message: &str, secret_key: &str) -> Result<String, &'static str> {
    let secret_key_bytes = decode(secret_key).map_err(|_| "Invalid secret key format")?;
    let secret_key = SecretKey::from_bytes(&secret_key_bytes).map_err(|_| "Invalid secret key")?;
    
    let signature = sign(message.as_bytes(), &secret_key);
    Ok(encode(signature.as_bytes()))
}

pub fn verify_signature(message: &str, signature: &str, public_key: &str) -> bool {
    if let (Ok(sig_bytes), Ok(pk_bytes)) = (decode(signature), decode(public_key)) {
        if let (Ok(signature), Ok(public_key)) = (
            DetachedSignature::from_bytes(&sig_bytes),
            PublicKey::from_bytes(&pk_bytes)
        ) {
            return public_key.verify(message.as_bytes(), &signature).is_ok();
        }
    }
    false
}

pub fn public_key_to_address(public_key: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(public_key.as_bytes());
    let hash = hasher.finalize();
    
    // Take first 20 bytes and encode as base58 (simplified)
    let address_bytes = &hash[0..20];
    encode(address_bytes)
}

pub fn generate_secure_random(size: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut bytes = vec![0u8; size];
    rng.fill_bytes(&mut bytes);
    bytes
}

pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, &'static str> {
    use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};
    
    let salt_string = SaltString::encode_b64(salt).map_err(|_| "Invalid salt")?;
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|_| "Key derivation failed")?;
    
    Ok(password_hash.hash.unwrap().as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keypair_generation() {
        let (public_key, secret_key) = generate_keypair();
        assert!(!public_key.is_empty());
        assert!(!secret_key.is_empty());
        assert_ne!(public_key, secret_key);
    }
    
    #[test]
    fn test_sign_and_verify() {
        let (public_key, secret_key) = generate_keypair();
        let message = "Hello, QuantumCoin!";
        
        let signature = sign_message(message, &secret_key).unwrap();
        assert!(verify_signature(message, &signature, &public_key));
        
        // Test with wrong message
        assert!(!verify_signature("Different message", &signature, &public_key));
    }
    
    #[test]
    fn test_address_generation() {
        let (public_key, _) = generate_keypair();
        let address = public_key_to_address(&public_key);
        assert!(!address.is_empty());
        
        // Same public key should generate same address
        let address2 = public_key_to_address(&public_key);
        assert_eq!(address, address2);
    }
}
