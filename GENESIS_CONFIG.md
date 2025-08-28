# QuantumCoin Genesis Configuration

## Deterministic Genesis Block Parameters

**Based on `chain_spec.toml` settings:**

```toml
[network]
name = "QuantumCoin"
symbol = "QC"
decimals = 8

[supply]
max_supply_sats = 22000000_00000000   # 22M QTC
halving_interval_blocks = 105120      # ~2 years
premine_sats = 0                      # FAIR LAUNCH

[consensus]
target_block_time_secs = 600          # 10 minutes
difficulty_adjustment = "ASERT" 
```

## Genesis Block Specification

```json
{
  "version": 1,
  "timestamp": "2025-01-15T00:00:00Z",
  "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "merkle_root": "calculated_from_coinbase_tx",
  "difficulty": "0x1d00ffff",
  "nonce": "to_be_mined",
  "transactions": [
    {
      "type": "coinbase",
      "inputs": [],
      "outputs": [],
      "amount": 0,
      "message": "QuantumCoin Genesis - Fair Launch, No Premine"
    }
  ]
}
```

## Critical Verification Points

✅ **No Premine**: Genesis coinbase allocates **0 QTC**
✅ **Fair Launch**: All 22M QTC must be earned through mining  
✅ **Deterministic**: Same genesis across all nodes
✅ **Post-Quantum**: Uses Dilithium2 signatures

## Genesis Generation Command

```bash
# Once Rust is available
cargo run --bin genesis-cli -- \
  --config ./chain_spec.toml \
  --output ./genesis.json \
  --verify-zero-premine
```

## Genesis Hash (Expected)

The genesis block hash will be deterministic based on:
- Timestamp: 2025-01-15T00:00:00Z
- Coinbase message: "QuantumCoin Genesis - Fair Launch, No Premine"
- Zero allocation transactions
- Fixed difficulty and nonce

**Status: Ready for regeneration once build environment is available**
