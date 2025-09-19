# QuantumCoin Exchange Integration Guide

**Version**: 1.0.0  
**Date**: January 2025  
**Status**: Exchange-Ready  

## Overview

This guide provides everything needed to integrate QuantumCoin (QTC) into cryptocurrency exchanges. QuantumCoin is a Bitcoin-like cryptocurrency with post-quantum signature support and institutional-grade security features.

## Quick Start

```bash
# 1. Download and run node
docker run -d --name qtc-node \
  -p 8545:8545 -p 8546:8546 \
  -v qtc-data:/data/quantumcoin \
  ghcr.io/quantumcoin-crypto/quantumcoin-node:latest

# 2. Wait for sync and check status
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getblockchaininfo","id":1}'

# 3. Create exchange wallets
docker exec qtc-node qtc-wallet new --name exchange-hot --type hot
docker exec qtc-node qtc-wallet new --name exchange-cold --type cold
```

## Key Information

| Parameter | Value |
|-----------|-------|
| **Ticker Symbol** | QTC |
| **Full Name** | QuantumCoin |
| **Max Supply** | 22,000,000 QTC |
| **Block Time** | 10 minutes (600 seconds) |
| **Confirmations** | 6 recommended (1 hour) |
| **Precision** | 8 decimal places (like Bitcoin) |
| **Address Format** | Bech32 with `qtc1` prefix |
| **Signature Algorithm** | Dilithium2 (post-quantum) |
| **Consensus** | Proof of Work (SHA256d) |

## Network Endpoints

### Mainnet
- **RPC Port**: 8545
- **P2P Port**: 8546  
- **Genesis Hash**: `00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048`
- **Magic Bytes**: `0xf9beb4d9`

### Testnet  
- **RPC Port**: 18545
- **P2P Port**: 18546
- **Genesis Hash**: `000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943`
- **Magic Bytes**: `0xfabfb5da`

## Address Format

QuantumCoin uses Bech32 encoding with post-quantum public keys:

```
qtc1qw508d6qejxtdg4y5r3zarvary0c5xw7k8gkx8k
```

**Address Validation Regex**:
```regex
^qtc1[02-9ac-hj-np-z]{39,59}$
```

**Address Validation Function (JavaScript)**:
```javascript
function isValidQuantumCoinAddress(address) {
    // Check prefix
    if (!address.startsWith('qtc1')) return false;
    
    // Check length (42-62 characters typical)
    if (address.length < 42 || address.length > 62) return false;
    
    // Check Bech32 characters only
    const validChars = /^[02-9ac-hj-np-z]+$/;
    return validChars.test(address.slice(4));
}
```

## RPC Interface

QuantumCoin provides a Bitcoin-compatible JSON-RPC interface:

### Connection
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"METHOD_NAME","params":[],"id":1}'
```

### Essential Methods for Exchanges

#### `getblockchaininfo`
Get current blockchain status:
```json
{
    "jsonrpc": "2.0",
    "method": "getblockchaininfo",
    "id": 1
}
```

**Response**:
```json
{
    "result": {
        "version": "1.0.0",
        "network": "QuantumCoin Mainnet",
        "height": 150000,
        "bestblockhash": "00000000000000000008...",
        "difficulty": 436835588915.0543,
        "supply": {
            "max": 2200000000000000,
            "current": 750000000000000,
            "premine": 0
        },
        "halvingInterval": 105120,
        "nextHalving": 20000
    }
}
```

#### `getaddressbalance`
Check address balance:
```json
{
    "jsonrpc": "2.0",
    "method": "getaddressbalance", 
    "params": {
        "addresses": ["qtc1qw508d6qejxtdg4y5r3zarvary0c5xw7k8gkx8k"],
        "min_confirmations": 6
    },
    "id": 1
}
```

#### `getaddresshistory`
Get transaction history for address:
```json
{
    "jsonrpc": "2.0",
    "method": "getaddresshistory",
    "params": {
        "addresses": ["qtc1qw508d6qejxtdg4y5r3zarvary0c5xw7k8gkx8k"],
        "from_height": 0,
        "to_height": 150000
    },
    "id": 1
}
```

#### `sendrawtransaction`
Broadcast signed transaction:
```json
{
    "jsonrpc": "2.0",
    "method": "sendrawtransaction",
    "params": {
        "hexstring": "01000000..."
    },
    "id": 1
}
```

## Wallet Management

### Hot Wallet Setup
For day-to-day operations:

```bash
# Create hot wallet
qtc-wallet new --name exchange-hot --type hot

# Generate deposit addresses
qtc-wallet address --wallet exchange-hot --label "user-12345"

# Check balance
qtc-wallet balance --wallet exchange-hot --confirmations 6

# Send withdrawal
qtc-wallet send --wallet exchange-hot \
  --to qtc1abc... --amount 100.50 \
  --memo "withdrawal-67890"
```

### Cold Wallet Setup
For secure storage of reserves:

```bash
# Create cold wallet (air-gapped machine)
qtc-wallet new --name exchange-cold --type cold

# Generate unsigned transaction (online machine)
qtc-wallet send --wallet exchange-cold \
  --to qtc1abc... --amount 1000.0 --offline

# Sign transaction (air-gapped machine) 
qtc-wallet sign --wallet exchange-cold \
  --transaction unsigned_tx_123.json

# Broadcast signed transaction (online machine)
qtc-wallet broadcast --file signed_tx_123.json
```

### Batch Operations
For processing multiple withdrawals:

```bash
# Create CSV file: address,amount,memo
echo "qtc1abc...,100.0,withdrawal-1" > withdrawals.csv
echo "qtc1def...,250.5,withdrawal-2" >> withdrawals.csv

# Process batch (with dry run first)
qtc-wallet batch-send --file withdrawals.csv \
  --wallet exchange-hot --dry-run

# Execute batch
qtc-wallet batch-send --file withdrawals.csv \
  --wallet exchange-hot
```

## Security Considerations

### Post-Quantum Cryptography
- QuantumCoin uses **Dilithium2** signatures (NIST PQC standard)
- All addresses and transactions are quantum-resistant
- No additional security measures needed vs Bitcoin

### RevStop Feature (DISABLED)
- QuantumCoin includes an optional transaction reversal mechanism
- **This feature is DISABLED by default and cannot be activated without consensus**
- Exchanges have full custody and control of funds (same as Bitcoin)
- No admin keys or centralized control

### Recommended Security Practices
1. **Cold Storage**: Keep majority of funds in offline cold wallets
2. **Multi-Sig**: Use multi-signature wallets for large amounts
3. **Confirmations**: Wait 6 confirmations (1 hour) for deposits
4. **Monitoring**: Monitor node health and network status
5. **Backup**: Regular encrypted backups of wallet files

## Transaction Monitoring

### Deposit Detection
Monitor specific addresses for incoming payments:

```python
import requests
import time

def check_deposits(addresses, min_confirmations=6):
    rpc_url = "http://localhost:8545"
    
    for address in addresses:
        payload = {
            "jsonrpc": "2.0",
            "method": "getaddresshistory", 
            "params": {
                "addresses": [address],
                "min_confirmations": min_confirmations
            },
            "id": 1
        }
        
        response = requests.post(rpc_url, json=payload)
        history = response.json()["result"]
        
        # Process new transactions
        for tx in history.get("transactions", []):
            if tx["category"] == "receive":
                print(f"New deposit: {tx['amount']} QTC to {address}")
                # Credit user account...
```

### Withdrawal Processing
```python
def process_withdrawal(user_id, address, amount):
    # Validate address format
    if not is_valid_address(address):
        raise ValueError("Invalid address")
    
    # Create transaction
    result = wallet_send(
        wallet="exchange-hot",
        to_address=address,
        amount=amount,
        memo=f"withdrawal-{user_id}"
    )
    
    # Store transaction ID for tracking
    store_withdrawal_txid(user_id, result["txid"])
    
    return result["txid"]
```

## Node Management

### System Requirements
- **CPU**: 4+ cores recommended
- **RAM**: 8GB minimum, 16GB recommended  
- **Storage**: 100GB+ SSD (blockchain grows ~50GB/year)
- **Network**: Reliable internet, 100Mbps+ recommended
- **OS**: Linux (Ubuntu 20.04+ recommended)

### Docker Deployment
```yaml
version: '3.8'
services:
  qtc-node:
    image: ghcr.io/quantumcoin-crypto/quantumcoin-node:latest
    ports:
      - "8545:8545"  # RPC
      - "8546:8546"  # P2P
    volumes:
      - qtc-data:/data/quantumcoin
      - ./quantumcoin.conf:/etc/quantumcoin/quantumcoin.conf:ro
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "quantumcoin-node", "status"]
      interval: 30s
      timeout: 10s
      retries: 3
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "3"

volumes:
  qtc-data:
```

### Monitoring & Alerts
Monitor these metrics:
- **Block Height**: Should increase every ~10 minutes
- **Peer Count**: Should be 8-125 peers
- **Memory Usage**: Should be stable < 4GB
- **Disk Space**: Monitor blockchain growth
- **Network Connectivity**: P2P port should be reachable

Example Prometheus metrics endpoint: `http://localhost:9090/metrics`

## Testing

### Testnet Integration
Use testnet for development and testing:

```bash
# Start testnet node
docker run -d --name qtc-testnet \
  -p 18545:18545 -p 18546:18546 \
  ghcr.io/quantumcoin-crypto/quantumcoin-node:latest \
  --network testnet

# Get testnet coins from faucet
curl -X POST https://faucet.quantumcoincrypto.com/drip \
  -d '{"address":"qtc1qtest..."}'
```

### Integration Test Checklist
- [ ] Node synchronization
- [ ] Address generation and validation
- [ ] Deposit detection (1, 3, 6 confirmations)
- [ ] Balance checking
- [ ] Transaction creation and signing
- [ ] Transaction broadcasting
- [ ] Withdrawal processing
- [ ] Fee estimation
- [ ] Error handling
- [ ] Backup and recovery

## Support & Resources

### Official Resources
- **Website**: https://quantumcoincrypto.com
- **Documentation**: https://docs.quantumcoincrypto.com  
- **GitHub**: https://github.com/quantumcoin-crypto/quantumcoin-core
- **Docker Images**: https://github.com/orgs/quantumcoin-crypto/packages

### Exchange Support
- **Technical Support**: exchanges@quantumcoincrypto.com
- **Response Time**: 24-48 hours
- **Security Issues**: security@quantumcoincrypto.com  
- **Critical Support**: +1 (555) 123-4567 (24/7)

### Integration Assistance
Free integration support includes:
- Technical consultation calls
- Custom documentation
- Code review and testing
- Sandbox/testnet setup
- Launch coordination

Contact: integrations@quantumcoincrypto.com

## Legal & Compliance

### Regulatory Status
- **Classification**: Utility token, decentralized cryptocurrency
- **Compliance**: No special regulatory requirements beyond standard crypto
- **Admin Keys**: None (fully decentralized)
- **Premine**: Zero (fair launch)
- **KYC/AML**: Exchange's responsibility (standard crypto practices)

### Legal Resources
- **Terms of Service**: Available on website
- **Privacy Policy**: Available on website
- **Legal Memorandum**: Available to exchanges upon request
- **Compliance Package**: Available to institutional integrators

---

*This integration guide is maintained by the QuantumCoin development team. For the latest version, visit: https://docs.quantumcoincrypto.com/exchange-integration*

**Last Updated**: January 19, 2025  
**Document Version**: 1.0.0
