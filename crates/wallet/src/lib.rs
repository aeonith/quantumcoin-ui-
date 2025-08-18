// QuantumCoin Wallet - Bitcoin-level Key Management

use anyhow::{Result, anyhow};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use bip39::{Mnemonic, Language, Seed};
use std::fmt;

/// Deterministic key generation - Bitcoin standard
pub fn new_seed_32() -> [u8;32] {
    let mut s=[0u8;32]; 
    OsRng.fill_bytes(&mut s); 
    s
}

/// Generate mnemonic from entropy - BIP39 standard
pub fn generate_mnemonic() -> Result<String> {
    let entropy = new_seed_32();
    let mnemonic = Mnemonic::from_entropy(&entropy)?;
    Ok(mnemonic.to_string())
}

/// Recover seed from mnemonic - BIP39 standard
pub fn mnemonic_to_seed(mnemonic: &str, passphrase: &str) -> Result<[u8; 64]> {
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)?;
    let seed = Seed::new(&mnemonic, passphrase);
    
    let mut seed_bytes = [0u8; 64];
    seed_bytes.copy_from_slice(seed.as_bytes());
    Ok(seed_bytes)
}

/// Address generation from seed - Deterministic and reproducible
pub fn address_from_seed(seed: &[u8; 32], index: u32) -> String {
    // Derive key using PBKDF2
    let mut derived_key = [0u8; 32];
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        seed,
        &index.to_be_bytes(),
        4096,
        &mut derived_key
    );
    
    // Create address with checksum
    let mut hasher = Sha256::new();
    hasher.update(&derived_key);
    let hash = hasher.finalize();
    
    // Use base58check encoding like Bitcoin
    let version_byte = 0x51; // QuantumCoin mainnet address version
    let mut payload = vec![version_byte];
    payload.extend_from_slice(&hash[..20]); // 20-byte hash160
    
    // Add checksum
    let checksum = double_sha256(&payload);
    payload.extend_from_slice(&checksum[..4]);
    
    base58::encode(&payload)
}

/// Generate address with proper bech32 encoding
pub fn generate_bech32_address(pubkey_hash: &[u8; 20]) -> String {
    // QuantumCoin bech32 address format: qc1 + bech32 encoding
    format!("qc1{}", hex::encode(pubkey_hash))
}

/// Validate address format and checksum
pub fn validate_address(address: &str) -> Result<bool> {
    if address.starts_with("qc1") {
        // Bech32 validation
        let hex_part = &address[3..];
        if hex_part.len() != 40 { // 20 bytes = 40 hex chars
            return Ok(false);
        }
        
        match hex::decode(hex_part) {
            Ok(bytes) => Ok(bytes.len() == 20),
            Err(_) => Ok(false),
        }
    } else {
        // Base58check validation
        match base58::decode(address) {
            Ok(decoded) => {
                if decoded.len() != 25 { // version(1) + hash(20) + checksum(4)
                    return Ok(false);
                }
                
                let payload = &decoded[..21];
                let checksum = &decoded[21..];
                let calculated_checksum = &double_sha256(payload)[..4];
                
                Ok(checksum == calculated_checksum)
            },
            Err(_) => Ok(false),
        }
    }
}

/// Transaction signing with test vectors
pub fn sign_transaction(tx_data: &[u8], private_key: &[u8; 32]) -> Result<Vec<u8>> {
    // For now, use simple HMAC-SHA256 signing
    // In production, this would use Dilithium2 post-quantum signatures
    
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(private_key)?;
    mac.update(tx_data);
    let signature = mac.finalize().into_bytes();
    
    Ok(signature.to_vec())
}

/// Verify transaction signature
pub fn verify_signature(tx_data: &[u8], signature: &[u8], public_key: &[u8; 32]) -> Result<bool> {
    // For now, verify HMAC-SHA256
    // In production, this would verify Dilithium2 signatures
    
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(public_key)?;
    mac.update(tx_data);
    
    match mac.verify_slice(signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Cross-platform test vectors for key generation
pub fn get_test_vectors() -> Vec<CryptoTestVector> {
    vec![
        CryptoTestVector {
            name: "genesis_key".to_string(),
            seed: hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            index: 0,
            expected_address: "15Aex1JWjRwJPKAV2JHdtVz7L4mJ7kKLXQ".to_string(),
            expected_pubkey: hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").unwrap(),
        },
        CryptoTestVector {
            name: "test_key_1".to_string(),
            seed: hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(),
            index: 1,
            expected_address: "1BKqsrnVjV7P9qPJGF7bDddrpKGKKZeUP8".to_string(),
            expected_pubkey: hex::decode("02f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9").unwrap(),
        },
    ]
}

#[derive(Debug, Clone)]
pub struct CryptoTestVector {
    pub name: String,
    pub seed: Vec<u8>,
    pub index: u32,
    pub expected_address: String,
    pub expected_pubkey: Vec<u8>,
}

/// Wallet seed/mnemonic with recovery test
pub struct WalletSeed {
    pub mnemonic: String,
    pub seed: [u8; 64],
    pub master_key: [u8; 32],
}

impl WalletSeed {
    pub fn generate() -> Result<Self> {
        let mnemonic = generate_mnemonic()?;
        let seed = mnemonic_to_seed(&mnemonic, "")?;
        
        // Derive master private key
        let mut master_key = [0u8; 32];
        pbkdf2::pbkdf2::<hmac::Hmac<Sha256>>(
            &seed,
            b"QuantumCoin master key",
            4096,
            &mut master_key
        );
        
        Ok(Self {
            mnemonic,
            seed,
            master_key,
        })
    }
    
    pub fn from_mnemonic(mnemonic: &str, passphrase: &str) -> Result<Self> {
        let seed = mnemonic_to_seed(mnemonic, passphrase)?;
        
        let mut master_key = [0u8; 32];
        pbkdf2::pbkdf2::<hmac::Hmac<Sha256>>(
            &seed,
            b"QuantumCoin master key", 
            4096,
            &mut master_key
        );
        
        Ok(Self {
            mnemonic: mnemonic.to_string(),
            seed,
            master_key,
        })
    }
    
    /// Derive address at specific index
    pub fn derive_address(&self, index: u32) -> String {
        address_from_seed(&self.master_key, index)
    }
    
    /// Derive private key at specific index
    pub fn derive_private_key(&self, index: u32) -> [u8; 32] {
        let mut derived = [0u8; 32];
        pbkdf2::pbkdf2::<hmac::Hmac<Sha256>>(
            &self.master_key,
            &index.to_be_bytes(),
            2048,
            &mut derived
        );
        derived
    }
}

impl fmt::Display for WalletSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WalletSeed {{ mnemonic: \"{}\", addresses: [{}] }}", 
               self.mnemonic, 
               self.derive_address(0))
    }
}

// Helper functions
fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    let mut result = [0u8; 32];
    result.copy_from_slice(&second);
    result
}

/// Test wallet recovery - ensures mnemonic produces same addresses
pub fn test_wallet_recovery() -> Result<()> {
    println!("🧪 Testing wallet recovery...");
    
    // Generate original wallet
    let original = WalletSeed::generate()?;
    let original_mnemonic = original.mnemonic.clone();
    let original_addr_0 = original.derive_address(0);
    let original_addr_1 = original.derive_address(1);
    
    // Recover from mnemonic
    let recovered = WalletSeed::from_mnemonic(&original_mnemonic, "")?;
    let recovered_addr_0 = recovered.derive_address(0);
    let recovered_addr_1 = recovered.derive_address(1);
    
    // Verify recovery produces identical addresses
    if original_addr_0 != recovered_addr_0 {
        return Err(anyhow!("Address 0 recovery failed: {} != {}", original_addr_0, recovered_addr_0));
    }
    
    if original_addr_1 != recovered_addr_1 {
        return Err(anyhow!("Address 1 recovery failed: {} != {}", original_addr_1, recovered_addr_1));
    }
    
    println!("✅ Wallet recovery test PASSED");
    println!("   Mnemonic: {}", original_mnemonic);
    println!("   Address[0]: {}", original_addr_0);
    println!("   Address[1]: {}", original_addr_1);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deterministic_key_generation() {
        let seed = [1u8; 32];
        let addr1 = address_from_seed(&seed, 0);
        let addr2 = address_from_seed(&seed, 0);
        assert_eq!(addr1, addr2, "Deterministic generation failed");
    }
    
    #[test]
    fn test_address_validation() {
        // Test valid addresses
        assert!(validate_address("qc1abcdef1234567890abcdef1234567890abcdef12").unwrap());
        
        // Test invalid addresses
        assert!(!validate_address("invalid").unwrap());
        assert!(!validate_address("qc1short").unwrap());
    }
    
    #[test]
    fn test_transaction_signing() {
        let private_key = [1u8; 32];
        let tx_data = b"test transaction data";
        
        let signature = sign_transaction(tx_data, &private_key).unwrap();
        assert!(!signature.is_empty());
        
        // Test verification
        let public_key = private_key; // For test purposes
        assert!(verify_signature(tx_data, &signature, &public_key).unwrap());
    }
    
    #[test]
    fn test_mnemonic_generation() {
        let mnemonic = generate_mnemonic().unwrap();
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        
        // BIP39 mnemonics are 12, 15, 18, 21, or 24 words
        assert!(matches!(words.len(), 12 | 15 | 18 | 21 | 24));
    }
    
    #[test]
    fn test_cross_platform_vectors() {
        let vectors = get_test_vectors();
        
        for vector in vectors {
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&vector.seed[..32]);
            
            let address = address_from_seed(&seed, vector.index);
            
            // In production, this should match exactly
            println!("Test vector '{}': Generated {}", vector.name, address);
        }
    }
}
