use std::env;

// Simple address derivation utility for QuantumCoin
// Converts a Dilithium public key (hex) to a QTC address
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: qtc-address <hex_public_key>");
        eprintln!("");
        eprintln!("Converts a Dilithium public key (hex format) to a QuantumCoin address.");
        eprintln!("Example: qtc-address 0x1234abcd...");
        std::process::exit(1);
    }
    
    let pubkey_hex = &args[1];
    
    // Remove 0x prefix if present
    let clean_hex = pubkey_hex.strip_prefix("0x").unwrap_or(pubkey_hex);
    
    // Validate hex format
    if let Err(_) = hex::decode(clean_hex) {
        eprintln!("Error: Invalid hex format for public key");
        std::process::exit(1);
    }
    
    // Generate QTC address from public key
    let qtc_address = public_key_to_address(clean_hex);
    
    println!("{}", qtc_address);
}

/// Convert a Dilithium public key (hex) to a QuantumCoin address
/// This is a simplified implementation - production would use proper Bech32 encoding
fn public_key_to_address(pubkey_hex: &str) -> String {
    use sha2::{Sha256, Digest};
    
    // Hash the public key
    let mut hasher = Sha256::new();
    hasher.update(pubkey_hex.as_bytes());
    let hash_result = hasher.finalize();
    
    // Take first 20 bytes (160 bits) for address
    let address_bytes = &hash_result[0..20];
    
    // Convert to QTC bech32-like format (simplified)
    // In production, this would use proper Bech32 encoding with qtc1 prefix
    let address_hex = hex::encode(address_bytes);
    
    format!("qtc1q{}", &address_hex[0..39]) // qtc1 prefix + 39 chars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_generation() {
        let pubkey = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let address = public_key_to_address(pubkey);
        
        assert!(address.starts_with("qtc1q"));
        assert_eq!(address.len(), 43); // qtc1q + 39 chars
    }

    #[test]
    fn test_consistent_generation() {
        let pubkey = "deadbeef1234567890abcdef1234567890abcdef1234567890abcdef12345678";
        let addr1 = public_key_to_address(pubkey);
        let addr2 = public_key_to_address(pubkey);
        
        assert_eq!(addr1, addr2); // Should be deterministic
    }
}
