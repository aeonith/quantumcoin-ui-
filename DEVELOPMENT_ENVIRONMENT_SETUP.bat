@echo off
echo Setting up QuantumCoin Development Environment...
echo.

REM Check if Rust is installed
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Rust not found. Installing Rust...
    echo Please install Rust from: https://rustup.rs/
    echo After installation, run this script again.
    pause
    exit /b 1
)

echo Rust found: 
cargo --version

REM Check if Node.js is installed
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Node.js not found. Installing Node.js...
    echo Please install Node.js from: https://nodejs.org/
    echo After installation, run this script again.
    pause
    exit /b 1
)

echo Node.js found:
node --version

REM Build the Rust backend
echo.
echo Building Rust backend...
cargo build --release
if %errorlevel% neq 0 (
    echo Build failed! Check error messages above.
    pause
    exit /b 1
)

REM Install Node.js dependencies if package.json exists
if exist package.json (
    echo.
    echo Installing Node.js dependencies...
    npm install
)

echo.
echo ========================================
echo Development Environment Setup Complete!
echo ========================================
echo.
echo To start the blockchain node:
echo   cargo run --release --bin quantumcoin-integrated node --genesis
echo.
echo To start the web interface:
echo   Open index.html in your browser
echo.
echo For production deployment, see PRODUCTION_DEPLOYMENT.md
echo.
pause
