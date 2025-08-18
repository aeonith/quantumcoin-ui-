# QuantumCoin ⚛️

[![CI](https://github.com/aeonith/quantumcoin-ui-/actions/workflows/ci.yml/badge.svg)](https://github.com/aeonith/quantumcoin-ui-/actions/workflows/ci.yml)
[![CodeQL](https://github.com/aeonith/quantumcoin-ui-/actions/workflows/codeql.yml/badge.svg)](https://github.com/aeonith/quantumcoin-ui-/actions/workflows/codeql.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A quantum-resistant cryptocurrency built with post-quantum cryptography and a sustainable economic model.

## 🚀 Quick Start

### 5-Minute Demo (Docker)

```bash
# Clone repository
git clone https://github.com/aeonith/quantumcoin-ui-.git
cd quantumcoin-ui-

# Start with Docker Compose
docker-compose up

# Open browser
open http://localhost:3000
```

### Local Development

```bash
# Prerequisites: Node.js 18+, Rust, pnpm

# Install dependencies
pnpm install

# Start development servers
make dev

# Or manually:
cd ui && pnpm dev &
cargo run --bin quantumcoin
```

## 📊 Economics Overview

| Parameter | Value |
|-----------|-------|
| **Total Supply** | 22,000,000 QTC |
| **Halving Period** | 2 years |
| **Total Duration** | 66 years |
| **Block Time** | 10 minutes (600s) |
| **Genesis Premine** | 0 QTC (No pre-mining) |
| **Development Fund** | 0 QTC (No pre-allocation) |
| **Total Mineable** | 22,000,000 QTC (100%) |
| **Algorithm** | Proof of Work |

### Issuance Schedule

![Issuance Curve](ui/public/issuance-curve.svg)

The supply follows a halving schedule every 2 years, with 33 total halvings over 66 years. This creates a sustainable, predictable monetary policy that approaches the maximum supply asymptotically.

## 🔒 Security Features

### Post-Quantum Cryptography
- **Dilithium2** signatures for quantum resistance
- Future-proof against quantum computing threats
- NIST-standardized algorithms

### RevStop Protection
- **Per-wallet freeze capability** for compromised accounts
- Cannot affect other users' funds
- Requires password authentication to disable
- **Default OFF** on exchanges (compliance-ready)

### Supply Chain Security
- SBOM (Software Bill of Materials) for all releases
- Container images signed with `cosign`
- Dependencies regularly audited with `cargo-audit`

## 🏗️ Architecture

```
quantumcoin-ui-/
├── crates/           # Rust workspace
│   ├── node/         # Blockchain node
│   ├── wallet/       # Wallet with PQ crypto
│   └── cli/          # Command-line interface
├── services/         # Service implementations
│   ├── explorer/     # Block explorer API (Rust)
│   └── explorer-proxy/ # Node.js fallback
├── ui/               # Next.js web interface
├── config/           # Canonical configuration
└── docs/             # Documentation
```

### Technology Stack
- **Backend**: Rust (async/tokio, axum, sqlx)
- **Frontend**: Next.js 14, TypeScript, TailwindCSS
- **Database**: SQLite (dev) / PostgreSQL (prod)
- **Crypto**: Dilithium2 post-quantum signatures
- **Deployment**: Docker, Kubernetes

## 🌐 API Reference

The explorer API follows OpenAPI 3.0 specification:

```bash
# Network status
curl http://localhost:8080/status

# Recent blocks
curl http://localhost:8080/blocks?limit=10

# Block details
curl http://localhost:8080/blocks/12345

# Transaction info
curl http://localhost:8080/tx/{hash}

# Address balance
curl http://localhost:8080/address/{addr}
```

Full API documentation: [OpenAPI Spec](openapi/openapi.yaml)

## 🧪 Testing

```bash
# Run all tests
make test

# Individual test suites
cargo test              # Rust tests
cd ui && pnpm test     # Frontend tests
cd ui && pnpm test:e2e # E2E tests

# Smoke tests
make smoke
```

## 🚦 Getting Started

### Testnet Quickstart

1. **Get testnet coins** from faucet: `curl -X POST https://faucet.testnet.quantumcoincrypto.com/request`
2. **Generate wallet**: `./quantumcoin-node wallet generate`
3. **Send transaction**: `./quantumcoin-node wallet send <address> <amount>`
4. **Mine blocks**: `./quantumcoin-node mine --address <your_address>`
5. **Verify in explorer**: https://testnet-explorer.quantumcoincrypto.com

### Mainnet

🟢 **Mainnet is LIVE** - Chain ID: `qtc-mainnet-1`

**DNS Seeds:**
- `seed1.quantumcoincrypto.com`
- `seed2.quantumcoincrypto.com` 
- `seed3.quantumcoincrypto.com`

**Confirmations Policy:**
- Small amounts (<1 QTC): 1 confirmation
- Medium amounts (1-100 QTC): 3 confirmations
- Large amounts (>100 QTC): 6 confirmations

**Network Information:**
- Default port: 8333
- RPC port: 8332
- Magic bytes: `0x51544343` ("QTCM")
- Address prefix: `qtc1`

**Quick Connect:**
```bash
# Download latest release
wget https://github.com/aeonith/quantumcoin-ui-/releases/download/v1.0.1-mainnet/quantumcoin-node-linux-x64.tar.gz
tar -xzf quantumcoin-node-linux-x64.tar.gz
./quantumcoin-node --addnode=seed1.quantumcoincrypto.com
```

## 📱 Wallet Integration

### Trust Wallet
- Chain ID: `quantumcoin-mainnet-v2`
- Symbol: `QTC`
- Decimals: `8`
- Logo: [Download Assets](listing/)

See [Trust Wallet Submission Checklist](TRUST_WALLET_LISTING_CHECKLIST.md) for full details.

### Exchange Listing
- **RevStop**: Disabled by default for exchanges
- **Withdrawal**: Standard UTXO model
- **Deposits**: Standard address-based
- **Confirmations**: 6 blocks recommended

## 🔧 Commands

```bash
# Development
make dev              # Start development environment
make test             # Run all tests
make build            # Build all components
make lint             # Lint code

# Docker
make up               # Start with docker-compose
make down             # Stop containers
make logs             # View logs

# Maintenance
make clean            # Clean build artifacts
make audit            # Security audit
make sbom             # Generate SBOM
```

## 📚 Documentation

- [Development Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [API Reference](openapi/openapi.yaml)
- [Economics Whitepaper](docs/brief-whitepaper.md)
- [Threat Model](docs/threat-model.md)
- [Deployment Guide](docs/runbooks/)

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) first.

### Development Process
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Standards
- **Rust**: `cargo fmt`, `cargo clippy`
- **TypeScript**: ESLint + Prettier
- **Commits**: Conventional Commits
- **Tests**: Required for new features

## 🏷️ RevStop Clarification

**RevStop is a per-wallet security feature that:**
- ✅ Allows wallet owners to freeze their own funds if compromised
- ✅ Requires password authentication to disable
- ✅ Is OFF by default for exchange integrations
- ❌ **Cannot** freeze other users' funds
- ❌ **Cannot** affect network consensus
- ❌ **Is not** a global kill switch

This is similar to account recovery features in traditional banking, but implemented in a decentralized way.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Website**: https://quantumcoin.network
- **Explorer**: https://explorer.quantumcoin.network
- **GitHub**: https://github.com/aeonith/quantumcoin-ui-
- **Discord**: https://discord.gg/quantumcoin
- **Twitter**: [@QuantumCoinDev](https://x.com/aeonith22?s=21)

---

**⚠️ Disclaimer**: QuantumCoin is experimental software. Use at your own risk. Not financial advice.
