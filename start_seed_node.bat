@echo off
REM QuantumCoin Seed Node Startup Script (Windows)
REM Starts a seed node for the QuantumCoin network

echo 🚀 Starting QuantumCoin Seed Node
echo ==================================

REM Configuration
set DATA_DIR=%USERPROFILE%\.qtc\seed
set CHAIN_SPEC=.\chain_spec.toml
set P2P_PORT=30333
set RPC_PORT=8545
set P2P_LISTEN=0.0.0.0:%P2P_PORT%
set RPC_LISTEN=127.0.0.1:%RPC_PORT%

REM Create data directory if it doesn't exist
if not exist "%DATA_DIR%" mkdir "%DATA_DIR%"

echo 📁 Data Directory: %DATA_DIR%
echo 🌐 P2P Listen: %P2P_LISTEN%
echo 🔧 RPC Listen: %RPC_LISTEN%
echo 📋 Chain Spec: %CHAIN_SPEC%
echo.

REM Check if binary exists
if not exist ".\target\release\qc-node.exe" (
    echo ❌ Error: qc-node.exe binary not found at .\target\release\qc-node.exe
    echo Run: cargo build --workspace --release
    pause
    exit /b 1
)

REM Check if chain spec exists
if not exist "%CHAIN_SPEC%" (
    echo ❌ Error: Chain spec not found at %CHAIN_SPEC%
    pause
    exit /b 1
)

echo ✅ Pre-flight checks passed
echo.
echo 🎯 Starting seed node...
echo    - P2P network on port %P2P_PORT%
echo    - RPC interface on port %RPC_PORT%
echo    - Data stored in %DATA_DIR%
echo.
echo Press Ctrl+C to stop the node
echo.

REM Start the seed node
.\target\release\qc-node.exe ^
  --data-dir "%DATA_DIR%" ^
  --chain-spec "%CHAIN_SPEC%" ^
  --p2p-listen "%P2P_LISTEN%" ^
  --rpc-listen "%RPC_LISTEN%" ^
  --log-level info ^
  --enable-rpc ^
  --seed-node ^
  %*
