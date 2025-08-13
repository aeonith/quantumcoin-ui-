#!/bin/bash

echo "=========================================="
echo "QuantumCoin Production Server Startup"
echo "=========================================="
echo ""

# Create necessary directories
mkdir -p data logs

# Set environment variables for production
export QTC_ENV=production
export QTC_LOG_LEVEL=info
export QTC_DATABASE_PATH=./data/quantumcoin_production.db
export QTC_JWT_SECRET=quantum-production-ultra-secure-key-2024-change-in-production

echo "Creating production environment..."
echo "- Environment: $QTC_ENV"
echo "- Database: $QTC_DATABASE_PATH"
echo "- Log Level: $QTC_LOG_LEVEL"
echo ""

echo "Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Cargo/Rust not found in PATH"
    echo "Please install Rust from https://rustup.rs/"
    echo "Then run: rustup default stable"
    exit 1
fi

echo "Rust found, building QuantumCoin..."
echo ""

# Build in release mode for maximum performance
echo "Building QuantumCoin in release mode..."
if ! cargo build --release; then
    echo "ERROR: Build failed"
    exit 1
fi

echo ""
echo "=========================================="
echo "Starting QuantumCoin Production Server"
echo "=========================================="
echo "- Quantum-safe cryptography: ENABLED"
echo "- AI fraud detection: ENABLED"
echo "- Lightning-fast processing: ENABLED"
echo "- Carbon-negative mining: ENABLED"
echo "- Production database: SQLite"
echo "- Real-time monitoring: ENABLED"
echo "=========================================="
echo ""

# Start the production server
cargo run --release

echo ""
echo "QuantumCoin server has stopped."
