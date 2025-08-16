@echo off
REM QuantumCoin Genesis Block Reproduction Verification Script
REM This script verifies that genesis blocks can be reproduced deterministically

echo QuantumCoin Genesis Block Reproduction Verification
echo ===================================================
echo.

REM Build the CLI tool
echo Building genesis CLI tool...
cargo build --release --bin genesis-cli
if %errorlevel% neq 0 (
    echo Error: Failed to build genesis CLI tool
    exit /b 1
)

REM Create test directory
if not exist "genesis_test" mkdir genesis_test

echo.
echo Testing deterministic reproduction...
echo.

REM Generate mainnet genesis block twice with same parameters
echo Generating first mainnet genesis block...
target\release\genesis-cli.exe mainnet ^
    --output genesis_test\mainnet_test1.json ^
    --format json

if %errorlevel% neq 0 (
    echo Error: Failed to generate first mainnet genesis block
    exit /b 1
)

echo Generating second mainnet genesis block...
target\release\genesis-cli.exe mainnet ^
    --output genesis_test\mainnet_test2.json ^
    --format json

if %errorlevel% neq 0 (
    echo Error: Failed to generate second mainnet genesis block
    exit /b 1
)

REM Generate testnet genesis block twice
echo Generating first testnet genesis block...
target\release\genesis-cli.exe testnet ^
    --output genesis_test\testnet_test1.json ^
    --format json

echo Generating second testnet genesis block...
target\release\genesis-cli.exe testnet ^
    --output genesis_test\testnet_test2.json ^
    --format json

echo.
echo Comparing genesis blocks...

REM Compare files using FC command
echo Comparing mainnet genesis blocks:
fc /B genesis_test\mainnet_test1.json genesis_test\mainnet_test2.json
set mainnet_match=%errorlevel%

echo.
echo Comparing testnet genesis blocks:
fc /B genesis_test\testnet_test1.json genesis_test\testnet_test2.json  
set testnet_match=%errorlevel%

echo.
echo Results:
if %mainnet_match% equ 0 (
    echo ✓ Mainnet genesis blocks are IDENTICAL - deterministic generation working
) else (
    echo ✗ Mainnet genesis blocks DIFFER - deterministic generation failed
)

if %testnet_match% equ 0 (
    echo ✓ Testnet genesis blocks are IDENTICAL - deterministic generation working
) else (
    echo ✗ Testnet genesis blocks DIFFER - deterministic generation failed
)

echo.
echo Testing reproduction from existing blocks...

REM Test reproduce command
target\release\genesis-cli.exe reproduce ^
    --genesis genesis_test\mainnet_test1.json ^
    --spec chain_spec.toml ^
    --output genesis_test\mainnet_reproduced.json

target\release\genesis-cli.exe reproduce ^
    --genesis genesis_test\testnet_test1.json ^
    --spec chain_spec.toml ^
    --output genesis_test\testnet_reproduced.json

echo Comparing reproduced blocks:
fc /B genesis_test\mainnet_test1.json genesis_test\mainnet_reproduced.json
set mainnet_repro_match=%errorlevel%

fc /B genesis_test\testnet_test1.json genesis_test\testnet_reproduced.json
set testnet_repro_match=%errorlevel%

echo.
echo Reproduction Results:
if %mainnet_repro_match% equ 0 (
    echo ✓ Mainnet block reproduction SUCCESS
) else (
    echo ✗ Mainnet block reproduction FAILED
)

if %testnet_repro_match% equ 0 (
    echo ✓ Testnet block reproduction SUCCESS  
) else (
    echo ✗ Testnet block reproduction FAILED
)

echo.
echo Testing custom seed determinism...

REM Test with custom seed
set TEST_SEED=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

target\release\genesis-cli.exe mainnet ^
    --output genesis_test\mainnet_seed1.json ^
    --seed %TEST_SEED% ^
    --format json

target\release\genesis-cli.exe mainnet ^
    --output genesis_test\mainnet_seed2.json ^
    --seed %TEST_SEED% ^
    --format json

fc /B genesis_test\mainnet_seed1.json genesis_test\mainnet_seed2.json
set seed_match=%errorlevel%

if %seed_match% equ 0 (
    echo ✓ Custom seed determinism SUCCESS
) else (
    echo ✗ Custom seed determinism FAILED
)

echo.
echo ========================================
echo Genesis Block Reproduction Test Summary
echo ========================================

set overall_success=1

if %mainnet_match% neq 0 set overall_success=0
if %testnet_match% neq 0 set overall_success=0
if %mainnet_repro_match% neq 0 set overall_success=0
if %testnet_repro_match% neq 0 set overall_success=0
if %seed_match% neq 0 set overall_success=0

if %overall_success% equ 1 (
    echo ✓ ALL TESTS PASSED - Genesis block reproduction is working correctly
    echo   The genesis blocks are completely deterministic and reproducible
) else (
    echo ✗ SOME TESTS FAILED - Genesis block reproduction has issues
    echo   Review the output above for specific failures
)

echo.
echo Cleaning up test files...
rmdir /s /q genesis_test

if %overall_success% equ 1 (
    exit /b 0
) else (
    exit /b 1
)
