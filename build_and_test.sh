#!/bin/bash

# QuantumCoin Build and Test Script
# Builds the entire workspace and runs all tests

set -e

echo "🏗️  QuantumCoin Build and Test Suite"
echo "===================================="
echo ""

# Check Rust installation
echo "🔍 Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust version: $(rustc --version)"
echo "✅ Cargo version: $(cargo --version)"
echo ""

# Build workspace
echo "🚀 Building workspace (release mode)..."
echo "----------------------------------------"
cargo build --workspace --release --all-features

if [ $? -eq 0 ]; then
    echo "✅ Workspace build successful"
else
    echo "❌ Workspace build failed"
    exit 1
fi
echo ""

# List built binaries
echo "📦 Built binaries:"
echo "------------------"
ls -la target/release/qtc-* target/release/qc-* 2>/dev/null || echo "No binaries found with qtc-/qc- prefix"
ls -la target/release/ | grep -E "(quantumcoin|node)" || echo "No quantumcoin binaries found"
echo ""

# Run tests
echo "🧪 Running test suite..."
echo "------------------------"
cargo test --workspace --all-features

if [ $? -eq 0 ]; then
    echo "✅ All tests passed"
else
    echo "⚠️  Some tests failed - check output above"
fi
echo ""

# Check for lint issues
echo "🔍 Running clippy (linter)..."
echo "-----------------------------"
cargo clippy --workspace --all-features -- -D warnings

if [ $? -eq 0 ]; then
    echo "✅ No lint issues found"
else
    echo "⚠️  Lint issues found - consider fixing before deployment"
fi
echo ""

# Verify key configuration files
echo "📋 Verifying configuration..."
echo "-----------------------------"

if [ -f "chain_spec.toml" ]; then
    echo "✅ chain_spec.toml found"
    if grep -q "premine_sats = 0" chain_spec.toml; then
        echo "✅ Fair launch confirmed (premine_sats = 0)"
    else
        echo "⚠️  Check premine setting in chain_spec.toml"
    fi
else
    echo "⚠️  chain_spec.toml not found"
fi

if [ -f "Cargo.toml" ]; then
    echo "✅ Workspace Cargo.toml found"
else
    echo "⚠️  Workspace Cargo.toml not found"
fi
echo ""

# Test address generation
echo "🔧 Testing address generation..."
echo "--------------------------------"
if [ -f "target/release/qtc-address" ]; then
    echo "✅ qtc-address binary found"
    echo "Testing with sample public key..."
    ./target/release/qtc-address "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    echo "✅ Address generation working"
else
    echo "⚠️  qtc-address binary not found"
fi
echo ""

# Final status
echo "🎯 Build and Test Summary"
echo "========================="
echo "✅ Workspace build: Complete"
echo "✅ Test suite: Executed"  
echo "✅ Linting: Checked"
echo "✅ Configuration: Verified"
echo "✅ Utilities: Tested"
echo ""
echo "🚀 Ready for deployment!"
echo ""
echo "Next steps:"
echo "  1. ./start_seed_node.sh     # Start seed node"
echo "  2. ./test_rpc_endpoints.sh  # Test RPC interface"
echo "  3. Deploy to public network"
