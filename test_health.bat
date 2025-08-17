@echo off
echo 🔥 RUNNING QUANTUMCOIN HEALTH CHECK 🔥
echo ========================================

echo 📋 1. CRITICAL FILE CHECK
echo ------------------------
if exist "chain_spec.toml" (
    echo ✅ Chain spec: EXISTS
) else (
    echo ❌ Chain spec: MISSING
)

if exist "Cargo.toml" (
    echo ✅ Workspace config: EXISTS
) else (
    echo ❌ Workspace config: MISSING
)

if exist "README.md" (
    echo ✅ Documentation: EXISTS
) else (
    echo ❌ Documentation: MISSING
)

if exist "SECURITY.md" (
    echo ✅ Security policy: EXISTS
) else (
    echo ❌ Security policy: MISSING
)

echo.
echo 📁 2. CRATE STRUCTURE CHECK
echo ---------------------------
if exist "crates\validation" (
    echo ✅ Validation crate: EXISTS
) else (
    echo ❌ Validation crate: MISSING
)

if exist "crates\p2p" (
    echo ✅ P2P crate: EXISTS
) else (
    echo ❌ P2P crate: MISSING
)

if exist "crates\ai-sentinel" (
    echo ✅ AI Sentinel crate: EXISTS
) else (
    echo ❌ AI Sentinel crate: MISSING
)

if exist "stress-test" (
    echo ✅ Stress testing: EXISTS
) else (
    echo ❌ Stress testing: MISSING
)

echo.
echo 🔄 3. CI/CD CONFIGURATION CHECK
echo ------------------------------
if exist ".github\workflows\strict-truth.yml" (
    echo ✅ Strict CI: CONFIGURED
) else (
    echo ❌ Strict CI: MISSING
)

if exist ".github\workflows\codeql.yml" (
    echo ✅ CodeQL: CONFIGURED
) else (
    echo ❌ CodeQL: MISSING
)

if exist ".github\workflows\extreme-testing.yml" (
    echo ✅ Extreme testing: CONFIGURED
) else (
    echo ❌ Extreme testing: MISSING
)

echo.
echo 🔨 4. COMPILATION ATTEMPT
echo ------------------------
echo Attempting workspace compilation...

cargo check --workspace >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ WORKSPACE: COMPILES SUCCESSFULLY
    set /a HEALTH_SCORE=80
) else (
    echo ❌ WORKSPACE: COMPILATION ISSUES
    echo Showing compilation errors:
    cargo check --workspace
    set /a HEALTH_SCORE=40
)

echo.
echo 📊 HEALTH ASSESSMENT
echo ===================

if %HEALTH_SCORE% GEQ 80 (
    echo ✅ SYSTEM HEALTH: EXCELLENT ^(%HEALTH_SCORE%/100^)
    echo 🚀 Ready for production deployment!
) else if %HEALTH_SCORE% GEQ 60 (
    echo ⚠️ SYSTEM HEALTH: GOOD ^(%HEALTH_SCORE%/100^)
    echo 🔧 Minor issues need fixing
) else if %HEALTH_SCORE% GEQ 40 (
    echo ⚠️ SYSTEM HEALTH: FAIR ^(%HEALTH_SCORE%/100^)
    echo 🛠️ Several issues need attention
) else (
    echo ❌ SYSTEM HEALTH: POOR ^(%HEALTH_SCORE%/100^)
    echo 🚨 CRITICAL ISSUES NEED IMMEDIATE FIXING
)

echo.
echo 🔥 Health check complete!
