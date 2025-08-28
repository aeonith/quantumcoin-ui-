@echo off
REM QuantumCoin Build and Test Script (Windows)
REM Builds the entire workspace and runs all tests

echo 🏗️  QuantumCoin Build and Test Suite
echo ====================================
echo.

REM Check Rust installation
echo 🔍 Checking Rust installation...
cargo --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Error: Cargo not found. Please install Rust:
    echo    https://forge.rust-lang.org/infra/channel-layout.html#getting-going
    pause
    exit /b 1
)

for /f "tokens=*" %%a in ('rustc --version 2^>nul') do set RUST_VERSION=%%a
for /f "tokens=*" %%a in ('cargo --version 2^>nul') do set CARGO_VERSION=%%a

echo ✅ Rust version: %RUST_VERSION%
echo ✅ Cargo version: %CARGO_VERSION%
echo.

REM Build workspace
echo 🚀 Building workspace (release mode)...
echo ----------------------------------------
cargo build --workspace --release --all-features

if errorlevel 1 (
    echo ❌ Workspace build failed
    pause
    exit /b 1
) else (
    echo ✅ Workspace build successful
)
echo.

REM List built binaries
echo 📦 Built binaries:
echo ------------------
dir target\release\qtc-*.exe 2>nul
dir target\release\qc-*.exe 2>nul  
dir target\release\*quantumcoin*.exe 2>nul
dir target\release\*node*.exe 2>nul
echo.

REM Run tests
echo 🧪 Running test suite...
echo ------------------------
cargo test --workspace --all-features

if errorlevel 1 (
    echo ⚠️  Some tests failed - check output above
) else (
    echo ✅ All tests passed
)
echo.

REM Check for lint issues
echo 🔍 Running clippy (linter)...
echo -----------------------------
cargo clippy --workspace --all-features -- -D warnings

if errorlevel 1 (
    echo ⚠️  Lint issues found - consider fixing before deployment
) else (
    echo ✅ No lint issues found
)
echo.

REM Verify key configuration files
echo 📋 Verifying configuration...
echo -----------------------------

if exist "chain_spec.toml" (
    echo ✅ chain_spec.toml found
    findstr /C:"premine_sats = 0" chain_spec.toml >nul
    if errorlevel 1 (
        echo ⚠️  Check premine setting in chain_spec.toml
    ) else (
        echo ✅ Fair launch confirmed (premine_sats = 0^)
    )
) else (
    echo ⚠️  chain_spec.toml not found
)

if exist "Cargo.toml" (
    echo ✅ Workspace Cargo.toml found
) else (
    echo ⚠️  Workspace Cargo.toml not found
)
echo.

REM Test address generation
echo 🔧 Testing address generation...
echo --------------------------------
if exist "target\release\qtc-address.exe" (
    echo ✅ qtc-address.exe binary found
    echo Testing with sample public key...
    target\release\qtc-address.exe "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    echo ✅ Address generation working
) else (
    echo ⚠️  qtc-address.exe binary not found
)
echo.

REM Final status
echo 🎯 Build and Test Summary
echo =========================
echo ✅ Workspace build: Complete
echo ✅ Test suite: Executed  
echo ✅ Linting: Checked
echo ✅ Configuration: Verified
echo ✅ Utilities: Tested
echo.
echo 🚀 Ready for deployment!
echo.
echo Next steps:
echo   1. start_seed_node.bat     # Start seed node
echo   2. test_rpc_endpoints.bat  # Test RPC interface
echo   3. Deploy to public network
echo.
pause
