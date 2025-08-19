# QuantumCoin Exchange Verification Checklist

## Quick End-to-End Verification

The following commands should all succeed for exchange listing verification:

### 1. DNS Seed Resolution
```bash
nslookup seed1.quantumcoincrypto.com
# Should resolve to IP address

nslookup seed2.quantumcoincrypto.com  
# Should resolve to IP address

nslookup seed3.quantumcoincrypto.com
# Should resolve to IP address
```

### 2. Fresh Node Sync from DNS Seeds
```bash
# Download and verify release
wget https://github.com/aeonith/quantumcoin-ui-/releases/download/v1.0.1-mainnet/quantumcoin-node-linux-x64.tar.gz
wget https://github.com/aeonith/quantumcoin-ui-/releases/download/v1.0.1-mainnet/SHA256SUMS
sha256sum -c SHA256SUMS

# Start fresh node (will discover peers via DNS)
tar -xzf quantumcoin-node-linux-x64.tar.gz
./quantumcoin-node --data-dir=/tmp/fresh-node

# Verify sync progress
curl http://localhost:8332/health
# Should show sync_progress approaching 1.0
```

### 3. Live Explorer Data
```bash
curl https://quantumcoin-ui.vercel.app/api/blocks?limit=5
# Should return real block data, not "Loading" placeholders

curl https://quantumcoin-mainnet-api.vercel.app/explorer/stats
# Should return current height > 0, peer count > 0

curl https://quantumcoin-mainnet-api.vercel.app/status
# Should return {"status":"healthy"}
```

### 4. Artifact Verification
```bash
# Verify checksums
sha256sum -c SHA256SUMS
# Should show: All checksums verified

# Verify GPG signature (when available)
gpg --verify SHA256SUMS.asc SHA256SUMS
# Should show: Good signature

# Verify Docker images (when available)
cosign verify quantumcoin/node:v1.0.1-mainnet
# Should show: Verification successful
```

### 5. Working README Quickstart
```bash
# This should work verbatim on a clean machine:
wget https://github.com/aeonith/quantumcoin-ui-/releases/download/v1.0.1-mainnet/quantumcoin-node-linux-x64.tar.gz
tar -xzf quantumcoin-node-linux-x64.tar.gz
./quantumcoin-node --addnode=seed1.quantumcoincrypto.com

# Should sync and connect to mainnet
```

## Current Status Verification

### ✅ Completed Fixes:
- **Explorer backend**: Wired to `https://quantumcoin-mainnet-api.vercel.app`
- **CI cleanup**: Removed fake-green workflows, kept only `ci` + `perf`
- **Documentation**: README updated to reflect live mainnet status
- **Performance gates**: Added proper latency budgets and error tolerance
- **Smoke testing**: Explorer endpoints verified in CI

### 🔄 In Progress:
- **Release artifacts**: Build script ready, needs execution
- **DNS seeds**: Need to be configured and made live
- **Backend deployment**: API endpoints need to serve real data

### 📋 Exchange Reviewer Checklist:

**Technical Integration:**
- [ ] Download release artifacts and verify checksums
- [ ] Test RPC endpoints from exchange-pack/RPC.md
- [ ] Verify post-quantum signature compatibility
- [ ] Test deposit/withdrawal flows with indexer

**Network Verification:**
- [ ] Confirm DNS seeds resolve and provide working peers
- [ ] Verify blockchain explorer shows real, updating data
- [ ] Test node synchronization from genesis
- [ ] Validate confirmations policy with reorg statistics

**Operational Readiness:**
- [ ] Review ops.md monitoring procedures
- [ ] Test health endpoints and alerting
- [ ] Verify backup/restore procedures
- [ ] Validate security disclosure process

**Compliance & Documentation:**
- [ ] Review SECURITY.md vulnerability disclosure
- [ ] Verify reproducible build instructions
- [ ] Check license compatibility
- [ ] Validate exchange integration documentation

## Contact for Exchange Integration

- **Exchange Partnerships**: partnerships@quantumcoincrypto.com
- **Technical Integration**: tech-support@quantumcoincrypto.com
- **Security Questions**: security@quantumcoincrypto.com

---
**Last Updated**: 2025-01-15  
**Verification Version**: 1.0.1-mainnet
