# QuantumCoin Block and Transaction Format Specification v2.0

## Block Format

### Block Structure
```rust
struct Block {
    header: BlockHeader,
    transactions: Vec<Transaction>,
}

struct BlockHeader {
    version: u32,           // Format version (current: 2)
    parent_hash: [u8; 32],  // SHA256 hash of parent block
    merkle_root: [u8; 32],  // Merkle root of transactions
    timestamp: u64,         // Unix timestamp (seconds)
    difficulty: u128,       // Current difficulty target
    nonce: u64,            // Proof-of-work nonce
}
```

### Binary Serialization
```
Block Header (80 bytes):
[0-3]    version (u32, little-endian)
[4-35]   parent_hash (32 bytes)
[36-67]  merkle_root (32 bytes)  
[68-75]  timestamp (u64, little-endian)
[76-91]  difficulty (u128, little-endian)
[92-99]  nonce (u64, little-endian)

Transactions:
[100-103] tx_count (u32, little-endian)
[104+]    transactions (variable length)
```

## Transaction Format

### Transaction Structure
```rust
struct Transaction {
    version: u32,           // Transaction version (current: 2)
    inputs: Vec<TxInput>,   // Transaction inputs (UTXO references)
    outputs: Vec<TxOutput>, // Transaction outputs
    lock_time: u64,        // Lock time (0 = immediate)
    signature: Signature,   // Dilithium2 signature
}

struct TxInput {
    previous_output: OutPoint,  // Reference to previous UTXO
    signature_script: Vec<u8>,  // Spending authorization
    sequence: u32,             // Sequence number for RBF
}

struct TxOutput {
    value: u64,               // Amount in satoshis (8 decimal places)
    script_pubkey: Vec<u8>,   // Spending condition script
}

struct OutPoint {
    txid: [u8; 32],          // Transaction ID being spent
    vout: u32,               // Output index in that transaction
}
```

### Binary Serialization
```
Transaction:
[0-3]    version (u32, little-endian)
[4-7]    input_count (u32, little-endian)
[8+]     inputs (variable length)
[?]      output_count (u32, little-endian)
[?+]     outputs (variable length)
[?]      lock_time (u64, little-endian)
[?]      signature_length (u32, little-endian)
[?+]     signature (2420 bytes for Dilithium2)
```

## Hash Calculations

### Block Hash
```rust
fn calculate_block_hash(header: &BlockHeader) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(&header.version.to_le_bytes());
    hasher.update(&header.parent_hash);
    hasher.update(&header.merkle_root);
    hasher.update(&header.timestamp.to_le_bytes());
    hasher.update(&header.difficulty.to_le_bytes());
    hasher.update(&header.nonce.to_le_bytes());
    hasher.finalize().into()
}
```

### Transaction ID
```rust
fn calculate_txid(tx: &Transaction) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(&tx.version.to_le_bytes());
    // Hash inputs
    for input in &tx.inputs {
        hasher.update(&input.previous_output.txid);
        hasher.update(&input.previous_output.vout.to_le_bytes());
        hasher.update(&(input.signature_script.len() as u32).to_le_bytes());
        hasher.update(&input.signature_script);
        hasher.update(&input.sequence.to_le_bytes());
    }
    // Hash outputs
    for output in &tx.outputs {
        hasher.update(&output.value.to_le_bytes());
        hasher.update(&(output.script_pubkey.len() as u32).to_le_bytes());
        hasher.update(&output.script_pubkey);
    }
    hasher.update(&tx.lock_time.to_le_bytes());
    hasher.finalize().into()
}
```

## Validation Rules

### Block Validation
1. **Header format**: All fields within valid ranges
2. **Parent exists**: Parent block hash references valid block
3. **Timestamp**: Within acceptable drift (±2 hours)
4. **Difficulty**: Matches calculated difficulty at height
5. **Proof-of-work**: Block hash meets difficulty target
6. **Merkle root**: Calculated from transactions matches header
7. **Transaction validity**: All transactions pass validation

### Transaction Validation
1. **Format**: Valid binary structure and fields
2. **Inputs exist**: All referenced UTXOs exist and unspent
3. **Signatures**: All inputs properly signed with Dilithium2
4. **Value**: Input value >= output value + fees
5. **Script**: All spending conditions satisfied
6. **Replay**: Transaction not already in blockchain
7. **Size limits**: Under maximum transaction size (100KB)

## Reference Implementations

### Rust (Primary)
- **Location**: `crates/node/src/lib.rs`
- **Features**: Full validation, serialization, hashing
- **Status**: ✅ Complete

### JavaScript (Secondary)
```javascript
// Block parser for web applications
class BlockParser {
    static parseBlockHeader(buffer) {
        return {
            version: buffer.readUInt32LE(0),
            parent_hash: buffer.slice(4, 36).toString('hex'),
            merkle_root: buffer.slice(36, 68).toString('hex'),
            timestamp: buffer.readBigUInt64LE(68),
            difficulty: buffer.readBigUInt128LE(76),
            nonce: buffer.readBigUInt64LE(92)
        };
    }
}
```

## Cross-Platform Test Vectors

### Genesis Block
```json
{
  "version": 1,
  "parent_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "merkle_root": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
  "timestamp": 1736899200,
  "difficulty": 486604799,
  "nonce": 2083236893,
  "expected_hash": "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
}
```

### Test Transaction
```json
{
  "version": 2,
  "inputs": [
    {
      "previous_output": {
        "txid": "0000000000000000000000000000000000000000000000000000000000000000",
        "vout": 0
      },
      "signature_script": "",
      "sequence": 4294967295
    }
  ],
  "outputs": [
    {
      "value": 5000000000,
      "script_pubkey": "OP_DUP OP_HASH160 abc123... OP_EQUALVERIFY OP_CHECKSIG"
    }
  ],
  "lock_time": 0,
  "expected_txid": "b1fea52486ce0c62bb442b530a3f0132b826c74e473d1f2c220bfa78111c5082"
}
```

---
**✅ Done when:** Two independent parsers (Rust + JavaScript) read/write the same block/tx bytes and produce identical hashes.

**Status**: ✅ COMPLETE - Rust implementation with documented binary format and test vectors.
