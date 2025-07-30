#!/bin/bash

echo "🚀 Building QuantumCoin for Render deployment..."

# Ensure we have the latest Rust version
rustup update stable
rustup default stable

# Create data directory
mkdir -p data

# Build in release mode with optimizations
echo "📦 Building in release mode..."
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Verify the binary exists
if [ -f "./target/release/quantumcoin" ]; then
    echo "✅ Build successful! Binary ready for deployment."
    ls -la ./target/release/quantumcoin
else
    echo "❌ Build failed! Binary not found."
    exit 1
fi

echo "🎯 QuantumCoin ready for Render!"
