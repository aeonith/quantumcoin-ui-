#!/bin/bash

# QuantumCoin Seed Node Startup Script
# Starts a seed node for the QuantumCoin network

set -e

echo "🚀 Starting QuantumCoin Seed Node"
echo "=================================="

# Configuration
DATA_DIR="${HOME}/.qtc/seed"
CHAIN_SPEC="./chain_spec.toml"
P2P_PORT="30333"
RPC_PORT="8545"
P2P_LISTEN="0.0.0.0:${P2P_PORT}"
RPC_LISTEN="127.0.0.1:${RPC_PORT}"

# Create data directory if it doesn't exist
mkdir -p "${DATA_DIR}"

echo "📁 Data Directory: ${DATA_DIR}"
echo "🌐 P2P Listen: ${P2P_LISTEN}"
echo "🔧 RPC Listen: ${RPC_LISTEN}"
echo "📋 Chain Spec: ${CHAIN_SPEC}"
echo ""

# Check if binary exists
if [ ! -f "./target/release/qc-node" ]; then
    echo "❌ Error: qc-node binary not found at ./target/release/qc-node"
    echo "Run: cargo build --workspace --release"
    exit 1
fi

# Check if chain spec exists
if [ ! -f "${CHAIN_SPEC}" ]; then
    echo "❌ Error: Chain spec not found at ${CHAIN_SPEC}"
    exit 1
fi

echo "✅ Pre-flight checks passed"
echo ""
echo "🎯 Starting seed node..."
echo "   - P2P network on port ${P2P_PORT}"
echo "   - RPC interface on port ${RPC_PORT}"
echo "   - Data stored in ${DATA_DIR}"
echo ""
echo "Press Ctrl+C to stop the node"
echo ""

# Start the seed node
exec ./target/release/qc-node \
  --data-dir "${DATA_DIR}" \
  --chain-spec "${CHAIN_SPEC}" \
  --p2p-listen "${P2P_LISTEN}" \
  --rpc-listen "${RPC_LISTEN}" \
  --log-level info \
  --enable-rpc \
  --seed-node \
  "$@"
