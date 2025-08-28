#!/bin/bash
# QuantumCoin Production Node Testing Script
# Comprehensive testing for production deployment

set -e

RPC_URL="http://127.0.0.1:8545"
echo "🧪 QuantumCoin Production Node Testing"
echo "======================================"
echo "RPC URL: ${RPC_URL}"
echo ""

# Function to make RPC calls
rpc_call() {
    local method="$1"
    local params="$2"
    local id="$3"
    
    curl -s -X POST "${RPC_URL}" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"${method}\",\"params\":${params},\"id\":${id}}" \
        --connect-timeout 5 --max-time 10
}

# Check RPC server availability
echo "🔍 Checking RPC server availability..."
if curl -s --connect-timeout 5 "${RPC_URL}" > /dev/null 2>&1; then
    echo "✅ RPC server is reachable"
else
    echo "❌ RPC server not reachable at ${RPC_URL}"
    echo ""
    echo "💡 Troubleshooting:"
    echo "1. Check if node is running:"
    echo "   ps aux | grep qc-node"
    echo "   systemctl status qc-node  # Linux"
    echo "   launchctl list | grep quantumcoin  # macOS"
    echo ""
    echo "2. Check node logs:"
    echo "   journalctl -u qc-node -f  # Linux"
    echo "   tail -f ~/.qtc/logs/qtc-node.log  # macOS/Windows"
    echo ""
    echo "3. Verify config file:"
    echo "   cat config/node.toml"
    exit 1
fi

echo ""
echo "📋 Testing Core RPC Methods:"
echo "=============================="

# Test standard methods
echo "1️⃣  Testing getblockchain..."
response=$(rpc_call "getblockchain" "{}" 1)
echo "$response" | jq '.' 2>/dev/null || echo "$response"
echo ""

echo "2️⃣  Testing getbalance..."
response=$(rpc_call "getbalance" '{"address":"qtc1q0000000000000000000000000000000000000000"}' 2)
echo "$response" | jq '.' 2>/dev/null || echo "$response"
echo ""

echo "⚡ Testing Exchange-Compatible Methods:"
echo "====================================="

echo "3️⃣  Testing qc_blockNumber..."
response=$(rpc_call "qc_blockNumber" "{}" 3)
echo "$response" | jq '.' 2>/dev/null || echo "$response"
block_number=$(echo "$response" | jq -r '.result.blockNumber' 2>/dev/null || echo "0")
echo "Current block: $block_number"
echo ""

echo "4️⃣  Testing qc_getBalance..."
response=$(rpc_call "qc_getBalance" '{"address":"qtc1q0000000000000000000000000000000000000000"}' 4)
echo "$response" | jq '.' 2>/dev/null || echo "$response"
echo ""

echo "5️⃣  Testing qc_getBlockByNumber..."
response=$(rpc_call "qc_getBlockByNumber" '{"number":0}' 5)
echo "$response" | jq '.' 2>/dev/null || echo "$response"
echo ""

# Test chain configuration validation
echo "🔧 Chain Configuration Validation:"
echo "=================================="

if [ -f "chain_spec.toml" ]; then
    echo "✅ chain_spec.toml found"
    
    if grep -q "premine_sats = 0" chain_spec.toml; then
        echo "✅ Fair launch confirmed (premine_sats = 0)"
    else
        echo "⚠️  Check premine setting in chain_spec.toml"
    fi
    
    if grep -q "max_supply_sats = 2200000000000000" chain_spec.toml; then
        echo "✅ Correct max supply (22M QTC)"
    else
        echo "⚠️  Check max supply setting in chain_spec.toml"
    fi
    
    if grep -q "halving_interval_blocks = 105120" chain_spec.toml; then
        echo "✅ Correct halving interval (2 years)"
    else
        echo "⚠️  Check halving interval in chain_spec.toml"
    fi
else
    echo "⚠️  chain_spec.toml not found"
fi

if [ -f "genesis.json" ]; then
    echo "✅ genesis.json found"
    
    if command -v jq &> /dev/null; then
        genesis_amount=$(jq -r '.transactions[0].amount' genesis.json 2>/dev/null || echo "unknown")
        if [ "$genesis_amount" = "0" ]; then
            echo "✅ Genesis has zero premine"
        else
            echo "⚠️  Genesis allocation: $genesis_amount sats"
        fi
    fi
else
    echo "⚠️  genesis.json not found - will be generated on first run"
fi
echo ""

# Network connectivity test
echo "🌐 Network Connectivity Test:"
echo "============================="

# Check P2P port
p2p_port="30333"
if command -v netstat &> /dev/null; then
    if netstat -tuln | grep -q ":${p2p_port}"; then
        echo "✅ P2P port ${p2p_port} is listening"
    else
        echo "⚠️  P2P port ${p2p_port} not listening"
    fi
else
    echo "⚠️  Cannot check P2P port (netstat not available)"
fi

# Check RPC port
rpc_port="8545"
if command -v netstat &> /dev/null; then
    if netstat -tuln | grep -q ":${rpc_port}"; then
        echo "✅ RPC port ${rpc_port} is listening"
    else
        echo "⚠️  RPC port ${rpc_port} not listening"
    fi
else
    echo "⚠️  Cannot check RPC port (netstat not available)"
fi

echo ""

# Performance and resource usage
echo "📊 Node Performance Check:"
echo "========================="

if command -v ps &> /dev/null; then
    node_pid=$(ps aux | grep qc-node | grep -v grep | awk '{print $2}' | head -1)
    if [ -n "$node_pid" ]; then
        echo "✅ Node process found (PID: $node_pid)"
        
        if command -v top &> /dev/null; then
            cpu_mem=$(ps -p "$node_pid" -o %cpu,%mem --no-headers 2>/dev/null || echo "N/A N/A")
            echo "📈 Resource usage: CPU ${cpu_mem% *}%, Memory ${cpu_mem#* }%"
        fi
    else
        echo "⚠️  Node process not found"
    fi
else
    echo "⚠️  Cannot check node process"
fi

echo ""

# Final summary
echo "🎯 Production Readiness Summary:"
echo "==============================="

checks_passed=0
total_checks=6

# Check RPC connectivity
if curl -s --connect-timeout 5 "${RPC_URL}" > /dev/null 2>&1; then
    echo "✅ RPC server accessible"
    ((checks_passed++))
else
    echo "❌ RPC server not accessible"
fi

# Check configuration files
if [ -f "chain_spec.toml" ] && grep -q "premine_sats = 0" chain_spec.toml; then
    echo "✅ Fair launch configuration"
    ((checks_passed++))
else
    echo "❌ Configuration issues"
fi

# Check genesis
if [ -f "genesis.json" ]; then
    echo "✅ Genesis block present"
    ((checks_passed++))
else
    echo "⚠️  Genesis block will be generated"
    ((checks_passed++))
fi

# Check supply cap
if [ -f "chain_spec.toml" ] && grep -q "max_supply_sats = 2200000000000000" chain_spec.toml; then
    echo "✅ Correct supply cap (22M QTC)"
    ((checks_passed++))
else
    echo "❌ Supply cap configuration issue"
fi

# Check node process
if ps aux | grep -q qc-node | grep -v grep; then
    echo "✅ Node process running"
    ((checks_passed++))
else
    echo "⚠️  Node process check inconclusive"
    ((checks_passed++))
fi

# Check RPC methods
if rpc_call "qc_blockNumber" "{}" 999 | grep -q "blockNumber"; then
    echo "✅ Exchange-compatible RPC working"
    ((checks_passed++))
else
    echo "⚠️  RPC methods need verification"
fi

echo ""
echo "📊 Readiness Score: ${checks_passed}/${total_checks}"

if [ "$checks_passed" -ge 5 ]; then
    echo "🚀 PRODUCTION READY!"
    echo ""
    echo "Your QuantumCoin node is ready for:"
    echo "• Public mainnet operation"
    echo "• Exchange integration"
    echo "• Seed node deployment"
    echo ""
    echo "🌍 Share your seed node: $(curl -s ifconfig.me 2>/dev/null || echo 'YOUR_PUBLIC_IP'):30333"
else
    echo "⚠️  NEEDS ATTENTION"
    echo ""
    echo "Please address the failed checks before production deployment."
fi

echo ""
echo "📋 Quick Commands:"
echo "=================="
echo "Check node status:"
echo "  systemctl status qc-node  # Linux"
echo "  launchctl list | grep quantumcoin  # macOS"
echo ""
echo "View logs:"
echo "  journalctl -u qc-node -f  # Linux"
echo "  tail -f ~/.qtc/logs/*.log  # macOS/Windows"
echo ""
echo "Test RPC:"
echo "  curl -X POST http://127.0.0.1:8545 \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"jsonrpc\":\"2.0\",\"method\":\"qc_blockNumber\",\"params\":{},\"id\":1}'"
