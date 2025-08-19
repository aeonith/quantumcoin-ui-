#!/bin/bash
# Build and upload actual release artifacts to GitHub Release
# This creates real downloadable binaries + SHA256SUMS

set -e

VERSION="v1.0.1-mainnet"
RELEASE_DIR="release-artifacts"

echo "🚀 Building Real Release Artifacts for $VERSION"
echo "=============================================="

# Clean and setup
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

echo "🔨 Building backend API server..."
cd backend
cargo build --release
cd ..

# Copy actual binaries
cp backend/target/release/quantumcoin-api "$RELEASE_DIR/quantumcoin-api-linux-x64"

# Build additional targets if available
if [ -f "target/release/quantumcoin-node" ]; then
    cp target/release/quantumcoin-node "$RELEASE_DIR/quantumcoin-node-linux-x64"
fi

# Package UI build
echo "🎨 Building frontend..."
npm run build

# Create tarballs
tar -czf "$RELEASE_DIR/quantumcoin-api-linux-x64.tar.gz" -C "$RELEASE_DIR" quantumcoin-api-linux-x64
rm "$RELEASE_DIR/quantumcoin-api-linux-x64"

if [ -f "$RELEASE_DIR/quantumcoin-node-linux-x64" ]; then
    tar -czf "$RELEASE_DIR/quantumcoin-node-linux-x64.tar.gz" -C "$RELEASE_DIR" quantumcoin-node-linux-x64
    rm "$RELEASE_DIR/quantumcoin-node-linux-x64"
fi

tar -czf "$RELEASE_DIR/quantumcoin-explorer.tar.gz" -C out .

# Copy essential files
cp chain_spec.toml "$RELEASE_DIR/"
cp SECURITY.md "$RELEASE_DIR/"
cp EXCHANGE_VERIFICATION.md "$RELEASE_DIR/"

# Generate SHA256SUMS
cd "$RELEASE_DIR"
sha256sum *.tar.gz *.toml *.md > SHA256SUMS.txt

echo ""
echo "📋 SHA256SUMS.txt contents:"
cat SHA256SUMS.txt

# Create GPG signature (if key available)
if command -v gpg >/dev/null 2>&1; then
    gpg --armor --detach-sign --output SHA256SUMS.txt.sig SHA256SUMS.txt 2>/dev/null || echo "GPG key not available"
fi

echo ""
echo "✅ Release artifacts built:"
ls -la

# Upload to GitHub Release using gh CLI
if command -v gh >/dev/null 2>&1; then
    echo ""
    echo "📤 Uploading to GitHub Release $VERSION..."
    
    # Upload all artifacts to existing release
    gh release upload "$VERSION" ./* --clobber
    
    echo "✅ Artifacts uploaded to: https://github.com/aeonith/quantumcoin-ui-/releases/tag/$VERSION"
else
    echo ""
    echo "📤 Manual upload required:"
    echo "Go to: https://github.com/aeonith/quantumcoin-ui-/releases/tag/$VERSION"
    echo "Upload these files to Assets section:"
    ls -1
fi

# Verify download URLs will work
echo ""
echo "🔗 Verification URLs:"
echo "wget https://github.com/aeonith/quantumcoin-ui-/releases/download/$VERSION/quantumcoin-api-linux-x64.tar.gz"
echo "wget https://github.com/aeonith/quantumcoin-ui-/releases/download/$VERSION/SHA256SUMS.txt"
echo "sha256sum -c SHA256SUMS.txt"

cd ..
echo ""
echo "🎉 Real release artifacts ready for exchange verification!"
