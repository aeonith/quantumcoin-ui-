# QuantumCoin Genesis Block Specification

## Overview

This document defines the complete specification for QuantumCoin's genesis block system, ensuring deterministic, reproducible, and cryptographically secure genesis block generation.

## Version

- **Specification Version**: 2.0.0
- **Implementation**: QuantumCoin Genesis Crate v2.0.0
- **Chain Specification**: v2.0.0 (chain_spec.toml)

## Design Principles

### 1. Complete Determinism
- Genesis blocks must be 100% reproducible from chain specification
- Same inputs must always produce identical outputs
- No randomness except from specified deterministic seeds

### 2. Post-Quantum Security
- All cryptographic operations use post-quantum algorithms
- Dilithium2 signatures for block authentication
- BLAKE3 hashing throughout

### 3. Immutable Parameters
- Genesis parameters are locked once mainnet launches
- Chain specification serves as the single source of truth
- All modifications require new network (hard fork)

### 4. Comprehensive Verification
- Multi-level validation system
- Cryptographic integrity checks
- Economic model compliance verification

## Genesis Block Structure

### Block Header
```rust
pub struct BlockHeader {
    pub version: u32,                    // Block format version
    pub previous_hash: [u8; 32],         // All zeros for genesis
    pub merkle_root: [u8; 32],           // Root of transaction tree
    pub timestamp: DateTime<Utc>,        // Genesis timestamp
    pub difficulty: u32,                 // Initial difficulty target
    pub nonce: u64,                      // Proof-of-work nonce
    pub extra_nonce: Vec<u8>,            // Extended nonce space
}
```

### Genesis Transactions

#### Coinbase Transaction
- **Type**: `TransactionType::Coinbase`
- **Purpose**: Initial block reward (if no premine)
- **Amount**: Specified in `supply.initial_reward`
- **Address**: Standard genesis address format

#### Allocation Transactions
- **Type**: `TransactionType::Allocation { purpose }`
- **Purpose**: Initial coin distributions
- **Amount**: Specified in chain spec allocations
- **Address**: Recipient address for each allocation

### Cryptographic Elements

#### Post-Quantum Signature
```rust
pub struct QuantumSignature {
    pub signature: Vec<u8>,              // Dilithium2 signature
    pub public_key: Vec<u8>,             // Dilithium2 public key
    pub algorithm: String,               // "dilithium2"
}
```

#### Merkle Tree
- **Algorithm**: BLAKE3-based Merkle tree
- **Leaf Hashing**: Double BLAKE3 for enhanced security
- **Branch Combining**: Concatenation + Double BLAKE3
- **Proof Generation**: Standard Merkle proof format

## Deterministic Generation Process

### 1. Seed Generation
```rust
fn generate_genesis_seed(
    chain_name: &str,
    network_magic: &[u8; 4], 
    timestamp: u64
) -> [u8; 32]
```

**Inputs:**
- Chain name from network config
- Network magic bytes
- Genesis timestamp (Unix seconds)
- Fixed salt: "QuantumCoin Genesis Seed v2.0"

**Output:**
- 32-byte deterministic seed

### 2. Key Generation
- Deterministic Dilithium2 keypair from seed
- Same seed always produces same keys
- Keys used for block signing

### 3. Transaction Creation
1. Create coinbase transaction with deterministic hash
2. Create allocation transactions (if any)
3. Calculate transaction hashes deterministically

### 4. Merkle Tree Construction
- Build tree from transaction hashes
- Calculate merkle root
- Store complete tree for proof generation

### 5. Block Assembly
- Create block header with merkle root
- Calculate block hash
- Sign block with generated keypair

## Chain Specification Integration

### Required Parameters
From `chain_spec.toml`:

```toml
[network]
name = "quantumcoin"
symbol = "QTC"
decimals = 8

[consensus] 
genesis_difficulty = 0x1d00ffff

[supply]
max_supply = 22000000000000000
initial_reward = 5000000000
premine = 0

[genesis]
timestamp = "2025-01-15T00:00:00Z"
message = "QuantumCoin Genesis Message"

[[genesis.allocations]]
address = "qtc1q..."
amount = 1000000000
purpose = "Development fund"
```

### Validation Rules
1. Total allocations ≤ max_supply
2. Genesis timestamp must be fixed
3. Network magic bytes must match
4. Post-quantum parameters must be consistent

## File Formats

### JSON Format (Human Readable)
```json
{
  "header": {
    "version": 1,
    "previous_hash": "0000...0000",
    "merkle_root": "abc123...",
    "timestamp": "2025-01-15T00:00:00Z",
    "difficulty": 486604799,
    "nonce": 0,
    "extra_nonce": [0, 1, 2, 3]
  },
  "transactions": [...],
  "merkle_tree": {...},
  "hash": "def456...",
  "signature": {
    "signature": "789abc...",
    "public_key": "fed321...",
    "algorithm": "dilithium2"
  }
}
```

### Binary Format (Efficient Storage)
- Uses `bincode` serialization
- Compact representation
- Preserves all data integrity

### Hex Format (Debug/Inspection)
- Binary format encoded as hexadecimal
- Easy to inspect and compare
- Copy-paste friendly

## Verification Process

### Level 1: Structure Validation
- Block hash calculation correctness
- Merkle root consistency
- Transaction hash verification
- Basic format compliance

### Level 2: Chain Spec Compliance
- Network magic bytes match
- Genesis timestamp exact match
- Difficulty target correctness
- Supply constraints compliance

### Level 3: Cryptographic Verification
- Post-quantum signature validation
- Hash algorithm consistency
- Merkle tree proof verification
- Key derivation correctness

### Level 4: Economic Model Validation
- Total supply constraints
- Allocation amount verification
- Premine compliance
- Address format validation

## Reproduction Requirements

### Exact Match Criteria
Two genesis blocks are considered identical if:
1. Block hashes match exactly
2. Merkle roots match exactly  
3. All transaction hashes match
4. Signatures match (deterministic mode)

### Reproduction Steps
1. Load identical chain specification
2. Use identical generation parameters
3. Run deterministic generation process
4. Compare all cryptographic hashes
5. Verify signature matches

## Security Considerations

### Post-Quantum Resistance
- Dilithium2 provides NIST Level 2 security
- BLAKE3 provides 256-bit security
- All algorithms quantum-resistant

### Deterministic Security
- Seed generation uses multiple entropy sources
- No weak pseudorandom generators
- Cryptographically secure deterministic RNG

### Immutability Guarantees
- Chain spec hash embedded in genesis
- Signature prevents tampering
- Merkle tree ensures transaction integrity

## Integration Guidelines

### Blockchain Node Integration
```rust
// Load genesis block during node initialization
let genesis = quantumcoin_genesis::create_mainnet_genesis()?;
blockchain.initialize_with_genesis(genesis)?;
```

### Wallet Integration
```rust
// Verify genesis block during wallet setup
let genesis = load_genesis_from_config();
let valid = quantumcoin_genesis::verify_genesis_block(&genesis, &chain_spec)?;
assert!(valid, "Invalid genesis block");
```

### Network Protocol
- Genesis hash used for network identification
- Prevents nodes from different networks connecting
- Serves as chain identity fingerprint

## Testing and Quality Assurance

### Automated Testing
- Unit tests for all components
- Property-based testing for determinism
- Integration tests with full chain spec
- Cross-platform compatibility tests

### Reproduction Testing
- Multiple independent reproductions
- Hash comparison verification
- Signature consistency checks
- Platform independence verification

### Performance Requirements
- Genesis generation: < 10 seconds
- Genesis verification: < 1 second
- Memory usage: < 100 MB
- Concurrent verification support

## Maintenance and Updates

### Version Management
- Semantic versioning for genesis crate
- Chain spec version tracking
- Backward compatibility requirements

### Security Updates
- Regular cryptographic library updates
- Security audit integration
- Vulnerability response process

### Documentation Maintenance
- Specification updates with implementation
- Example code maintenance
- Best practices evolution

## Reference Implementation

The reference implementation is provided in the `quantumcoin-genesis` crate:

- **Repository**: https://github.com/aeonith/quantumcoin-ui-
- **Crate**: `crates/genesis/`
- **CLI Tool**: `genesis-cli`
- **Documentation**: This specification

## Appendices

### A. Cryptographic Parameters
- **Hash Algorithm**: BLAKE3
- **Signature Algorithm**: Dilithium2
- **Key Size**: 2528 bytes (private), 1312 bytes (public)
- **Signature Size**: 2420 bytes

### B. Network Constants
- **Mainnet Magic**: [0x51, 0x54, 0x43, 0x4D] ("QTCM")
- **Testnet Magic**: [0x51, 0x54, 0x43, 0x54] ("QTCT")
- **Genesis Timestamp**: 2025-01-15T00:00:00Z (Mainnet)

### C. Example CLI Usage
```bash
# Generate mainnet genesis
genesis-cli mainnet --output mainnet_genesis.json

# Verify genesis block
genesis-cli verify --genesis mainnet_genesis.json --detailed

# Reproduce from existing block
genesis-cli reproduce --genesis mainnet_genesis.json --output reproduced.json
```

---

**Document Version**: 2.0.0  
**Last Updated**: 2025-01-15  
**Author**: QuantumCoin Development Team  
**Status**: Final Specification
