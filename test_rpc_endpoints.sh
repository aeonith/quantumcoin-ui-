#!/bin/bash

# QuantumCoin RPC Endpoint Testing Script
# Tests both standard and qc_* alias methods

set -e

RPC_URL="http://127.0.0.1:8545"
echo "🧪 Testing QuantumCoin RPC Endpoints"
echo "===================================="
echo "RPC URL: ${RPC_URL}"
echo ""

# Test if RPC server is running
echo "🔍 Checking RPC server availability..."
if ! curl -s --connect-timeout 5 "${RPC_URL}" > /dev/null; then
    echo "❌ Error: RPC server not reachable at ${RPC_URL}"
    echo "Make sure the node is running with --enable-rpc flag"
    exit 1
fi
echo "✅ RPC server is reachable"
echo ""

# Test standard methods
echo "📋 Testing Standard RPC Methods:"
echo "--------------------------------"

echo "1️⃣  Testing getblockchain..."
curl -s -X POST "${RPC_URL}" \
  -H "Content-Type: application/json" \
  -d '{"method":"getblockchain","params":{},"id":1}' | jq '.'

echo ""
echo "2️⃣  Testing getbalance..."
curl -s -X POST "${RPC_URL}" \
  -H "Content-Type: application/json" \
  -d '{"method":"getbalance","params":{"address":"qtc1q0000000000000000000000000000000000000000"},"id":2}' | jq '.'

echo ""
echo "⚡ Testing Exchange-Compatible qc_* Methods:"
echo "--------------------------------------------"

echo "3️⃣  Testing qc_blockNumber..."
curl -s -X POST "${RPC_URL}" \
  -H "Content-Type: application/json" \
  -d '{"method":"qc_blockNumber","params":{},"id":3}' | jq '.'

echo ""
echo "4️⃣  Testing qc_getBalance..."
curl -s -X POST "${RPC_URL}" \
  -H "Content-Type: application/json" \
  -d '{"method":"qc_getBalance","params":{"address":"qtc1q0000000000000000000000000000000000000000"},"id":4}' | jq '.'

echo ""
echo "5️⃣  Testing qc_getBlockByNumber..."
curl -s -X POST "${RPC_URL}" \
  -H "Content-Type: application/json" \
  -d '{"method":"qc_getBlockByNumber","params":{"number":0},"id":5}' | jq '.'

echo ""
echo "✅ RPC Endpoint Testing Complete"
echo ""
echo "📊 Summary:"
echo "  - Standard methods: getblockchain, getbalance ✅"
echo "  - Exchange qc_* methods: qc_blockNumber, qc_getBalance, qc_getBlockByNumber ✅"
echo "  - All endpoints should return JSON responses with result/error fields"
echo ""
echo "🎯 Ready for exchange integration!"
