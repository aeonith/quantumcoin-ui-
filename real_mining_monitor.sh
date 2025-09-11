#!/bin/bash
# Real QuantumCoin Mining Monitor

echo "⚛️ QuantumCoin Real Mining Monitor"
echo "=================================="

# Check if processes are running
NODE_PID=$(pgrep -f "real_quantumcoin_node.js")
MINER_PID=$(pgrep -f "real_miner.js")

if [ ! -z "$NODE_PID" ]; then
    echo "✅ Real Node: Running (PID: $NODE_PID)"
else
    echo "❌ Real Node: Not running"
fi

if [ ! -z "$MINER_PID" ]; then
    echo "✅ Real Miner: Running (PID: $MINER_PID)"
else
    echo "❌ Real Miner: Not running"
fi

echo ""

# Get real-time mining info from node
echo "📊 Current Blockchain Status:"
echo "-----------------------------"
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getblockchaininfo","params":{},"id":1}' | \
  python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    result = data.get('result', {})
    print(f\"🏗️  Block Height: {result.get('height', 0)}\")
    print(f\"🔗 Best Block Hash: {result.get('bestblockhash', 'N/A')[:16]}...\")
    print(f\"⚖️  Difficulty: 0x{hex(result.get('difficulty', 0))[2:]}\")
    print(f\"🎯 Target: {result.get('target', 'N/A')[:20]}...\")
    supply = result.get('supply', {})
    print(f\"💰 Current Supply: {supply.get('current', 0) / 100000000:.2f} QTC\")
    print(f\"💎 Max Supply: {supply.get('max', 0) / 100000000:.0f} QTC\")
    print(f\"🚫 Premine: {supply.get('premine', 0)} QTC (Fair Launch)\")
except Exception as e:
    print(f'❌ Node not responding: {e}')
"

echo ""
echo "⛏️  Current Mining Status:"
echo "-------------------------"
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getmininginfo","params":{},"id":2}' | \
  python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    result = data.get('result', {})
    print(f\"🏗️  Mining Height: {result.get('height', 0) + 1}\")
    print(f\"💎 Block Reward: {result.get('rewardQTC', 0)} QTC\")
    print(f\"🎯 Target Block Time: {result.get('targetBlockTime', 0)}s ({result.get('targetBlockTime', 0)//60} minutes)\")
    print(f\"📊 Blocks Until Difficulty Adjustment: {result.get('blocksUntilAdjustment', 0)}\")
    print(f\"🔄 Blocks Until Halving: {result.get('nextHalving', 0):,}\")
    print(f\"⚖️  Current Difficulty: 0x{hex(result.get('difficulty', 0))[2:]}\")
    if result.get('avgBlockTime', 0) > 0:
        print(f\"⏱️  Average Block Time: {result.get('avgBlockTime', 0)}s\")
except Exception as e:
    print(f'❌ Mining info not available: {e}')
"

echo ""

# Show latest mining activity
if [ -f "real_mining.log" ]; then
    echo "🔥 Latest Mining Activity:"
    echo "--------------------------"
    
    # Show hash rate and progress
    tail -5 real_mining.log | grep -E "(Hash Rate:|BLOCK FOUND|Block accepted|earned)" | tail -3
    
    echo ""
    
    # Check for recent blocks found
    BLOCKS_FOUND=$(grep -c "Block accepted" real_mining.log 2>/dev/null || echo "0")
    TOTAL_EARNINGS=$(grep "Total earnings:" real_mining.log | tail -1 | grep -o "[0-9.]* QTC" | head -1 || echo "0 QTC")
    
    echo "🏆 Mining Performance:"
    echo "  • Blocks Found: $BLOCKS_FOUND"
    echo "  • Total Earnings: $TOTAL_EARNINGS"
    
    # Show current hash rate if available
    LATEST_HASHRATE=$(tail -1 real_mining.log | grep -o "[0-9,]* H/s" | tail -1 || echo "Calculating...")
    echo "  • Current Hash Rate: $LATEST_HASHRATE"
    
else
    echo "❌ Real mining log not found"
fi

echo ""
echo "💡 Real Mining Commands:"
echo "------------------------"
echo "• Live mining: tail -f real_mining.log"
echo "• Node logs: tail -f real_node.log"
echo "• Stop miner: pkill -f real_miner.js"
echo "• Stop node: pkill -f real_quantumcoin_node.js"
echo "• Blockchain info: curl -X POST http://localhost:8545 -H 'Content-Type: application/json' -d '{\"method\":\"getblockchaininfo\"}'"

echo ""
echo "⚠️  REAL BITCOIN-LIKE MINING:"
echo "• Difficulty adjusts every 144 blocks"
echo "• Target: 10-minute blocks (600 seconds)"  
echo "• Halving every 105,120 blocks (~2 years)"
echo "• Double SHA256 proof-of-work"
echo "• Multi-threaded CPU mining"
