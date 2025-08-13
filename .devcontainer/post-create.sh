#!/bin/bash
set -e

echo "🚀 Setting up QuantumCoin development environment..."

# Install pnpm globally
npm install -g pnpm@latest

# Install cargo utilities
cargo install cargo-audit cargo-deny cargo-edit cargo-watch cargo-llvm-cov

# Install additional tools for SBOM and security
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin
curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin

# Install cosign for image signing
curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
sudo chmod +x /usr/local/bin/cosign

# Install protobuf compiler
sudo apt-get update && sudo apt-get install -y protobuf-compiler jq

# Set up git hooks directory
git config core.hooksPath .githooks
chmod +x .githooks/*

# Install UI dependencies if package.json exists in ui/
if [ -f "ui/package.json" ]; then
    cd ui && pnpm install && cd ..
fi

# Install root dependencies if package.json exists  
if [ -f "package.json" ]; then
    pnpm install
fi

# Build Rust components
cargo check --workspace

echo "✅ Development environment setup complete!"
echo ""
echo "Available commands:"
echo "  make dev      - Start development servers"
echo "  make test     - Run all tests"  
echo "  make lint     - Lint all code"
echo "  make build    - Build all components"
echo "  make up       - Start with docker-compose"
echo ""
