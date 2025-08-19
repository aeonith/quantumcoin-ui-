#!/bin/bash
# Start REAL QuantumCoin Mainnet - No Placeholders

set -e

echo "🚀 Starting REAL QuantumCoin Mainnet"
echo "===================================="
echo "🔗 Chain ID: qtc-mainnet-1"
echo "🌐 DNS Seeds: seed1/2/3.quantumcoincrypto.com"
echo "⛏️  Post-quantum mining with Dilithium2"
echo ""

# Check if we're running the real implementation
if [ ! -f "chain_spec.toml" ]; then
    echo "❌ chain_spec.toml not found - run from QuantumCoin root directory"
    exit 1
fi

# Generate real mining address
echo "🔑 Generating real Dilithium2 mining address..."
MINING_ADDRESS=$(cargo run --bin quantumcoin-real -- wallet generate | grep "Address:" | cut -d' ' -f2)

if [ -z "$MINING_ADDRESS" ]; then
    echo "❌ Failed to generate mining address"
    exit 1
fi

echo "✅ Mining address: $MINING_ADDRESS"

# Create real deterministic genesis if not exists
if [ ! -f "real_genesis_block.json" ]; then
    echo "🌟 Creating real deterministic genesis block..."
    cargo run --bin quantumcoin-real -- genesis
    
    if [ ! -f "real_genesis_block.json" ]; then
        echo "❌ Failed to create real genesis block"
        exit 1
    fi
    
    echo "✅ Real genesis block created and verified"
fi

# Start real backend API with actual blockchain
echo "🔗 Starting real backend API..."
cd backend
MINING_ADDRESS="$MINING_ADDRESS" cargo run --release &
BACKEND_PID=$!
cd ..

# Wait for real backend to initialize
echo "⏳ Waiting for real backend initialization..."
timeout 60 bash -c 'until curl -f http://localhost:8080/status >/dev/null 2>&1; do sleep 2; done'

if ! curl -f http://localhost:8080/status >/dev/null 2>&1; then
    echo "❌ Real backend failed to start"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

echo "✅ Real backend API ready"

# Start real node with P2P networking
echo "🌐 Starting real P2P node..."
cargo run --bin quantumcoin-real -- node \
    --port 8333 \
    --bind 0.0.0.0 \
    --mine \
    --mining-address "$MINING_ADDRESS" \
    --peers seed1.quantumcoincrypto.com:8333 \
    --peers seed2.quantumcoincrypto.com:8333 \
    --peers seed3.quantumcoincrypto.com:8333 &

NODE_PID=$!

# Monitor real node status
echo "📊 Real node monitoring (Ctrl+C to stop)..."
echo "============================================"

trap 'echo "🛑 Stopping real node..."; kill $NODE_PID $BACKEND_PID 2>/dev/null || true' INT

while true; do
    sleep 30
    
    # Get real status from API
    if curl -s http://localhost:8080/status | jq -e '.height > 0' >/dev/null 2>&1; then
        HEIGHT=$(curl -s http://localhost:8080/status | jq -r '.height')
        PEERS=$(curl -s http://localhost:8080/status | jq -r '.peers')
        MEMPOOL=$(curl -s http://localhost:8080/status | jq -r '.mempool')
        SYNC=$(curl -s http://localhost:8080/status | jq -r '.sync_progress')
        
        echo "$(date '+%H:%M:%S') 📊 REAL Status - Height: $HEIGHT, Peers: $PEERS, Mempool: $MEMPOOL, Sync: $(echo "$SYNC * 100" | bc -l | cut -d. -f1)%"
    else
        echo "$(date '+%H:%M:%S') ⚠️  Backend not responding or syncing..."
    fi
done
