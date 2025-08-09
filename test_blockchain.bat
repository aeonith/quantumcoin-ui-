@echo off
echo Testing QuantumCoin Blockchain Setup...
echo.

REM Test Rust installation
echo Testing Rust installation...
cargo --version
if %errorlevel% neq 0 (
    echo ERROR: Rust/Cargo not found!
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Test Node.js installation  
echo Testing Node.js installation...
node --version
if %errorlevel% neq 0 (
    echo ERROR: Node.js not found!
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

echo.
echo Building QuantumCoin blockchain...
cargo build --release
if %errorlevel% neq 0 (
    echo Build failed! Check error messages above.
    pause
    exit /b 1
)

echo.
echo ========================================
echo ✅ QuantumCoin Setup Complete!
echo ========================================
echo.
echo To start your blockchain network:
echo   1. Genesis Node: cargo run --release --bin quantumcoin-integrated node --genesis
echo   2. Open explorer.html in your browser
echo   3. Open mining.html to start mining
echo.
echo Your blockchain is ready for production!
echo.
pause
