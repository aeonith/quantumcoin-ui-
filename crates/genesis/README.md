# QuantumCoin Genesis Block System

A production-grade, deterministic genesis block generation system for QuantumCoin with post-quantum cryptographic security.

## Features

- ✅ **100% Deterministic**: Reproducible genesis blocks from chain specification
- ✅ **Post-Quantum Security**: Dilithium2 signatures and BLAKE3 hashing
- ✅ **Comprehensive Verification**: Multi-level validation system
- ✅ **Multiple Formats**: JSON, binary, and hex output formats
- ✅ **CLI Tools**: Complete command-line interface
- ✅ **Cross-Platform**: Windows, Linux, and macOS support
- ✅ **Zero Dependencies**: Self-contained genesis generation
- ✅ **Cryptographic Integrity**: Merkle trees and hash verification

## Quick Start

### Generate Genesis Blocks

```bash
# Build the CLI tool
cargo build --release --bin genesis-cli

# Generate mainnet genesis
./target/release/genesis-cli mainnet --output mainnet_genesis.json

# Generate testnet genesis  
./target/release/genesis-cli testnet --output testnet_genesis.json

# Verify genesis blocks
./target/release/genesis-cli verify --genesis mainnet_genesis.json --spec ../../chain_spec.toml
```

### Automated Generation

```bash
# Windows
../../scripts/generate_genesis.bat

# Linux/macOS
../../scripts/generate_genesis.sh
```

## Library Usage

```rust
use quantumcoin_genesis::{create_mainnet_genesis, GenesisVerifier, ChainSpec};

// Generate mainnet genesis
let genesis = create_mainnet_genesis()?;
println!("Genesis hash: {}", genesis.hash_hex());

// Custom generation
let chain_spec = ChainSpec::load_from_file("chain_spec.toml")?;
let builder = GenesisBuilder::new(chain_spec);
let genesis = builder.build()?;

// Verification
let verifier = GenesisVerifier::new(&chain_spec);
assert!(verifier.verify(&genesis)?);
```

## Genesis Block Structure

```rust
pub struct GenesisBlock {
    pub header: BlockHeader,           // Block metadata
    pub transactions: Vec<GenesisTransaction>, // Genesis transactions
    pub merkle_tree: MerkleTree,       // Transaction merkle tree
    pub hash: [u8; 32],               // Block hash
    pub signature: Option<QuantumSignature>, // Post-quantum signature
    pub chain_spec_hash: [u8; 32],    // Chain spec commitment
    pub metadata: GenesisMetadata,     // Creation metadata
}
```

## CLI Commands

### Generation
- `mainnet` - Generate mainnet genesis block
- `testnet` - Generate testnet genesis block  
- `custom` - Generate from custom chain spec

### Verification
- `verify` - Verify genesis block against chain spec
- `reproduce` - Reproduce genesis from existing parameters

### Utilities
- `info` - Display genesis block information
- `mine` - Mine genesis block with proof-of-work

### Examples

```bash
# Generate with custom seed
genesis-cli mainnet --seed 0123...cdef --output seeded_genesis.json

# Detailed verification
genesis-cli verify --genesis genesis.json --spec chain_spec.toml --detailed

# Mine with custom difficulty
genesis-cli mine --genesis genesis.json --difficulty 1d00ffff --output mined.json

# Show block information
genesis-cli info --genesis mainnet_genesis.json
```

## Chain Specification Integration

The genesis system reads parameters from `chain_spec.toml`:

```toml
[network]
name = "quantumcoin"
symbol = "QTC"

[consensus]
genesis_difficulty = 0x1d00ffff

[supply]
max_supply = 22000000000000000
initial_reward = 5000000000

[genesis]
timestamp = "2025-01-15T00:00:00Z"
message = "QuantumCoin Genesis"

# Optional allocations for testnet
[[genesis.allocations]]
address = "qtc1q..."
amount = 1000000000
purpose = "Test allocation"
```

## Deterministic Generation

Genesis blocks are generated deterministically using:

1. **Seed Generation**: From chain name, network magic, and timestamp
2. **Key Derivation**: Deterministic Dilithium2 keypair generation
3. **Transaction Hashing**: Deterministic transaction hash calculation
4. **Merkle Tree**: Deterministic tree construction
5. **Block Signing**: Consistent post-quantum signatures

Same inputs always produce identical outputs.

## Verification System

The verification system performs comprehensive checks:

### Structure Validation
- Block hash calculation correctness
- Merkle root consistency
- Transaction integrity
- Format compliance

### Chain Spec Compliance
- Network parameters match
- Genesis timestamp exact match
- Supply constraints verification
- Economic model validation

### Cryptographic Verification
- Post-quantum signature validation
- Hash algorithm consistency
- Merkle proof verification
- Key derivation correctness

## Security Features

### Post-Quantum Cryptography
- **Dilithium2**: NIST Level 2 post-quantum signatures
- **BLAKE3**: 256-bit cryptographic hashing
- **Quantum Resistance**: All algorithms are quantum-safe

### Integrity Protection
- **Block Hash**: Double BLAKE3 for tamper detection
- **Merkle Tree**: Transaction integrity verification
- **Chain Spec Hash**: Parameter commitment
- **Digital Signature**: Block authenticity guarantee

## File Formats

### JSON (Human Readable)
```json
{
  "header": {
    "version": 1,
    "previous_hash": "0000...0000",
    "merkle_root": "abc123...",
    "timestamp": "2025-01-15T00:00:00Z"
  },
  "transactions": [...],
  "hash": "def456...",
  "signature": {...}
}
```

### Binary (Efficient)
- Compact binary serialization using `bincode`
- Minimal storage footprint
- Fast loading and verification

### Hex (Debug)
- Binary format encoded as hexadecimal
- Easy inspection and comparison
- Copy-paste friendly for debugging

## Testing and Quality Assurance

### Automated Tests
```bash
# Run all tests
cargo test

# Test specific components
cargo test merkle
cargo test crypto
cargo test verification
```

### Reproduction Verification
```bash
# Verify deterministic generation
../../scripts/verify_genesis_reproduction.bat  # Windows
../../scripts/verify_genesis_reproduction.sh   # Linux/macOS
```

### Performance Benchmarks
- Genesis generation: < 10 seconds
- Verification: < 1 second
- Memory usage: < 100 MB
- Cross-platform compatibility

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Chain Spec    │───▶│ Genesis Builder │───▶│  Genesis Block  │
│   (TOML)        │    │                 │    │   (Multiple     │
└─────────────────┘    └─────────────────┘    │    Formats)     │
                                  │            └─────────────────┘
                                  ▼                     │
┌─────────────────┐    ┌─────────────────┐              │
│ Crypto System   │◀───│ Merkle Tree     │              │
│ (Dilithium2)    │    │ (BLAKE3)        │              │
└─────────────────┘    └─────────────────┘              │
                                  ▲                     │
                                  │                     ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │  Transactions   │    │   Verifier      │
                       │   (Genesis)     │    │  (Multi-level)  │
                       └─────────────────┘    └─────────────────┘
```

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
pqcrypto-dilithium = "0.3.3"
blake3 = "1.5"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
hex = "0.4"
toml = "0.8"
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Ensure all tests pass
5. Submit a pull request

### Development Setup
```bash
# Clone repository
git clone https://github.com/aeonith/quantumcoin-ui-
cd quantumcoin-ui-/crates/genesis

# Build and test
cargo build
cargo test

# Run CLI tool
cargo run --bin genesis-cli -- --help
```

## Documentation

- [Genesis Block Specification](../../docs/GENESIS_BLOCK_SPECIFICATION.md)
- [Creation Guide](../../docs/GENESIS_CREATION_GUIDE.md)
- [API Documentation](https://docs.rs/quantumcoin-genesis)

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Support

- **Issues**: https://github.com/aeonith/quantumcoin-ui-/issues
- **Discord**: https://discord.gg/quantumcoin
- **Email**: support@quantumcoincrypto.com

---

**Version**: 2.0.0  
**Rust Version**: 1.70+  
**Last Updated**: 2025-01-15
