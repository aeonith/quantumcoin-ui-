#!/bin/bash
# QuantumCoin Node - Linux Server Deployment Script
# Deploys a production QuantumCoin node as a systemd service

set -e

echo "🚀 QuantumCoin Node - Linux Server Deployment"
echo "============================================="

# Configuration
QTC_USER="qtc"
QTC_DIR="/opt/qtc"
DATA_DIR="/var/lib/qtc"
SERVICE_NAME="qc-node"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ This script must be run as root (use sudo)"
    exit 1
fi

echo "📁 Creating directories..."
mkdir -p "${QTC_DIR}/config" "${QTC_DIR}/logs" "${DATA_DIR}"

echo "👤 Creating system user..."
if ! id "${QTC_USER}" &>/dev/null; then
    useradd -r -s /usr/sbin/nologin -d "${DATA_DIR}" "${QTC_USER}"
    echo "✅ Created user: ${QTC_USER}"
else
    echo "✅ User already exists: ${QTC_USER}"
fi

# Check if binaries exist
if [ ! -f "./target/release/qc-node" ]; then
    echo "❌ Binary not found: ./target/release/qc-node"
    echo "Run: cargo build --workspace --release"
    exit 1
fi

echo "📦 Installing binaries and configuration..."
cp target/release/qc-node "${QTC_DIR}/"
cp chain_spec.toml "${QTC_DIR}/"
cp genesis.json "${QTC_DIR}/" 2>/dev/null || echo "⚠️ genesis.json not found - will be generated"
cp config/node.toml "${QTC_DIR}/config/"

# Update paths in config for server deployment
sed -i 's|path = "~/.qtc"|path = "/var/lib/qtc"|' "${QTC_DIR}/config/node.toml"
sed -i 's|spec = "./chain_spec.toml"|spec = "/opt/qtc/chain_spec.toml"|' "${QTC_DIR}/config/node.toml"
sed -i 's|genesis = "./genesis.json"|genesis = "/opt/qtc/genesis.json"|' "${QTC_DIR}/config/node.toml"

echo "🔧 Setting permissions..."
chown -R "${QTC_USER}:${QTC_USER}" "${QTC_DIR}" "${DATA_DIR}"
chmod +x "${QTC_DIR}/qc-node"

echo "🔥 Installing systemd service..."
cp config/qc-node.service /etc/systemd/system/
systemctl daemon-reload

echo "🌐 Configuring firewall..."
if command -v ufw &> /dev/null; then
    ufw --force enable
    ufw allow 30333/tcp comment "QuantumCoin P2P"
    ufw reload
    echo "✅ UFW: Opened P2P port 30333"
elif command -v firewall-cmd &> /dev/null; then
    firewall-cmd --permanent --add-port=30333/tcp
    firewall-cmd --reload
    echo "✅ Firewalld: Opened P2P port 30333"
else
    echo "⚠️  No firewall detected - manually open port 30333/tcp"
fi

echo "🎯 Starting service..."
systemctl enable "${SERVICE_NAME}"
systemctl start "${SERVICE_NAME}"

echo ""
echo "✅ QuantumCoin Node Deployment Complete!"
echo ""
echo "📊 Status Commands:"
echo "  sudo systemctl status ${SERVICE_NAME}"
echo "  sudo journalctl -u ${SERVICE_NAME} -f"
echo ""  
echo "🔧 Management Commands:"
echo "  sudo systemctl start ${SERVICE_NAME}"
echo "  sudo systemctl stop ${SERVICE_NAME}"
echo "  sudo systemctl restart ${SERVICE_NAME}"
echo ""
echo "🧪 Test RPC:"
echo "  curl -X POST http://127.0.0.1:8545 \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"jsonrpc\":\"2.0\",\"method\":\"qc_blockNumber\",\"params\":{},\"id\":1}'"
echo ""
echo "📍 Your seed node address: $(curl -s ifconfig.me 2>/dev/null || echo 'YOUR_PUBLIC_IP'):30333"
echo ""
echo "🎉 Node is running! Check logs: sudo journalctl -u ${SERVICE_NAME} -f"
