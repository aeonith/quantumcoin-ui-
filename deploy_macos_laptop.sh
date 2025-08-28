#!/bin/bash
# QuantumCoin Node - macOS Laptop Deployment Script
# Sets up node to run while laptop is active

set -e

echo "🍎 QuantumCoin Node - macOS Laptop Setup"
echo "========================================"

NODE_DIR="$HOME/.qtc"
CONFIG_DIR="$NODE_DIR/config"
SERVICE_NAME="com.quantumcoin.node"
PLIST_PATH="$HOME/Library/LaunchAgents/${SERVICE_NAME}.plist"

# Check if binary exists
if [ ! -f "./target/release/qc-node" ]; then
    echo "❌ Binary not found: ./target/release/qc-node"
    echo "Run: cargo build --workspace --release"
    exit 1
fi

echo "📁 Creating directories..."
mkdir -p "$NODE_DIR/logs" "$CONFIG_DIR"

echo "📦 Installing files..."
cp target/release/qc-node "$NODE_DIR/"
cp chain_spec.toml "$NODE_DIR/"
cp config/node.toml "$CONFIG_DIR/"

if [ -f "genesis.json" ]; then
    cp genesis.json "$NODE_DIR/"
else
    echo "⚠️  genesis.json not found - will be generated on first run"
fi

# Update config paths for macOS
sed -i '' "s|path = \"~/.qtc\"|path = \"$NODE_DIR\"|" "$CONFIG_DIR/node.toml"
sed -i '' "s|spec = \"./chain_spec.toml\"|spec = \"$NODE_DIR/chain_spec.toml\"|" "$CONFIG_DIR/node.toml"
sed -i '' "s|genesis = \"./genesis.json\"|genesis = \"$NODE_DIR/genesis.json\"|" "$CONFIG_DIR/node.toml"

echo "🔧 Setting permissions..."
chmod +x "$NODE_DIR/qc-node"

echo "📋 Creating launchd service (optional - runs automatically)..."
cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${SERVICE_NAME}</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>$NODE_DIR/qc-node</string>
        <string>--config</string>
        <string>$CONFIG_DIR/node.toml</string>
    </array>
    
    <key>WorkingDirectory</key>
    <string>$NODE_DIR</string>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>$NODE_DIR/logs/stdout.log</string>
    
    <key>StandardErrorPath</key>
    <string>$NODE_DIR/logs/stderr.log</string>
    
    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>
</dict>
</plist>
EOF

echo ""
echo "✅ QuantumCoin Node Setup Complete!"
echo ""
echo "🎯 Choose your preferred method:"
echo ""
echo "1️⃣  Manual (tmux - recommended for laptop):"
echo "   brew install tmux"
echo "   tmux new -s qtc"
echo "   $NODE_DIR/qc-node --config $CONFIG_DIR/node.toml"
echo "   # Detach: Ctrl-b then d"
echo "   # Reattach: tmux attach -t qtc"
echo ""
echo "2️⃣  Automatic (launchd service):"
echo "   launchctl load $PLIST_PATH"
echo "   launchctl start $SERVICE_NAME"
echo ""
echo "📊 Service Management (if using launchd):"
echo "   launchctl start $SERVICE_NAME"
echo "   launchctl stop $SERVICE_NAME"
echo "   launchctl unload $PLIST_PATH  # Remove service"
echo ""
echo "📄 Logs: $NODE_DIR/logs/"
echo "⚙️  Config: $CONFIG_DIR/node.toml"
echo ""
echo "🧪 Test RPC:"
echo "   curl -X POST http://127.0.0.1:8545 \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"jsonrpc\":\"2.0\",\"method\":\"qc_blockNumber\",\"params\":{},\"id\":1}'"
echo ""
echo "💡 Tip: The tmux method is better for laptops that sleep/wake"
