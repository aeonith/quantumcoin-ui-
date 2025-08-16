@echo off
REM QuantumCoin Genesis Block Generation Script
REM This script generates the official mainnet and testnet genesis blocks

echo QuantumCoin Genesis Block Generation
echo ====================================
echo.

REM Check if Rust is available
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Cargo not found. Please install Rust.
    echo Download from: https://rustup.rs/
    exit /b 1
)

echo Building genesis generation tools...
cargo build --release --bin genesis-cli
if %errorlevel% neq 0 (
    echo Error: Failed to build genesis CLI tool
    exit /b 1
)

echo.
echo Generating mainnet genesis block...
target\release\genesis-cli.exe mainnet ^
    --output genesis\mainnet_genesis.json ^
    --format json

if %errorlevel% neq 0 (
    echo Error: Failed to generate mainnet genesis block
    exit /b 1
)

echo.
echo Generating testnet genesis block...
target\release\genesis-cli.exe testnet ^
    --output genesis\testnet_genesis.json ^
    --format json

if %errorlevel% neq 0 (
    echo Error: Failed to generate testnet genesis block
    exit /b 1
)

echo.
echo Verifying mainnet genesis block...
target\release\genesis-cli.exe verify ^
    --genesis genesis\mainnet_genesis.json ^
    --spec chain_spec.toml ^
    --detailed

if %errorlevel% neq 0 (
    echo Error: Mainnet genesis block verification failed
    exit /b 1
)

echo.
echo Verifying testnet genesis block...
target\release\genesis-cli.exe verify ^
    --genesis genesis\testnet_genesis.json ^
    --spec chain_spec.toml ^
    --detailed

if %errorlevel% neq 0 (
    echo Error: Testnet genesis block verification failed
    exit /b 1
)

REM Create binary versions for efficient storage
echo.
echo Creating binary versions...
target\release\genesis-cli.exe mainnet ^
    --output genesis\mainnet_genesis.bin ^
    --format binary

target\release\genesis-cli.exe testnet ^
    --output genesis\testnet_genesis.bin ^
    --format binary

REM Generate hex versions for easy inspection
target\release\genesis-cli.exe mainnet ^
    --output genesis\mainnet_genesis.hex ^
    --format hex

target\release\genesis-cli.exe testnet ^
    --output genesis\testnet_genesis.hex ^
    --format hex

echo.
echo Genesis block generation completed successfully!
echo.
echo Files created:
echo   genesis\mainnet_genesis.json - Mainnet genesis block (JSON)
echo   genesis\mainnet_genesis.bin  - Mainnet genesis block (binary)
echo   genesis\mainnet_genesis.hex  - Mainnet genesis block (hex)
echo   genesis\testnet_genesis.json - Testnet genesis block (JSON)
echo   genesis\testnet_genesis.bin  - Testnet genesis block (binary)
echo   genesis\testnet_genesis.hex  - Testnet genesis block (hex)
echo.

REM Show genesis info
echo Mainnet Genesis Block Information:
echo ----------------------------------
target\release\genesis-cli.exe info --genesis genesis\mainnet_genesis.json

echo.
echo Testnet Genesis Block Information:
echo ----------------------------------
target\release\genesis-cli.exe info --genesis genesis\testnet_genesis.json

echo.
echo Genesis blocks are ready for integration!
pause
