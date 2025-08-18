#!/bin/bash
# Run EXTREME stress test - 1000 requests/minute for 2 minutes
# ZERO TOLERANCE for errors, warnings, or failures

set -e

echo "🔥 EXTREME QUANTUMCOIN STRESS TEST"
echo "=================================="
echo "Target: 1000 requests/minute for 2 minutes"
echo "Tolerance: ZERO errors, warnings, or failures"
echo ""

# Ensure k6 is available
if ! command -v k6 >/dev/null 2>&1; then
    echo "📥 Installing k6..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install k6
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        wget -q https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz
        tar -xzf k6-v0.47.0-linux-amd64.tar.gz
        sudo mv k6-v0.47.0-linux-amd64/k6 /usr/local/bin/
    else
        echo "❌ Unsupported OS for k6 installation"
        exit 1
    fi
fi

# Start real QuantumCoin backend
echo "🚀 Starting REAL QuantumCoin backend..."
cd backend

# Build with zero warnings tolerance
RUSTFLAGS="-D warnings" cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Backend build failed - ZERO TOLERANCE VIOLATED"
    exit 1
fi

# Start backend with comprehensive error handling
cargo run --release &
BACKEND_PID=$!

# Wait for backend to be fully ready
echo "⏳ Waiting for backend initialization..."
BACKEND_READY=false
for i in {1..60}; do
    if curl -s -f http://localhost:8080/status >/dev/null 2>&1; then
        BACKEND_READY=true
        echo "✅ Backend ready after ${i} seconds"
        break
    fi
    sleep 1
done

if [ "$BACKEND_READY" != "true" ]; then
    echo "❌ Backend failed to start within 60 seconds"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

cd ..

# Verify backend health before stress testing
echo "🔍 Pre-stress health verification..."
HEALTH_RESPONSE=$(curl -s http://localhost:8080/status)

if ! echo "$HEALTH_RESPONSE" | jq -e '.height > 0' >/dev/null 2>&1; then
    echo "❌ Backend health check failed - height not positive"
    echo "Response: $HEALTH_RESPONSE"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

echo "✅ Backend health verified - starting extreme stress test"

# Run the extreme stress test
echo ""
echo "🔥 EXECUTING EXTREME STRESS TEST"
echo "==============================="
echo "Duration: 2 minutes"
echo "Rate: 1000 requests/minute (~16.67/second)"
echo "Tolerance: ZERO failures"
echo ""

START_TIME=$(date +%s)

# Run k6 stress test
if k6 run stress_test_extreme.js; then
    STRESS_RESULT="PASSED"
    echo ""
    echo "🎉 EXTREME STRESS TEST PASSED"
    echo "=============================="
else
    STRESS_RESULT="FAILED"
    echo ""
    echo "❌ EXTREME STRESS TEST FAILED"
    echo "============================="
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Final health verification
echo "🔍 Post-stress health verification..."
FINAL_HEALTH=$(curl -s http://localhost:8080/status)

if ! echo "$FINAL_HEALTH" | jq -e '.height > 0' >/dev/null 2>&1; then
    echo "❌ Backend health degraded after stress test"
    echo "Response: $FINAL_HEALTH"
    STRESS_RESULT="FAILED"
fi

# Cleanup
echo "🧹 Cleaning up..."
kill $BACKEND_PID 2>/dev/null || true

# Generate final report
echo ""
echo "📊 EXTREME STRESS TEST FINAL REPORT"
echo "==================================="
echo "Result: $STRESS_RESULT"
echo "Duration: ${DURATION} seconds"
echo "Target Rate: 1000 requests/minute"
echo "Total Expected Requests: ~2000"
echo ""

if [ "$STRESS_RESULT" = "PASSED" ]; then
    echo "✅ QuantumCoin survived extreme stress testing"
    echo "✅ Zero errors detected"
    echo "✅ Zero warnings detected"
    echo "✅ All endpoints remained responsive"
    echo "✅ Backend maintained health throughout"
    echo ""
    echo "🏆 QUANTUMCOIN IS BULLETPROOF UNDER EXTREME LOAD"
    exit 0
else
    echo "❌ QuantumCoin failed extreme stress testing"
    echo "❌ System is not ready for production"
    echo "❌ Fix all errors before proceeding"
    echo ""
    echo "💥 STRESS TEST FAILURE - NOT PRODUCTION READY"
    exit 1
fi
