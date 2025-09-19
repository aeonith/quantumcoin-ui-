# QuantumCoin Blockchain ⚛️

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A quantum-resistant cryptocurrency blockchain implementation built with post-quantum cryptography and Rust. This repository contains the **core blockchain protocol only** - website and frontend components are maintained in a separate repository.

## 🚀 Quick Start

### Build from Source

```bash
# Prerequisites: Rust 1.70+
git clone https://github.com/aeonith/quantumcoin-ui-.git
cd quantumcoin-ui-

# Build the blockchain node
cargo build --release

# Run the node
cargo run --release --bin quantumcoin-node

# Build API server
cd backend && cargo run --release
```

### Docker Deployment

```bash
# Build container
docker build -t quantumcoin-node .

# Run blockchain node
docker run -p 8333:8333 -p 8332:8332 quantumcoin-node
```

## 📊 Blockchain Specifications

| Parameter | Value |
|-----------|-------|
| **Total Supply** | 22,000,000 QTC |
| **Halving Period** | 2 years (105,120 blocks) |
| **Block Time** | 10 minutes (600 seconds) |
| **Genesis Allocation** | 0 QTC (Fair Launch) |
| **Mining Algorithm** | Proof of Work (SHA-256d) |
| **Difficulty Adjustment** | ASERT (Absolutely Scheduled Exponentially Rising Targets) |
| **Address Format** | Bech32 (`qtc1...`) |

### Economic Model

- **Fair Launch**: No premine, no founder allocation
- **Halving Schedule**: Every 105,120 blocks (~2 years)
- **Total Halvings**: 33 over 66 years
- **Asymptotic Supply**: Approaches 22M QTC limit

## 🔒 Post-Quantum Security

### Cryptographic Primitives
- **Digital Signatures**: Dilithium2 (NIST PQC standard)
- **Hash Function**: SHA-256 (double SHA-256 for mining)
- **Quantum Resistance**: Future-proof against Shor's algorithm

### RevStop™ Protection
- **Individual Wallet Control**: Per-address freeze capability
- **Password Protected**: Requires authentication to disable
- **Exchange Compliant**: OFF by default for institutional use
- **Non-Global**: Cannot affect other users' funds

## 🏗️ Architecture

```
quantumcoin-ui-/
├── src/               # Core blockchain implementation
│   ├── blockchain.rs  # Chain state and validation
│   ├── consensus.rs   # Consensus engine
│   ├── mining.rs      # Proof-of-work mining
│   ├── network.rs     # P2P networking
│   ├── rpc.rs         # RPC interface
│   └── wallet.rs      # Wallet functionality
├── crates/            # Modular components
│   ├── crypto/        # Post-quantum cryptography
│   ├── node/          # Full node implementation
│   ├── types/         # Blockchain data types
│   └── validation/    # Transaction validation
├── backend/           # API server (Rocket framework)
├── scripts/           # Utilities and tools
└── config/            # Network configuration
```

## 🌐 RPC Interface

### Node RPC Endpoints

```bash
# Network status
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"method":"getblockchaininfo","params":[],"id":1}'

# Block information
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"method":"getblock","params":["<block_hash>"],"id":1}'

# Transaction details
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"method":"gettransaction","params":["<tx_hash>"],"id":1}'

# Address balance
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"method":"getaddressbalance","params":["<address>"],"id":1}'
```

### Exchange-Compatible RPC

```bash
# Compatibility aliases for exchanges
# qc_getBalance, qc_getBlockByNumber, qc_sendTransaction
# Full compatibility layer documented in exchange-pack/RPC.md
```

## ⚙️ Configuration

### Chain Parameters (`chain_spec.toml`)

```toml
[network]
name = "QuantumCoin"
symbol = "QC"
decimals = 8

[consensus]
target_block_time_secs = 600
difficulty_adjustment = "ASERT"

[supply]
max_supply_sats = 22000000_00000000
halving_interval_blocks = 105120
premine_sats = 0  # Fair launch
```

## 🧪 Testing

```bash
# Run all blockchain tests
cargo test --workspace

# Run consensus tests specifically
cargo test --package quantumcoin-node consensus

# Run integration tests
cargo test --test integration_tests

# Stress testing
./run_extreme_test.sh
```

## 🚦 Network Deployment

### Mainnet Connection

```bash
# Connect to mainnet
cargo run --release --bin quantumcoin-node -- \
  --network=mainnet \
  --addnode=seed1.quantumcoincrypto.com \
  --addnode=seed2.quantumcoincrypto.com
```

### Testnet Development

```bash
# Run testnet node
cargo run --bin quantumcoin-node -- \
  --network=testnet \
  --rpcport=18332 \
  --port=18333
```

### Mining

```bash
# Solo mining
cargo run --bin quantumcoin-node -- \
  --mine \
  --mining-address=<your_qtc_address>

# Mining pool connection
cargo run --bin quantumcoin-node -- \
  --pool=stratum+tcp://pool.example.com:4444 \
  --pool-user=<username> \
  --pool-pass=<password>
```

## 🔧 Development Commands

```bash
# Build all components
cargo build --workspace --release

# Run clippy linting
cargo clippy --workspace --all-features

# Format code
cargo fmt --all

# Security audit
cargo audit

# Generate documentation
cargo doc --workspace --no-deps
```

## 📚 Documentation

- [**REPOSITORY_STRUCTURE.md**](REPOSITORY_STRUCTURE.md) - Repository organization
- [**SECURITY.md**](SECURITY.md) - Security guidelines and audit checklist
- [**exchange-pack/**](exchange-pack/) - Exchange integration documentation
- [**docs/**](docs/) - Technical specifications
- [**AGENT.md**](AGENT.md) - Development guide and commands

## 🤝 Contributing

### Development Setup

1. **Install Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **Clone repository**: `git clone https://github.com/aeonith/quantumcoin-ui-.git`
3. **Build project**: `cargo build --workspace`
4. **Run tests**: `cargo test --workspace`

### Code Standards

- **Rust**: Use `cargo fmt` and `cargo clippy`
- **Commits**: Follow conventional commit format
- **Tests**: Add tests for new features
- **Documentation**: Update docs for API changes

### Pull Request Process

1. Fork the repository
2. Create feature branch: `git checkout -b feature/your-feature`
3. Make changes and add tests
4. Run CI checks: `cargo test && cargo clippy`
5. Submit pull request with clear description

## 📋 Exchange Integration

### Quick Integration Checklist

- ✅ **RPC Compatibility**: Standard Bitcoin-like RPC interface
- ✅ **Address Format**: Bech32 format (`qtc1...`)
- ✅ **Confirmations**: 6 blocks recommended for large amounts
- ✅ **RevStop**: Disabled by default for exchange addresses
- ✅ **UTXO Model**: Standard Bitcoin-like transaction model

Full integration guide: [exchange-pack/README.md](exchange-pack/)

## 🏷️ RevStop™ Clarification

RevStop is a **per-wallet security feature** that:

- ✅ Allows individual wallet owners to freeze their own funds
- ✅ Requires password authentication to disable
- ✅ Is disabled by default for exchange integrations  
- ❌ **Cannot** freeze other users' funds
- ❌ **Cannot** affect network consensus or mining
- ❌ **Is not** a global kill switch or central control

This provides individual account security similar to traditional banking recovery features, implemented in a decentralized manner.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Blockchain Repository**: https://github.com/aeonith/quantumcoin-ui-
- **Issue Tracker**: https://github.com/aeonith/quantumcoin-ui-/issues
- **Releases**: https://github.com/aeonith/quantumcoin-ui-/releases

---

**⚠️ Important**: This repository contains **only the blockchain core**. For website, explorer UI, or frontend components, please refer to the separate web repository.

**Disclaimer**: QuantumCoin is experimental blockchain software. Use at your own risk. This is not financial advice.
