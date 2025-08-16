//! Post-quantum cryptographic functions for genesis block

use anyhow::{Result, Context};
use pqcrypto_dilithium::{dilithium2, dilithium2::*};
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// Post-quantum key pair for genesis block signing
#[derive(Debug, Clone)]
pub struct QuantumKeyPair {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

/// Post-quantum signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSignature {
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub algorithm: String,
}

/// Deterministic key generation for reproducible genesis blocks
pub struct DeterministicKeyGen {
    seed: [u8; 32],
}

impl DeterministicKeyGen {
    /// Create new deterministic key generator with seed
    pub fn new(seed: [u8; 32]) -> Self {
        Self { seed }
    }
    
    /// Generate deterministic key pair from seed
    pub fn generate_keypair(&self) -> Result<QuantumKeyPair> {
        // Use BLAKE3 to derive a proper seed for Dilithium
        let mut hasher = Hasher::new();
        hasher.update(b"QuantumCoin Genesis Key Generation");
        hasher.update(&self.seed);
        let derived_seed = hasher.finalize();
        
        // Initialize RNG with derived seed
        let mut rng = DeterministicRng::new(*derived_seed.as_bytes());
        
        // Generate Dilithium2 key pair
        let (public_key, secret_key) = keypair_with_rng(&mut rng);
        
        Ok(QuantumKeyPair {
            public_key,
            secret_key,
        })
    }
}

/// Deterministic RNG for reproducible key generation
struct DeterministicRng {
    state: [u8; 32],
    counter: u64,
}

impl DeterministicRng {
    fn new(seed: [u8; 32]) -> Self {
        Self {
            state: seed,
            counter: 0,
        }
    }
    
    fn next_bytes(&mut self, bytes: &mut [u8]) {
        let mut hasher = Hasher::new();
        hasher.update(&self.state);
        hasher.update(&self.counter.to_le_bytes());
        
        let hash = hasher.finalize();
        let hash_bytes = hash.as_bytes();
        
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = hash_bytes[i % hash_bytes.len()];
        }
        
        // Update state for next call
        self.counter += 1;
        self.state = *hash.as_bytes();
    }
}

impl rand_core::RngCore for DeterministicRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.next_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }
    
    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.next_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }
    
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.next_bytes(dest);
    }
    
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand_core::CryptoRng for DeterministicRng {}

/// Genesis block cryptographic operations
pub struct GenesisCrypto {
    keypair: QuantumKeyPair,
}

impl GenesisCrypto {
    /// Create new genesis crypto with deterministic seed
    pub fn new_deterministic(seed: [u8; 32]) -> Result<Self> {
        let keygen = DeterministicKeyGen::new(seed);
        let keypair = keygen.generate_keypair()
            .context("Failed to generate deterministic keypair")?;
        
        Ok(Self { keypair })
    }
    
    /// Create genesis crypto with specific keypair
    pub fn with_keypair(keypair: QuantumKeyPair) -> Self {
        Self { keypair }
    }
    
    /// Sign data with Dilithium2
    pub fn sign(&self, data: &[u8]) -> Result<QuantumSignature> {
        let signed_message = sign(data, &self.keypair.secret_key);
        
        Ok(QuantumSignature {
            signature: signed_message.to_vec(),
            public_key: self.keypair.public_key.as_bytes().to_vec(),
            algorithm: "dilithium2".to_string(),
        })
    }
    
    /// Get public key
    pub fn public_key(&self) -> &PublicKey {
        &self.keypair.public_key
    }
    
    /// Get public key bytes
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.keypair.public_key.as_bytes().to_vec()
    }
}

/// Verify a quantum signature
pub fn verify_quantum_signature(
    data: &[u8],
    signature: &QuantumSignature,
) -> Result<bool> {
    if signature.algorithm != "dilithium2" {
        anyhow::bail!("Unsupported signature algorithm: {}", signature.algorithm);
    }
    
    let public_key = PublicKey::from_bytes(&signature.public_key)
        .map_err(|_| anyhow::anyhow!("Invalid public key"))?;
    
    match open(&signature.signature, &public_key) {
        Ok(message) => Ok(message == data),
        Err(_) => Ok(false),
    }
}

/// Create BLAKE3 hash
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Create double BLAKE3 hash for enhanced security
pub fn double_blake3_hash(data: &[u8]) -> [u8; 32] {
    let first_hash = blake3_hash(data);
    blake3_hash(&first_hash)
}

/// Generate deterministic seed from chain specification
pub fn generate_genesis_seed(chain_name: &str, network_magic: &[u8; 4], timestamp: u64) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(b"QuantumCoin Genesis Seed v2.0");
    hasher.update(chain_name.as_bytes());
    hasher.update(network_magic);
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(b"Post-Quantum Cryptography Era");
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deterministic_key_generation() {
        let seed = [0x42; 32];
        let crypto1 = GenesisCrypto::new_deterministic(seed).unwrap();
        let crypto2 = GenesisCrypto::new_deterministic(seed).unwrap();
        
        // Same seed should generate same public key
        assert_eq!(
            crypto1.public_key().as_bytes(),
            crypto2.public_key().as_bytes()
        );
    }
    
    #[test]
    fn test_quantum_signature() {
        let seed = [0x42; 32];
        let crypto = GenesisCrypto::new_deterministic(seed).unwrap();
        
        let data = b"test data";
        let signature = crypto.sign(data).unwrap();
        
        assert!(verify_quantum_signature(data, &signature).unwrap());
        
        // Wrong data should fail
        let wrong_data = b"wrong data";
        assert!(!verify_quantum_signature(wrong_data, &signature).unwrap());
    }
    
    #[test]
    fn test_blake3_hashing() {
        let data = b"test";
        let hash1 = blake3_hash(data);
        let hash2 = blake3_hash(data);
        assert_eq!(hash1, hash2);
        
        let double1 = double_blake3_hash(data);
        let double2 = double_blake3_hash(data);
        assert_eq!(double1, double2);
        assert_ne!(hash1, double1);
    }
    
    #[test]
    fn test_genesis_seed_generation() {
        let seed1 = generate_genesis_seed("quantumcoin", &[0x51, 0x54, 0x43, 0x4D], 1640995200);
        let seed2 = generate_genesis_seed("quantumcoin", &[0x51, 0x54, 0x43, 0x4D], 1640995200);
        assert_eq!(seed1, seed2);
        
        // Different parameters should produce different seeds
        let seed3 = generate_genesis_seed("quantumcoin", &[0x51, 0x54, 0x43, 0x4D], 1640995201);
        assert_ne!(seed1, seed3);
    }
}
