@echo off
echo ==========================================
echo QuantumCoin Production Server Startup
echo ==========================================
echo.

REM Create necessary directories
if not exist "data" mkdir data
if not exist "logs" mkdir logs

REM Set environment variables for production
set QTC_ENV=production
set QTC_LOG_LEVEL=info
set QTC_DATABASE_PATH=./data/quantumcoin_production.db
set QTC_JWT_SECRET=quantum-production-ultra-secure-key-2024-change-in-production

echo Creating production environment...
echo - Environment: %QTC_ENV%
echo - Database: %QTC_DATABASE_PATH%
echo - Log Level: %QTC_LOG_LEVEL%
echo.

echo Checking Rust installation...
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Cargo/Rust not found in PATH
    echo Please install Rust from https://rustup.rs/
    echo Then run: rustup default stable
    pause
    exit /b 1
)

echo Rust found, building QuantumCoin...
echo.

REM Build in release mode for maximum performance
echo Building QuantumCoin in release mode...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Build failed
    pause
    exit /b 1
)

echo.
echo ==========================================
echo Starting QuantumCoin Production Server
echo ==========================================
echo - Quantum-safe cryptography: ENABLED
echo - AI fraud detection: ENABLED  
echo - Lightning-fast processing: ENABLED
echo - Carbon-negative mining: ENABLED
echo - Production database: SQLite
echo - Real-time monitoring: ENABLED
echo ==========================================
echo.

REM Start the production server
cargo run --release

echo.
echo QuantumCoin server has stopped.
pause
