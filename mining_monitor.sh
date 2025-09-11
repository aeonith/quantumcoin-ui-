#!/bin/bash
# QuantumCoin Mining Monitor Script

echo "🔍 QuantumCoin Mining Monitor"
echo "============================="

# Check if processes are running
NODE_PID=$(pgrep -f "quick_node_server.js")
MINER_PID=$(pgrep -f "simple_miner.js")

if [ ! -z "$NODE_PID" ]; then
    echo "✅ Node Server: Running (PID: $NODE_PID)"
else
    echo "❌ Node Server: Not running"
fi

if [ ! -z "$MINER_PID" ]; then
    echo "✅ Miner: Running (PID: $MINER_PID)"
else
    echo "❌ Miner: Not running"
fi

echo ""

# Show latest mining stats
if [ -f "mining.log" ]; then
    echo "📊 Latest Mining Statistics:"
    echo "----------------------------"
    
    # Extract latest block info
    LATEST_BLOCK=$(grep "Successfully mined block" mining.log | tail -1 | grep -o "#[0-9]*" | sed 's/#//')
    TOTAL_EARNINGS=$(grep "Total earnings:" mining.log | tail -1 | grep -o "[0-9]* QTC" | head -1)
    
    if [ ! -z "$LATEST_BLOCK" ]; then
        echo "🏆 Latest Block Mined: #$LATEST_BLOCK"
        echo "💰 Total Earnings: $TOTAL_EARNINGS"
        
        # Calculate approximate mining rate (blocks per minute)
        FIRST_BLOCK_TIME=$(grep "Successfully mined block #1" mining.log -A1 | head -1 | grep -o '[0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}.*' || echo "")
        if [ ! -z "$FIRST_BLOCK_TIME" ]; then
            echo "⚡ Mining Rate: Very Fast (simplified PoW)"
        fi
        
        echo ""
        echo "📈 Recent Mining Activity (last 5 blocks):"
        echo "----------------------------------------"
        grep -A4 "Successfully mined block" mining.log | tail -20 | grep -E "(Successfully mined|Hash:|Time:|Total earnings)" | tail -8
    else
        echo "⏳ No blocks mined yet..."
    fi
else
    echo "❌ Mining log not found"
fi

echo ""
echo "🌐 Node RPC Test:"
echo "----------------"
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getinfo","params":{},"id":1}' | \
  python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    result = data.get('result', {})
    print(f\"📡 Network: {result.get('network', 'Unknown')}\")
    print(f\"🏗️  Block Height: {result.get('height', 0)}\")
    print(f\"👥 Peers: {result.get('peers', 0)}\")
    print(f\"💎 Max Supply: {result.get('supply', {}).get('max', 0) / 100000000} QTC\")
except:
    print('❌ Node not responding')
"

echo ""
echo "💡 Commands to monitor mining:"
echo "------------------------------"
echo "• Watch live: tail -f mining.log"
echo "• Check status: ./mining_monitor.sh"
echo "• Stop miner: pkill -f simple_miner.js"
echo "• Stop node: pkill -f quick_node_server.js"
