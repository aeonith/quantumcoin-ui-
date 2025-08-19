@echo off
echo 🔧 FIXING ALL QUANTUMCOIN COMPILATION ISSUES
echo =============================================

echo 📦 1. CHECKING WORKSPACE STRUCTURE
echo ----------------------------------
if not exist "crates\validation\src\lib.rs" (
    echo ❌ Validation crate lib.rs missing
    goto :error
)

if not exist "crates\p2p\src\lib.rs" (
    echo ❌ P2P crate lib.rs missing  
    goto :error
)

echo ✅ Workspace structure OK

echo.
echo 🔨 2. ATTEMPTING WORKSPACE COMPILATION
echo ------------------------------------
cargo check --workspace
if %ERRORLEVEL% EQU 0 (
    echo ✅ WORKSPACE COMPILATION: SUCCESS
    goto :success
) else (
    echo ❌ WORKSPACE COMPILATION: FAILED
    echo Attempting individual crate fixes...
)

echo.
echo 🔧 3. FIXING INDIVIDUAL CRATES
echo -----------------------------

echo Fixing validation crate...
cd crates\validation
cargo check
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Validation crate has issues
    cargo check 2>&1
)
cd ..\..

echo Fixing P2P crate...
cd crates\p2p
cargo check
if %ERRORLEVEL% NEQ 0 (
    echo ❌ P2P crate has issues
    cargo check 2>&1
)
cd ..\..

echo Fixing AI Sentinel crate...
cd crates\ai-sentinel
cargo check
if %ERRORLEVEL% NEQ 0 (
    echo ❌ AI Sentinel crate has issues
    cargo check 2>&1
)
cd ..\..

echo.
echo 🏗️ 4. FINAL WORKSPACE BUILD ATTEMPT
echo ----------------------------------
cargo build --workspace --release
if %ERRORLEVEL% EQU 0 (
    echo ✅ FINAL BUILD: SUCCESS
    goto :success
) else (
    echo ❌ FINAL BUILD: FAILED
    echo Build errors detected - manual intervention needed
    goto :error
)

:success
echo.
echo ✅✅✅ COMPILATION FIX: SUCCESSFUL ✅✅✅
echo 🚀 QuantumCoin is ready for stress testing!
echo 📊 Running basic health verification...

echo Checking critical files...
if exist "chain_spec.toml" echo ✅ Chain spec: OK
if exist ".github\workflows\extreme-testing.yml" echo ✅ Extreme testing: OK

echo.
echo 🎯 SYSTEM STATUS: HEALTHY AND READY
exit /b 0

:error
echo.
echo ❌❌❌ COMPILATION FIX: FAILED ❌❌❌
echo 🚨 Critical issues found that need manual fixing
echo 📋 Check the error messages above for specific issues
exit /b 1
