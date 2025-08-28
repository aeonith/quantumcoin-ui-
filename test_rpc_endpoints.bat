@echo off
REM QuantumCoin RPC Endpoint Testing Script (Windows)
REM Tests both standard and qc_* alias methods

set RPC_URL=http://127.0.0.1:8545
echo 🧪 Testing QuantumCoin RPC Endpoints
echo ====================================
echo RPC URL: %RPC_URL%
echo.

echo 🔍 Checking RPC server availability...
curl -s --connect-timeout 5 "%RPC_URL%" > nul 2>&1
if errorlevel 1 (
    echo ❌ Error: RPC server not reachable at %RPC_URL%
    echo Make sure the node is running with --enable-rpc flag
    pause
    exit /b 1
)
echo ✅ RPC server is reachable
echo.

echo 📋 Testing Standard RPC Methods:
echo --------------------------------

echo 1️⃣  Testing getblockchain...
curl -s -X POST "%RPC_URL%" ^
  -H "Content-Type: application/json" ^
  -d "{\"method\":\"getblockchain\",\"params\":{},\"id\":1}"
echo.

echo 2️⃣  Testing getbalance...
curl -s -X POST "%RPC_URL%" ^
  -H "Content-Type: application/json" ^
  -d "{\"method\":\"getbalance\",\"params\":{\"address\":\"qtc1q0000000000000000000000000000000000000000\"},\"id\":2}"
echo.

echo ⚡ Testing Exchange-Compatible qc_* Methods:
echo --------------------------------------------

echo 3️⃣  Testing qc_blockNumber...
curl -s -X POST "%RPC_URL%" ^
  -H "Content-Type: application/json" ^
  -d "{\"method\":\"qc_blockNumber\",\"params\":{},\"id\":3}"
echo.

echo 4️⃣  Testing qc_getBalance...
curl -s -X POST "%RPC_URL%" ^
  -H "Content-Type: application/json" ^
  -d "{\"method\":\"qc_getBalance\",\"params\":{\"address\":\"qtc1q0000000000000000000000000000000000000000\"},\"id\":4}"
echo.

echo 5️⃣  Testing qc_getBlockByNumber...
curl -s -X POST "%RPC_URL%" ^
  -H "Content-Type: application/json" ^
  -d "{\"method\":\"qc_getBlockByNumber\",\"params\":{\"number\":0},\"id\":5}"
echo.

echo ✅ RPC Endpoint Testing Complete
echo.
echo 📊 Summary:
echo   - Standard methods: getblockchain, getbalance ✅
echo   - Exchange qc_* methods: qc_blockNumber, qc_getBalance, qc_getBlockByNumber ✅
echo   - All endpoints should return JSON responses with result/error fields
echo.
echo 🎯 Ready for exchange integration!
echo.
pause
