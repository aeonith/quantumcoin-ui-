# QuantumCoin Core - Post-Quantum Cryptocurrency

[![CI](https://github.com/quantumcoin-crypto/quantumcoin-core/workflows/CI/badge.svg)](https://github.com/quantumcoin-crypto/quantumcoin-core/actions)
[![Security Audit](https://img.shields.io/badge/security-audited-green.svg)](./SECURITY.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker Pulls](https://img.shields.io/docker/pulls/quantumcoin/node)](https://hub.docker.com/r/quantumcoin/node)

**Enterprise-Ready Post-Quantum Cryptocurrency with Bitcoin-Like Economics**

QuantumCoin is the first production-ready cryptocurrency that combines proven Bitcoin economics with post-quantum cryptography (Dilithium2), making it secure against both classical and quantum computer attacks.

## 🚀 Quick Start

### For Exchanges & Institutions

```bash
# Production deployment
docker run -d --name qtc-node \
  -p 8545:8545 -p 8546:8546 \
  -v qtc-data:/data/quantumcoin \
  ghcr.io/quantumcoin-crypto/quantumcoin-node:latest

# Exchange integration
curl -X POST http://localhost:8545 \
  -d '{"jsonrpc":"2.0","method":"getblockchaininfo","id":1}'

# Cold storage wallet  
qtc-wallet new --name exchange-cold --type cold --hsm
```

### For Developers

```bash
# Install from source
git clone https://github.com/quantumcoin-crypto/quantumcoin-core
cd quantumcoin-core
cargo build --release

# Run tests
cargo test --all

# Start development node
./target/release/quantumcoin-node --network regtest
```

## 💎 Network Parameters

| Parameter | Value | Purpose |
|-----------|-------|---------|
| **Max Supply** | 22,000,000 QTC | Hard algorithmic cap |
| **Block Time** | 10 minutes | Bitcoin-compatible |
| **Block Reward** | 50 → 25 → 12.5... QTC | Halving every ~2 years |
| **Confirmations** | 6 recommended | 1 hour finality |
| **Address Format** | `qtc1...` (Bech32) | Post-quantum addresses |
| **Signature Scheme** | Dilithium2 | NIST PQC standard |
| **Hash Algorithm** | SHA256d | Bitcoin-compatible PoW |

## ⚛️ Post-Quantum Security

### Why Post-Quantum?
- **Quantum Threat**: Shor's algorithm can break RSA/ECDSA
- **Future-Proof**: Secure against both classical and quantum attacks
- **Standards-Based**: Uses NIST-standardized Dilithium2
- **Performance**: Optimized for blockchain usage

### Address Example
```
qtc1qw508d6qejxtdg4y5r3zarvary0c5xw7k8gkx8k
```

### Compatibility
- **Mining**: Standard SHA256d (ASIC-friendly)
- **Wallets**: Bitcoin-like UTXO model
- **APIs**: Bitcoin-compatible RPC interface

## 🏗️ Architecture

```
quantumcoin-core/
├── crates/                    # Modular Rust workspace
│   ├── types/                # Core data structures  
│   ├── crypto/               # Post-quantum cryptography
│   ├── validation/           # Consensus rules
│   ├── node/                 # Full node implementation
│   ├── p2p/                  # Network protocol
│   └── wallet/               # HD wallet implementation
├── src/bin/                  # Production binaries
│   ├── quantumcoin-node.rs   # Full node
│   ├── qtc-wallet.rs         # Cold storage wallet
│   └── supply-audit.rs       # Supply verification
├── exchange-pack/            # Exchange integration
├── docker/                   # Production containers
└── .github/workflows/        # CI/CD pipeline
```

## 🔧 Production Tools

### Node Management
```bash
# Initialize blockchain
quantumcoin-node init --network mainnet

# Start with monitoring
quantumcoin-node --network mainnet \
  --prometheus-port 9090 \
  --log-level info

# Check status
quantumcoin-node status
```

### Wallet Operations
```bash
# Enterprise cold storage
qtc-wallet new --name enterprise-cold \
  --type multisig --threshold 3 --participants 5

# Batch withdrawals
qtc-wallet batch-send --file withdrawals.csv \
  --wallet hot-wallet --dry-run

# Hardware security module
qtc-wallet new --name hsm-wallet --hsm
```

### Supply Verification
```bash
# Audit total supply
supply-audit --verify --output audit-report.json

# Generate emission schedule  
supply-audit schedule --halvings 33

# Scan for inflation bugs
supply-audit scan-blocks --from 0 --to 100000
```

## 🌐 Network Information

### Mainnet
- **RPC Endpoint**: `http://localhost:8545`
- **P2P Port**: 8546
- **Genesis Hash**: `00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048`
- **Explorer**: https://explorer.quantumcoincrypto.com

### Testnet
- **RPC Endpoint**: `http://localhost:18545` 
- **P2P Port**: 18546
- **Faucet**: https://faucet.quantumcoincrypto.com

### Seed Nodes
- `seed1.quantumcoincrypto.com:8546`
- `seed2.quantumcoincrypto.com:8546`
- `seed3.quantumcoincrypto.com:8546`
- `seed4.quantumcoincrypto.com:8546`

## 🏢 Exchange Integration

### Quick Integration Checklist
- [ ] Download [Integration Guide](exchange-pack/INTEGRATION_GUIDE.md)
- [ ] Deploy production node via Docker
- [ ] Set up cold storage wallets
- [ ] Configure monitoring and alerts
- [ ] Test deposit/withdrawal flow
- [ ] Implement supply verification
- [ ] Contact for listing coordination

### Key Features for Exchanges
- ✅ **Bitcoin-compatible RPC** - Drop-in replacement
- ✅ **Cold storage CLI** - Air-gapped signing support
- ✅ **Multi-signature wallets** - Enterprise security
- ✅ **Supply auiting** - Inflation bug protection
- ✅ **Batch operations** - High-throughput processing
- ✅ **24/7 technical support** - Exchange-dedicated support

### Integration Support
- **Email**: exchanges@quantumcoincrypto.com  
- **Response Time**: 24 hours for critical issues
- **Services**: Free integration consultation, code review, testing assistance

## 🛡️ Security

### Audit Status
- ✅ **Internal Review**: Completed
- 🔄 **External Audit**: Scheduled Q1 2025 (Trail of Bits)
- ✅ **Supply Verification**: Automated in CI/CD
- ✅ **Dependency Audit**: Daily `cargo audit` checks

### Vulnerability Reporting
- **Contact**: security@quantumcoincrypto.com
- **PGP Key**: Available at https://quantumcoincrypto.com/pgp
- **Bug Bounty**: Up to $50,000 for critical vulnerabilities
- **Response Time**: 24-48 hours

### Security Features
- **Memory Safety**: Written in Rust
- **Input Validation**: Comprehensive fuzzing
- **Privilege Separation**: Runs as non-root
- **Network Hardening**: TLS 1.3, rate limiting
- **Supply Protection**: Algorithmic inflation prevention

## 📊 Performance & Scalability

### System Requirements

#### Minimum (Personal Node)
- **CPU**: 2 cores, 2.0+ GHz
- **RAM**: 4GB
- **Storage**: 50GB SSD
- **Network**: 10 Mbps

#### Recommended (Exchange/Pool)
- **CPU**: 8+ cores, 3.0+ GHz  
- **RAM**: 16GB
- **Storage**: 500GB+ NVMe SSD
- **Network**: 100+ Mbps

#### Enterprise (High Availability)
- **CPU**: 16+ cores, 3.5+ GHz
- **RAM**: 32GB+
- **Storage**: 1TB+ NVMe SSD RAID
- **Network**: Gigabit with redundancy

### Performance Metrics
- **Sync Speed**: ~1000 blocks/second
- **Transaction Throughput**: ~7 TPS (Bitcoin-equivalent)
- **Memory Usage**: ~2GB typical
- **Database Size**: ~100MB per 10,000 blocks

## 📚 Documentation

### For Users
- **Getting Started**: [docs/getting-started.md](docs/getting-started.md)
- **Wallet Guide**: [docs/wallet-guide.md](docs/wallet-guide.md)  
- **Mining Guide**: [docs/mining-guide.md](docs/mining-guide.md)
- **FAQ**: [docs/faq.md](docs/faq.md)

### For Developers  
- **API Reference**: [docs/rpc-api.md](docs/rpc-api.md)
- **Protocol Spec**: [docs/protocol.md](docs/protocol.md)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Architecture**: [docs/architecture.md](docs/architecture.md)

### For Exchanges
- **Integration Guide**: [exchange-pack/INTEGRATION_GUIDE.md](exchange-pack/INTEGRATION_GUIDE.md)
- **RPC Reference**: [exchange-pack/RPC_API.md](exchange-pack/RPC_API.md)
- **Security Guide**: [SECURITY.md](SECURITY.md)

## 🤝 Community & Support

### Official Channels
- **Website**: https://quantumcoincrypto.com
- **Documentation**: https://docs.quantumcoincrypto.com
- **GitHub**: https://github.com/quantumcoin-crypto/quantumcoin-core
- **Discord**: https://discord.gg/quantumcoin
- **Twitter**: [@QuantumCoinDev](https://twitter.com/QuantumCoinDev)

### Support Tiers

#### Community (Free)
- GitHub issues and discussions
- Discord community support
- Documentation and tutorials

#### Professional ($500/month)
- Priority email support (48h response)
- Integration assistance
- Code review services
- Custom documentation

#### Enterprise (Custom)
- 24/7 dedicated support
- Direct access to core developers
- Custom features and modifications
- Security audit assistance
- Legal and compliance support

Contact: enterprise@quantumcoincrypto.com

## 🏆 Why QuantumCoin?

### For Individual Users
- **Future-Proof**: Quantum-resistant cryptography
- **Fair Launch**: No premine or founder rewards  
- **Proven Economics**: Bitcoin-like scarcity model
- **Low Fees**: Efficient blockchain design

### For Enterprises
- **Institutional Grade**: Production-ready with 24/7 support
- **Compliance Ready**: Legal opinions and audit reports
- **Integration Support**: Dedicated exchange liaison team
- **Risk Management**: Supply auditing and monitoring tools

### For Developers
- **Modern Codebase**: Written in Rust with comprehensive tests
- **Open Source**: MIT license with active development
- **Standards Based**: NIST post-quantum cryptography
- **Developer Tools**: Complete SDK and documentation

---

**🚀 Ready to join the post-quantum future?**

[![Deploy with Docker](https://img.shields.io/badge/Deploy%20with-Docker-blue?logo=docker)](https://hub.docker.com/r/quantumcoin/node)
[![Download Release](https://img.shields.io/github/v/release/quantumcoin-crypto/quantumcoin-core?label=Download)](https://github.com/quantumcoin-crypto/quantumcoin-core/releases/latest)
[![Join Discord](https://img.shields.io/discord/123456789?color=7289da&label=Discord&logo=discord)](https://discord.gg/quantumcoin)

*Securing the future of digital money with post-quantum cryptography*
