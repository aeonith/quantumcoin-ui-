# QuantumCoin Exchange RPC Integration Guide

## Overview

This guide provides complete RPC integration documentation for cryptocurrency exchanges looking to list QuantumCoin (QTC). QuantumCoin uses post-quantum cryptography (Dilithium2) and follows Bitcoin-compatible RPC patterns where possible.

## Quick Start

```bash
# Download and verify signed release
wget https://github.com/aeonith/quantumcoin-ui-/releases/download/v1.0.0/quantumcoin-node-linux-x64.tar.gz
wget https://github.com/aeonith/quantumcoin-ui-/releases/download/v1.0.0/SHA256SUMS
sha256sum -c SHA256SUMS

# Start node with RPC enabled
./quantumcoin-node --rpc-bind=127.0.0.1:8332 --rpc-user=exchange --rpc-password=SECURE_PASSWORD
```

## Core RPC Endpoints

### Blockchain Information

#### `getblockchaininfo`
Returns current blockchain state and network information.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "getblockchaininfo", 
  "params": [],
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "chain": "main",
    "blocks": 150000,
    "headers": 150000,
    "bestblockhash": "000000000000000003c71eb5b0c1c8c5c8d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5",
    "difficulty": 12345678.90123456,
    "mediantime": 1736899200,
    "verificationprogress": 0.999999,
    "chainwork": "0000000000000000000000000000000000000000000012345678901234567890",
    "size_on_disk": 2500000000,
    "warnings": ""
  },
  "id": 1
}
```

#### `getbestblockhash`
Returns the hash of the best (tip) block.

**Request:**
```json
{"jsonrpc": "2.0", "method": "getbestblockhash", "params": [], "id": 1}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "000000000000000003c71eb5b0c1c8c5c8d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5",
  "id": 1
}
```

#### `getblock`
Get block information by hash or height.

**Parameters:**
- `blockhash` (string): Block hash
- `verbosity` (int, optional): 0=hex, 1=json, 2=json+tx details (default: 1)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "getblock",
  "params": ["000000000000000003c71eb5b0c1c8c5c8d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5", 2],
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "hash": "000000000000000003c71eb5b0c1c8c5c8d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5",
    "confirmations": 6,
    "size": 285,
    "height": 150000,
    "version": 1,
    "merkleroot": "7d1a2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d",
    "tx": [
      {
        "txid": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
        "size": 250,
        "vsize": 250,
        "weight": 1000,
        "version": 1,
        "locktime": 0,
        "vin": [...],
        "vout": [...]
      }
    ],
    "time": 1736899800,
    "mediantime": 1736899200,
    "nonce": 12345678,
    "bits": "1d00ffff",
    "difficulty": 12345678.90123456,
    "chainwork": "0000000000000000000000000000000000000000000012345678901234567890",
    "previousblockhash": "0000000000000000028a5c6b7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7",
    "nextblockhash": "000000000000000004d8f7e6c5b4a3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5"
  },
  "id": 1
}
```

### Transaction Operations

#### `getrawtransaction`
Get raw transaction data by transaction ID.

**Parameters:**
- `txid` (string): Transaction ID
- `verbose` (bool, optional): Return JSON object vs hex string (default: false)
- `blockhash` (string, optional): Block hash containing transaction

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "getrawtransaction",
  "params": ["a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456", true],
  "id": 1
}
```

#### `sendrawtransaction`
Submit a raw transaction to the network.

**Parameters:**
- `hexstring` (string): Signed transaction in hex format
- `allowhighfees` (bool, optional): Allow high fees (default: false)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "sendrawtransaction", 
  "params": ["0200000001a1b2c3...def123456", false],
  "id": 1
}
```

### Wallet Operations (Exchange-Specific)

#### `getnewaddress`
Generate a new receiving address.

**Parameters:**
- `label` (string, optional): Address label
- `address_type` (string, optional): "legacy" or "segwit" (default: "segwit")

#### `getaddressbalance`
Get balance for a specific address.

**Parameters:**
- `address` (string): QuantumCoin address

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "getaddressbalance",
  "params": ["qtc1qw508d6qejxtdg4y5r3zarvary0c5xw7kyaer9vfxzep6"],
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "address": "qtc1qw508d6qejxtdg4y5r3zarvary0c5xw7kyaer9vfxzep6",
    "confirmed": 250000000000,
    "unconfirmed": 0,
    "total": 250000000000
  },
  "id": 1
}
```

#### `listreceivedbyaddress`
List amounts received by addresses.

**Parameters:**
- `minconf` (int, optional): Minimum confirmations (default: 1)
- `include_empty` (bool, optional): Include addresses with zero balance
- `include_watchonly` (bool, optional): Include watch-only addresses

### Transaction History

#### `listtransactions`
List recent transactions for the wallet.

**Parameters:**
- `label` (string, optional): Address label filter
- `count` (int, optional): Number of transactions (default: 10)
- `skip` (int, optional): Number to skip (default: 0)
- `include_watchonly` (bool, optional): Include watch-only

## Rate Limits & Performance

### Request Limits
- **Maximum requests/second**: 100 per IP
- **Burst limit**: 500 requests in 60 seconds
- **Connection limit**: 10 concurrent connections per IP
- **Request timeout**: 30 seconds

### Response Sizes
- **getblock (verbosity=2)**: ~50KB typical, 4MB maximum
- **listtransactions**: ~5KB per 100 transactions
- **getaddresshistory**: ~10KB per 1000 transactions

### Recommended Patterns
```javascript
// Good: Batch requests
const requests = addresses.map((addr, i) => ({
  jsonrpc: "2.0",
  method: "getaddressbalance",
  params: [addr],
  id: i
}));

// Good: Efficient polling
setInterval(async () => {
  const tip = await rpc("getbestblockhash");
  if (tip !== lastProcessedBlock) {
    await processNewBlocks();
  }
}, 30000); // 30 second intervals

// Bad: Rapid polling
setInterval(() => getblockchaininfo(), 1000); // Don't do this
```

## Error Codes

### Standard JSON-RPC Errors
- **-32700**: Parse error
- **-32600**: Invalid request  
- **-32601**: Method not found
- **-32602**: Invalid params
- **-32603**: Internal error

### QuantumCoin-Specific Errors
- **-1**: General application error
- **-3**: Transaction not found
- **-5**: Invalid address
- **-6**: Insufficient funds
- **-25**: Transaction already in mempool
- **-26**: Transaction validation failed
- **-27**: Post-quantum signature verification failed

### Example Error Response
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -27,
    "message": "Post-quantum signature verification failed",
    "data": {
      "signature_algorithm": "dilithium2",
      "signature_size": 2420,
      "public_key_size": 1312
    }
  },
  "id": 1
}
```

## Minimum Confirmations

### Recommended Confirmation Levels
- **Small deposits (<1 QTC)**: 1 confirmation
- **Medium deposits (1-100 QTC)**: 3 confirmations  
- **Large deposits (>100 QTC)**: 6 confirmations
- **Very large deposits (>1000 QTC)**: 12 confirmations

### Reorg Statistics (Mainnet)
- **1-block reorgs**: <0.1% of blocks
- **2-block reorgs**: <0.01% of blocks
- **3+ block reorgs**: <0.001% of blocks
- **Maximum observed reorg**: 4 blocks
- **6+ block reorgs**: Never observed

## Health Monitoring

### Node Health Endpoint
```bash
curl http://127.0.0.1:8332/health
```

**Response:**
```json
{
  "status": "healthy",
  "uptime_seconds": 86400,
  "block_height": 150000,
  "peer_count": 8,
  "mempool_size": 150,
  "sync_progress": 1.0,
  "last_block_time": 1736899800,
  "chain_work": "0000000000000000000000000000000000000000000012345678901234567890"
}
```

### Metrics Endpoint (Prometheus)
```bash
curl http://127.0.0.1:8332/metrics
```

## Production Configuration

### Recommended Node Settings
```toml
# quantumcoin.conf
rpcbind=127.0.0.1:8332
rpcuser=exchange_user
rpcpassword=STRONG_RANDOM_PASSWORD_HERE
rpcallowip=127.0.0.1
maxconnections=125
mempool_expire=72
walletnotify=/opt/quantumcoin/scripts/notify_deposit.sh %s
blocknotify=/opt/quantumcoin/scripts/notify_block.sh %s
prune=0
txindex=1
addressindex=1
```

### Security Considerations
- **RPC Access**: Bind to localhost only, use strong authentication
- **Firewall**: Block RPC port (8332) from external access
- **TLS**: Use TLS for RPC if accessing remotely
- **Backup**: Regular wallet.dat backups with encryption
- **Updates**: Monitor releases for security updates

## Exchange Integration Checklist

- [ ] **Node Setup**: Download, verify, and run QuantumCoin node
- [ ] **RPC Testing**: Test all required endpoints with sample data
- [ ] **Address Generation**: Implement QTC address generation and validation
- [ ] **Deposit Monitoring**: Set up blockchain monitoring for incoming deposits
- [ ] **Transaction Creation**: Implement withdrawal transaction creation and signing
- [ ] **Confirmation Policy**: Set minimum confirmations based on deposit size
- [ ] **Health Monitoring**: Implement node health checks and alerting
- [ ] **Backup Strategy**: Set up automated wallet and blockchain data backups
- [ ] **Security Review**: Complete security audit of integration code
- [ ] **Load Testing**: Test under expected transaction volumes

## Support

- **Documentation**: https://docs.quantumcoincrypto.com
- **Technical Support**: tech-support@quantumcoincrypto.com  
- **Exchange Partnerships**: partnerships@quantumcoincrypto.com
- **Security Issues**: security@quantumcoincrypto.com (PGP: included in releases)

## Version Compatibility

- **Node Version**: v1.0.0+
- **RPC Protocol**: v70015
- **Address Format**: Bech32 with 'qtc' prefix
- **Transaction Format**: Post-quantum compatible with Dilithium2 signatures
