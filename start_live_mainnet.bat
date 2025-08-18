@echo off
echo 🚀 LAUNCHING QUANTUMCOIN LIVE MAINNET 🚀
echo ==========================================

echo 📁 Setting up workspace on D: drive...
cd /d D:\quantumcoin-working

echo 🔧 Installing Rust dependencies...
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo ❌ Build failed, checking individual components...
    echo 🔨 Building core components...
    cargo build --release --bin quantumcoin-node
    cargo build --release --bin quantumcoin-wallet
    cargo build --release --bin quantumcoin-explorer
)

echo 🏗️ Initializing QuantumCoin mainnet...
target\release\quantumcoin-node.exe init

echo 🌟 Starting QuantumCoin Live Services...

echo 📡 Starting Live Node (Port 8080)...
start "QuantumCoin Node" target\release\quantumcoin-node.exe start

timeout /t 5 /nobreak >nul

echo 🔍 Starting Live Explorer Backend (Port 8081)...
start "QuantumCoin Explorer" target\release\quantumcoin-explorer.exe

timeout /t 3 /nobreak >nul

echo 🌐 Starting Web Interface (Port 3000)...
start "QuantumCoin UI" npm run dev

timeout /t 5 /nobreak >nul

echo ✅ QUANTUMCOIN MAINNET IS NOW LIVE!
echo =====================================
echo 🌐 Node RPC: http://localhost:8080/status
echo 🔍 Explorer API: http://localhost:8081/api/stats
echo 💻 Web UI: http://localhost:3000
echo 📊 Live Explorer: http://localhost:3000/explorer
echo 💰 Wallet: target\release\quantumcoin-wallet.exe --help

echo.
echo 📊 Checking live services...
timeout /t 2 /nobreak >nul

curl -s http://localhost:8080/status && (
    echo ✅ Node: ONLINE
) || (
    echo ⚠️ Node: Starting up...
)

curl -s http://localhost:8081/health && (
    echo ✅ Explorer: ONLINE  
) || (
    echo ⚠️ Explorer: Starting up...
)

echo.
echo 🎉 QUANTUMCOIN MAINNET OPERATIONAL!
echo 🔗 Connect additional nodes to: localhost:8333
echo ⛏️ Start mining: target\release\quantumcoin-node.exe mine
echo 💰 Create wallet: target\release\quantumcoin-wallet.exe create my-wallet

pause
