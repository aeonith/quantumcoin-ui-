#!/bin/bash
# QuantumCoin Genesis Block Generation Script
# This script generates the official mainnet and testnet genesis blocks

set -e  # Exit on any error

echo "QuantumCoin Genesis Block Generation"
echo "===================================="
echo

# Check if Rust is available
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Rust."
    echo "Download from: https://rustup.rs/"
    exit 1
fi

# Create genesis directory if it doesn't exist
mkdir -p genesis

echo "Building genesis generation tools..."
cargo build --release --bin genesis-cli

echo
echo "Generating mainnet genesis block..."
./target/release/genesis-cli mainnet \
    --output genesis/mainnet_genesis.json \
    --format json

echo
echo "Generating testnet genesis block..."
./target/release/genesis-cli testnet \
    --output genesis/testnet_genesis.json \
    --format json

echo
echo "Verifying mainnet genesis block..."
./target/release/genesis-cli verify \
    --genesis genesis/mainnet_genesis.json \
    --spec chain_spec.toml \
    --detailed

echo
echo "Verifying testnet genesis block..."
./target/release/genesis-cli verify \
    --genesis genesis/testnet_genesis.json \
    --spec chain_spec.toml \
    --detailed

# Create binary versions for efficient storage
echo
echo "Creating binary versions..."
./target/release/genesis-cli mainnet \
    --output genesis/mainnet_genesis.bin \
    --format binary

./target/release/genesis-cli testnet \
    --output genesis/testnet_genesis.bin \
    --format binary

# Generate hex versions for easy inspection
./target/release/genesis-cli mainnet \
    --output genesis/mainnet_genesis.hex \
    --format hex

./target/release/genesis-cli testnet \
    --output genesis/testnet_genesis.hex \
    --format hex

echo
echo "Genesis block generation completed successfully!"
echo
echo "Files created:"
echo "  genesis/mainnet_genesis.json - Mainnet genesis block (JSON)"
echo "  genesis/mainnet_genesis.bin  - Mainnet genesis block (binary)"
echo "  genesis/mainnet_genesis.hex  - Mainnet genesis block (hex)"
echo "  genesis/testnet_genesis.json - Testnet genesis block (JSON)"
echo "  genesis/testnet_genesis.bin  - Testnet genesis block (binary)"
echo "  genesis/testnet_genesis.hex  - Testnet genesis block (hex)"
echo

# Show genesis info
echo "Mainnet Genesis Block Information:"
echo "----------------------------------"
./target/release/genesis-cli info --genesis genesis/mainnet_genesis.json

echo
echo "Testnet Genesis Block Information:"
echo "----------------------------------"
./target/release/genesis-cli info --genesis genesis/testnet_genesis.json

echo
echo "Genesis blocks are ready for integration!"
