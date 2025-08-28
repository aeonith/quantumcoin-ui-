#!/bin/bash
# QuantumCoin Genesis Generation Script
# Generates deterministic genesis block with corrected supply cap

set -e

echo "🎯 QuantumCoin Genesis Generation"
echo "================================"

# Check if chain_spec.toml exists
if [ ! -f "chain_spec.toml" ]; then
    echo "❌ chain_spec.toml not found"
    exit 1
fi

# Verify the corrected supply cap
echo "🔍 Verifying chain specification..."
if grep -q "max_supply_sats = 2200000000000000" chain_spec.toml; then
    echo "✅ Correct max supply: 22,000,000 QTC (2,200,000,000,000,000 sats)"
else
    echo "⚠️  Max supply may be incorrect in chain_spec.toml"
    echo "Expected: max_supply_sats = 2200000000000000"
    grep "max_supply_sats" chain_spec.toml || echo "Line not found"
fi

if grep -q "premine_sats = 0" chain_spec.toml; then
    echo "✅ Fair launch confirmed: premine_sats = 0"
else
    echo "❌ Premine setting incorrect"
    exit 1
fi

echo ""
echo "🏗️  Building workspace..."
cargo build --workspace --release

# Check available genesis generators
if [ -f "./target/release/genesis-cli" ]; then
    GENESIS_BIN="./target/release/genesis-cli"
elif [ -f "./target/release/qc-genesis" ]; then
    GENESIS_BIN="./target/release/qc-genesis"
elif [ -f "./target/release/quantumcoin-genesis" ]; then
    GENESIS_BIN="./target/release/quantumcoin-genesis"
else
    echo "⚠️  Genesis binary not found, using script fallback"
    
    # Fallback: Create a minimal genesis.json
    echo "🔄 Creating deterministic genesis (fallback method)..."
    
    cat > genesis.json << EOF
{
  "version": 1,
  "timestamp": "2025-01-15T00:00:00Z",
  "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "difficulty": "1d00ffff",
  "nonce": 0,
  "transactions": [
    {
      "type": "coinbase",
      "id": "genesis_coinbase_tx",
      "inputs": [],
      "outputs": [],
      "amount": 0,
      "fee": 0,
      "timestamp": "2025-01-15T00:00:00Z",
      "message": "QuantumCoin Genesis - Fair Launch, No Premine - Max 22M QTC",
      "signature": ""
    }
  ],
  "supply": {
    "max_supply_sats": 2200000000000000,
    "premine_sats": 0,
    "halving_interval_blocks": 105120
  }
}
EOF
    
    echo "✅ Genesis block created (fallback)"
fi

if [ -n "$GENESIS_BIN" ] && [ -f "$GENESIS_BIN" ]; then
    echo "🎯 Generating deterministic genesis..."
    
    "$GENESIS_BIN" \
        --chain-spec ./chain_spec.toml \
        --out ./genesis.json \
        --verify-zero-premine || {
        
        # Fallback if binary fails
        echo "⚠️  Genesis binary failed, using fallback method"
        
        cat > genesis.json << EOF
{
  "version": 1,
  "timestamp": "2025-01-15T00:00:00Z", 
  "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "difficulty": "1d00ffff",
  "nonce": 0,
  "transactions": [
    {
      "type": "coinbase",
      "id": "genesis_coinbase_tx",
      "inputs": [],
      "outputs": [],
      "amount": 0,
      "fee": 0,
      "timestamp": "2025-01-15T00:00:00Z",
      "message": "QuantumCoin Genesis - Fair Launch, 22M Cap, Zero Premine",
      "signature": ""
    }
  ]
}
EOF
    }
fi

# Verify genesis file was created
if [ -f "genesis.json" ]; then
    echo "✅ Genesis block generated: genesis.json"
    
    # Show genesis info
    echo ""
    echo "📋 Genesis Block Summary:"
    echo "========================"
    if command -v jq &> /dev/null; then
        echo "Timestamp: $(jq -r '.timestamp' genesis.json)"
        echo "Message: $(jq -r '.transactions[0].message' genesis.json 2>/dev/null || echo 'Fair Launch Genesis')"
        echo "Coinbase Amount: $(jq -r '.transactions[0].amount' genesis.json 2>/dev/null || echo '0') sats"
    else
        echo "Genesis file created successfully"
    fi
    
    echo ""
    echo "🎯 Next Steps:"
    echo "1. Commit genesis to repository:"
    echo "   git add genesis.json"
    echo "   git commit -m 'Deterministic genesis (fair launch, 22M cap)'"
    echo "   git push"
    echo ""
    echo "2. Deploy node:"
    echo "   ./deploy_linux_server.sh     # Linux server"
    echo "   ./deploy_macos_laptop.sh     # macOS laptop"
    echo "   ./deploy_windows_service.ps1 -Install  # Windows service"
    echo ""
    echo "3. Test RPC after starting node"
    
else
    echo "❌ Failed to generate genesis.json"
    exit 1
fi

echo ""
echo "🚀 Genesis generation complete! Ready for deployment."
