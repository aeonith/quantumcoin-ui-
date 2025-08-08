# QuantumCoin™ 2.0 - Production Blockchain

A quantum-resistant cryptocurrency implementation built with Rust, featuring post-quantum cryptography, P2P networking, and enterprise-grade security.

## Features

### 🔐 Quantum-Resistant Security
- **Dilithium2** post-quantum digital signatures
- **SHA-3** hashing algorithms 
- **Argon2** key derivation
- Future-proof against quantum computer attacks

### 🌐 Distributed Network
- Full P2P node implementation
- Automatic peer discovery
- Blockchain synchronization
- Real-time transaction broadcasting

### ⛏️ Advanced Mining
- Proof-of-Work consensus with automatic difficulty adjustment
- Multi-threaded mining support
- Mempool management with fee prioritization
- Mining reward halving (Bitcoin-like economics)

### 💰 Economic Model
- **Total Supply**: 22 Million QTC
- **Block Reward**: 50 QTC (halving every 210,000 blocks)
- **Block Time**: ~10 minutes target
- **Minimum Fee**: 0.00001 QTC

### 🛡️ Security Features
- **RevStop**: Emergency transaction halt mechanism
- Double-spending protection
- UTXO validation
- Cryptographic transaction verification

## Quick Start

### Prerequisites
- Rust 1.70+
- Git

### Installation

```bash
git clone https://github.com/aeonith/quantumcoin-ui-
cd quantumcoin-ui-
cargo build --release
```

### Running a Node

```bash
# Start a full node with mining
./target/release/quantumcoin node --mine --mining-address YOUR_ADDRESS

# Start a node and connect to peers
./target/release/quantumcoin node --peers 192.168.1.100:8333,192.168.1.101:8333

# Custom port and mining threads
./target/release/quantumcoin node --port 8334 --mine --mining-address YOUR_ADDRESS
```

### Wallet Operations

```bash
# Generate new wallet
./target/release/quantumcoin wallet generate

# Check balance
./target/release/quantumcoin wallet balance YOUR_ADDRESS

# Send transaction
./target/release/quantumcoin wallet send FROM_ADDRESS TO_ADDRESS 100000000 --fee 1000
```

### Mining

```bash
# Start mining with 4 threads
./target/release/quantumcoin mine YOUR_MINING_ADDRESS --threads 4
```

### Blockchain Info

```bash
# Get blockchain information
./target/release/quantumcoin blockchain info

# Get specific block
./target/release/quantumcoin blockchain block BLOCK_HASH

# Get transaction details
./target/release/quantumcoin blockchain transaction TX_ID
```

## Architecture

### Core Components

- **Blockchain Engine**: Block validation, consensus, UTXO management
- **P2P Network**: Peer discovery, message routing, sync protocols
- **Mining System**: PoW algorithm, difficulty adjustment, reward distribution
- **Mempool**: Transaction prioritization, fee estimation, spam protection
- **Wallet**: Key management, transaction signing, balance tracking
- **RevStop**: Emergency halt mechanism for security incidents

### Network Protocol

QuantumCoin uses a custom binary protocol for P2P communication:

- **Handshake**: Version negotiation and peer identification
- **Block Sync**: Efficient blockchain synchronization
- **Transaction Relay**: Real-time transaction broadcasting
- **Peer Discovery**: Automatic network topology building

### Cryptography

- **Signatures**: Dilithium2 (NIST PQC standard)
- **Hashing**: SHA-3 (Keccak)
- **Key Derivation**: Argon2id
- **Address Format**: Base64-encoded public key hash

## Configuration

### Network Settings

Default configuration in `~/.quantumcoin/config.toml`:

```toml
[network]
port = 8333
max_peers = 100
seed_nodes = [
    "seed1.quantumcoin.io:8333",
    "seed2.quantumcoin.io:8333"
]

[mining]
target_block_time = 600  # 10 minutes
difficulty_adjustment_interval = 144  # ~1 day
max_difficulty_adjustment = 4.0

[mempool]
max_size = 10000
max_age_hours = 24
min_fee_per_byte = 1000

[security]
revstop_enabled = true
max_reorg_depth = 100
```

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo check
cargo clippy
```

### Architecture Overview

```
src/
├── blockchain.rs      # Core blockchain logic
├── transaction.rs     # Transaction validation
├── block.rs          # Block structure and validation
├── mining.rs         # Mining and PoW implementation
├── mempool.rs        # Transaction pool management
├── network/          # P2P networking
│   ├── node.rs       # Network node implementation
│   ├── peer.rs       # Peer connection handling
│   ├── message.rs    # Protocol messages
│   ├── sync.rs       # Blockchain synchronization
│   └── discovery.rs  # Peer discovery
├── quantum_crypto.rs # Post-quantum cryptography
├── wallet.rs         # Wallet functionality
├── revstop.rs        # Emergency halt mechanism
└── main.rs          # CLI application
```

## Security Considerations

### Quantum Resistance
QuantumCoin is designed to be secure against both classical and quantum computer attacks:

- **Current Security**: Classical cryptanalysis resistant
- **Future Security**: Quantum computer resistant via post-quantum cryptography
- **Upgrade Path**: Algorithm agility for future cryptographic advances

### Network Security
- Rate limiting on connections and messages
- Peer reputation system
- DDoS protection mechanisms
- Sybil attack resistance

### Economic Security
- Balanced mining economics to prevent centralization
- Fee market for transaction prioritization
- RevStop mechanism for emergency situations

## Performance

### Benchmarks
- **Transaction Throughput**: ~10 TPS (design target)
- **Block Validation**: <1s typical
- **Network Latency**: <5s block propagation
- **Memory Usage**: ~100MB typical node

### Optimization
- Efficient UTXO set management
- Parallel signature verification
- Optimized serialization with bincode
- Memory-mapped blockchain storage

## Roadmap

### Phase 1: Core Infrastructure ✅
- [x] Blockchain implementation
- [x] P2P networking
- [x] Mining system
- [x] CLI interface

### Phase 2: Advanced Features 🚧
- [ ] Web interface
- [ ] REST API
- [ ] Database integration
- [ ] Performance monitoring

### Phase 3: Ecosystem 📋
- [ ] Light clients
- [ ] Mobile wallets
- [ ] Exchange integration
- [ ] Smart contracts (future)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests
5. Submit a pull request

### Code Style
- Use `rustfmt` for formatting
- Follow Rust naming conventions
- Add documentation for public APIs
- Write comprehensive tests

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- **Documentation**: [docs.quantumcoin.io](https://docs.quantumcoin.io)
- **Issues**: [GitHub Issues](https://github.com/aeonith/quantumcoin-ui-/issues)
- **Discord**: [QuantumCoin Community](https://discord.gg/quantumcoin)
- **Email**: support@quantumcoincrypto.com

---

**QuantumCoin™** - Securing the future of digital currency with quantum-resistant technology.
