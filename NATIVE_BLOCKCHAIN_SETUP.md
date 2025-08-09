# QuantumCoin Native Blockchain Setup Guide

## Prerequisites

### 1. Install Rust Toolchain
```bash
# Windows (run in PowerShell as Administrator)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update

# Verify installation
cargo --version
rustc --version
```

### 2. Install Node.js (for frontend)
Download and install from: https://nodejs.org/
```bash
# Verify installation
node --version
npm --version
```

## Building the Blockchain

### 1. Build Rust Backend
```bash
cd quantumcoin-ui-
cargo build --release
```

### 2. Build Integrated Node
```bash
cargo build --release --bin quantumcoin-integrated
```

## Running the Network

### 1. Start Genesis Node
```bash
# Start the first node (genesis)
cargo run --release --bin quantumcoin-integrated node --genesis --listen 0.0.0.0:8333 --rpc-port 8332
```

### 2. Start Additional Nodes
```bash
# Start peer nodes
cargo run --release --bin quantumcoin-integrated node --peer 127.0.0.1:8333 --listen 0.0.0.0:8334 --rpc-port 8333
```

### 3. Start Mining
```bash
# Start mining on a node
cargo run --release --bin quantumcoin-integrated mine --threads 4 --address QTC1234567890abcdef
```

## Testing the Network

### 1. Check Node Status
```bash
curl -X POST http://localhost:8332/rpc \
  -H "Content-Type: application/json" \
  -d '{"method": "getblockchaininfo", "params": [], "id": 1}'
```

### 2. Get Wallet Balance
```bash
curl -X POST http://localhost:8332/rpc \
  -H "Content-Type: application/json" \
  -d '{"method": "getbalance", "params": [], "id": 1}'
```

### 3. Send Transaction
```bash
curl -X POST http://localhost:8332/rpc \
  -H "Content-Type: application/json" \
  -d '{"method": "sendtoaddress", "params": ["QTC_ADDRESS", 1.0], "id": 1}'
```

## Frontend Interface

### 1. Start Development Server
```bash
# Install dependencies
npm install

# Start dev server
npm run dev
```

### 2. Access Web Interface
- Main UI: http://localhost:3000
- Block Explorer: http://localhost:3000/explorer.html
- Mining Interface: http://localhost:3000/mining.html
- Wallet: http://localhost:3000/wallet.html

## Production Deployment

### 1. Build for Production
```bash
cargo build --release
npm run build
```

### 2. Deploy to Server
Follow the PRODUCTION_DEPLOYMENT.md guide for complete deployment instructions.

## Trust Wallet Integration

### 1. Submit Blockchain Info
- Use TRUST_WALLET_BLOCKCHAIN_INFO.json
- Submit to Trust Wallet assets repository
- Include logo files per QUANTUMCOIN_LOGO_SPECS.md

### 2. Network Requirements
- Stable mainnet running for 3+ months
- Active community and development
- Security audit completed
- Documentation and explorer available

## Troubleshooting

### Common Issues
1. **Cargo not found**: Install Rust toolchain
2. **Compilation errors**: Update Rust with `rustup update`
3. **Network connection issues**: Check firewall settings
4. **RPC errors**: Verify node is running and ports are open

### Support
- GitHub Issues: https://github.com/aeonith/quantumcoin-ui-/issues
- Documentation: See PRODUCTION_DEPLOYMENT.md
- Community: Discord/Telegram (to be created)
