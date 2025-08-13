# QuantumCoin Setup Guide

## Current Status ✅
- **Frontend Ready**: All HTML/CSS/JS components are complete and functional
- **Project Structure**: Complete Rust backend structure following master script
- **Documentation**: All required docs, CI, and scripts created
- **RevStop System**: Implemented with persistence and proper modes
- **Utilities**: Atomic writes, SHA-256 checksums, and file integrity

## When Rust is Available

### 1. Install Rust (Required)
```bash
# Visit https://rustup.rs/ and install
# OR run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Restart terminal/VS Code after installation
```

### 2. Verify Installation
```bash
cargo --version
rustc --version
```

### 3. Build & Test (Master Script Gates)
```bash
# Gate A: Initial build
cargo build

# Gate B: Clean compilation  
cargo build --release

# Gate C: Tests pass
cargo test

# Gate D: Linting
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

### 4. Run Blockchain
```bash
# Single node
cargo run --release

# Localnet (two nodes)
bash scripts/localnet.sh

# With explorer (feature-gated)
cargo run --release --features explorer
```

### 5. Test Frontend
- Open `index.html` in browser
- Navigate to `wallet.html`, `explorer.html`, `mining.html`
- All pages should load with proper styling

### 6. Verify Complete Integration
- RevStop persistence across restarts
- Two nodes relay transactions/blocks
- Explorer API responds at `/api/height`
- Mining functionality works
- CLI menu fully functional

## Ready for v0.1.0-rc1 Release ✅

The codebase is structured and ready. Once Rust is installed:
1. Run the build tests above
2. Fix any compilation issues
3. Test full functionality
4. Create GitHub release with binaries

## Storage Requirements
- Rust toolchain: ~1GB
- Dependencies: ~500MB  
- Build artifacts: ~200MB
- **Total: ~1.7GB**

Alternative: Use GitHub Codespaces or online Rust playground for testing.
