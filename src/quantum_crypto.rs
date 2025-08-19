use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};
use rand::RngCore;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSignature {
    pub signature: String,
    pub public_key: String,
    pub message_hash: String,
}

/// Generate a new Dilithium2 keypair
pub fn generate_keypair() -> (String, String) {
    let (pk, sk) = dilithium2::keypair();
    
    let public_key = hex::encode(pk.as_bytes());
    let private_key = hex::encode(sk.as_bytes());
    
    (public_key, private_key)
}

/// Generate a QuantumCoin address from a public key
pub fn public_key_to_address(public_key: &str) -> String {
    let pk_bytes = hex::decode(public_key).expect("Invalid public key hex");
    
    // Hash the public key with Blake3
    let hash = blake3::hash(&pk_bytes);
    let hash_bytes = hash.as_bytes();
    
    // Take first 20 bytes for address
    let address_bytes = &hash_bytes[..20];
    
    // Add version byte (0x1C for QuantumCoin)
    let mut versioned_address = vec![0x1C];
    versioned_address.extend_from_slice(address_bytes);
    
    // Calculate checksum (double Blake3)
    let checksum_hash = blake3::hash(&versioned_address);
    let checksum = &checksum_hash.as_bytes()[..4];
    
    versioned_address.extend_from_slice(checksum);
    
    // Encode with custom base58 alphabet
    encode_base58(&versioned_address)
}

/// Sign a message using Dilithium2
pub fn sign_message(private_key: &str, message: &[u8]) -> Result<QuantumSignature> {
    let sk_bytes = hex::decode(private_key)
        .map_err(|_| anyhow!("Invalid private key hex"))?;
    
    let secret_key = dilithium2::SecretKey::from_bytes(&sk_bytes)
        .map_err(|_| anyhow!("Invalid Dilithium2 secret key"))?;
    
    let signed_message = dilithium2::sign(message, &secret_key);
    let signature_bytes = signed_message.as_bytes();
    
    // Extract public key from secret key
    let (public_key, _) = dilithium2::keypair_from_secret_key(&secret_key);
    let public_key_hex = hex::encode(public_key.as_bytes());
    
    let message_hash = hex::encode(blake3::hash(message).as_bytes());
    
    Ok(QuantumSignature {
        signature: hex::encode(signature_bytes),
        public_key: public_key_hex,
        message_hash,
    })
}

/// Verify a Dilithium2 signature
pub fn verify_signature(signature: &QuantumSignature, message: &[u8]) -> bool {
    // Verify message hash
    let computed_hash = hex::encode(blake3::hash(message).as_bytes());
    if computed_hash != signature.message_hash {
        return false;
    }
    
    // Decode signature and public key
    let signature_bytes = match hex::decode(&signature.signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    let public_key_bytes = match hex::decode(&signature.public_key) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    // Create public key object
    let public_key = match dilithium2::PublicKey::from_bytes(&public_key_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    
    // Create signed message object
    let signed_message = match dilithium2::SignedMessage::from_bytes(&signature_bytes) {
        Ok(sm) => sm,
        Err(_) => return false,
    };
    
    // Verify signature
    match dilithium2::open(&signed_message, &public_key) {
        Ok(verified_message) => verified_message == message,
        Err(_) => false,
    }
}

/// Derive a private key from public key (simplified for demo)
/// In a real implementation, this would not be possible
pub fn derive_private_key_from_public(public_key: &str) -> String {
    // This is a simplified implementation for the demo
    // In reality, you cannot derive a private key from a public key
    let hash = blake3::hash(public_key.as_bytes());
    hex::encode(hash.as_bytes())
}

/// Generate secure random bytes
pub fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

/// Hash data using Blake3
pub fn hash_data(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hex::encode(hash.as_bytes())
}

/// Custom Base58 encoding for QuantumCoin addresses
fn encode_base58(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    if input.is_empty() {
        return String::new();
    }
    
    // Count leading zeros
    let leading_zeros = input.iter().take_while(|&&b| b == 0).count();
    
    // Convert to big integer (simplified)
    let mut num: Vec<u8> = input[leading_zeros..].to_vec();
    let mut result = Vec::new();
    
    while !num.is_empty() && !num.iter().all(|&x| x == 0) {
        let mut carry = 0u16;
        let mut i = 0;
        
        while i < num.len() {
            carry = carry * 256 + num[i] as u16;
            num[i] = (carry / 58) as u8;
            carry %= 58;
            i += 1;
        }
        
        result.push(ALPHABET[carry as usize]);
        
        // Remove leading zeros
        while num.first() == Some(&0) && num.len() > 1 {
            num.remove(0);
        }
        if num.len() == 1 && num[0] == 0 {
            break;
        }
    }
    
    // Add leading '1's for leading zeros
    let leading_ones = "1".repeat(leading_zeros);
    result.reverse();
    
    leading_ones + &String::from_utf8(result).unwrap_or_default()
}

/// Quantum-resistant key derivation function
pub fn derive_key(seed: &[u8], salt: &[u8], iterations: u32) -> Result<Vec<u8>> {
    use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};
    
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|_| anyhow!("Invalid salt for key derivation"))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(seed, &salt_string)
        .map_err(|_| anyhow!("Key derivation failed"))?;
    
    Ok(password_hash.hash.unwrap().as_bytes().to_vec())
}

/// Advanced signature scheme for transactions
pub struct QuantumTransactionSigner {
    keypair: KeyPair,
}

impl QuantumTransactionSigner {
    pub fn new() -> Self {
        let (public_key, private_key) = generate_keypair();
        Self {
            keypair: KeyPair {
                public_key,
                private_key,
            },
        }
    }

    pub fn from_private_key(private_key: String) -> Result<Self> {
        // Derive public key from private key
        let sk_bytes = hex::decode(&private_key)
            .map_err(|_| anyhow!("Invalid private key hex"))?;
        
        let secret_key = dilithium2::SecretKey::from_bytes(&sk_bytes)
            .map_err(|_| anyhow!("Invalid Dilithium2 secret key"))?;
        
        let (public_key, _) = dilithium2::keypair_from_secret_key(&secret_key);
        let public_key_hex = hex::encode(public_key.as_bytes());
        
        Ok(Self {
            keypair: KeyPair {
                public_key: public_key_hex,
                private_key,
            },
        })
    }

    pub fn get_public_key(&self) -> &str {
        &self.keypair.public_key
    }

    pub fn get_address(&self) -> String {
        public_key_to_address(&self.keypair.public_key)
    }

    pub fn sign_transaction(&self, transaction_data: &[u8]) -> Result<QuantumSignature> {
        sign_message(&self.keypair.private_key, transaction_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let (pub_key, priv_key) = generate_keypair();
        assert!(!pub_key.is_empty());
        assert!(!priv_key.is_empty());
        assert_ne!(pub_key, priv_key);
    }

    #[test]
    fn test_address_generation() {
        let (pub_key, _) = generate_keypair();
        let address = public_key_to_address(&pub_key);
        assert!(!address.is_empty());
        assert!(address.starts_with('Q')); // QuantumCoin addresses start with Q
    }

    #[test]
    fn test_sign_and_verify() {
        let (pub_key, priv_key) = generate_keypair();
        let message = b"Hello, Quantum World!";
        
        let signature = sign_message(&priv_key, message).unwrap();
        assert_eq!(signature.public_key, pub_key);
        
        let is_valid = verify_signature(&signature, message);
        assert!(is_valid);
        
        // Test with different message
        let wrong_message = b"Wrong message";
        let is_invalid = verify_signature(&signature, wrong_message);
        assert!(!is_invalid);
    }

    #[test]
    fn test_transaction_signer() {
        let signer = QuantumTransactionSigner::new();
        let transaction_data = b"transaction_data_here";
        
        let signature = signer.sign_transaction(transaction_data).unwrap();
        let is_valid = verify_signature(&signature, transaction_data);
        assert!(is_valid);
    }
}
