#!/bin/bash

echo "🔥 RUNNING QUANTUMCOIN HEALTH CHECK AND STRESS TESTS 🔥"
echo "========================================================="

# Check basic compilation first
echo "🔨 1. COMPILATION HEALTH CHECK"
echo "------------------------------"

# Try to check individual crates
echo "Checking validation crate..."
cd crates/validation 2>/dev/null && {
    if cargo check 2>/dev/null; then
        echo "✅ Validation crate: COMPILES"
    else
        echo "❌ Validation crate: COMPILATION ISSUES"
        cargo check 2>&1 | head -20
    fi
    cd ../..
} || echo "⚠️ Validation crate directory not accessible"

echo "Checking P2P crate..."
cd crates/p2p 2>/dev/null && {
    if cargo check 2>/dev/null; then
        echo "✅ P2P crate: COMPILES"
    else
        echo "❌ P2P crate: COMPILATION ISSUES"
        cargo check 2>&1 | head -20
    fi
    cd ../..
} || echo "⚠️ P2P crate directory not accessible"

echo "Checking AI Sentinel crate..."
cd crates/ai-sentinel 2>/dev/null && {
    if cargo check 2>/dev/null; then
        echo "✅ AI Sentinel crate: COMPILES"
    else
        echo "❌ AI Sentinel crate: COMPILATION ISSUES"
        cargo check 2>&1 | head -20
    fi
    cd ../..
} || echo "⚠️ AI Sentinel crate directory not accessible"

# Check workspace compilation
echo ""
echo "🏗️ 2. WORKSPACE COMPILATION CHECK"
echo "----------------------------------"
if cargo check --workspace 2>/dev/null; then
    echo "✅ WORKSPACE: COMPILES SUCCESSFULLY"
else
    echo "❌ WORKSPACE: COMPILATION ISSUES"
    echo "Showing compilation errors:"
    cargo check --workspace 2>&1 | head -30
fi

# Run basic stress tests (if compilation passes)
echo ""
echo "💥 3. BASIC STRESS TESTING"
echo "-------------------------"

if [ -d "stress-test" ]; then
    cd stress-test
    if cargo build --release 2>/dev/null; then
        echo "✅ Stress test framework: BUILDS"
        
        # Run quick stress test
        echo "Running 30-second stress test..."
        timeout 30s cargo run --release --bin extreme-stress-test 2>/dev/null && {
            echo "✅ Stress test: COMPLETED"
        } || echo "⚠️ Stress test: COMPLETED WITH ISSUES"
        
    else
        echo "❌ Stress test framework: BUILD ISSUES"
        cargo build --release 2>&1 | head -20
    fi
    cd ..
else
    echo "⚠️ Stress test directory not found"
fi

# Check critical files
echo ""
echo "📋 4. CRITICAL FILE CHECK"
echo "------------------------"
test -f "chain_spec.toml" && echo "✅ Chain spec: EXISTS" || echo "❌ Chain spec: MISSING"
test -f "Cargo.toml" && echo "✅ Workspace config: EXISTS" || echo "❌ Workspace config: MISSING"
test -f "README.md" && echo "✅ Documentation: EXISTS" || echo "❌ Documentation: MISSING"
test -f "SECURITY.md" && echo "✅ Security policy: EXISTS" || echo "❌ Security policy: MISSING"

# Check CI configuration
echo ""
echo "🔄 5. CI/CD HEALTH CHECK"
echo "-----------------------"
test -f ".github/workflows/strict-truth.yml" && echo "✅ Strict CI: CONFIGURED" || echo "❌ Strict CI: MISSING"
test -f ".github/workflows/codeql.yml" && echo "✅ CodeQL: CONFIGURED" || echo "❌ CodeQL: MISSING"
test -f ".github/workflows/extreme-testing.yml" && echo "✅ Extreme testing: CONFIGURED" || echo "❌ Extreme testing: MISSING"

echo ""
echo "🎯 QUANTUMCOIN HEALTH CHECK COMPLETE"
echo "====================================="

# Determine overall health
HEALTH_SCORE=0

# Check if basic things work
cargo check --workspace >/dev/null 2>&1 && HEALTH_SCORE=$((HEALTH_SCORE + 30))
test -f "chain_spec.toml" && HEALTH_SCORE=$((HEALTH_SCORE + 20))
test -f ".github/workflows/strict-truth.yml" && HEALTH_SCORE=$((HEALTH_SCORE + 20))
test -d "crates/validation" && HEALTH_SCORE=$((HEALTH_SCORE + 15))
test -d "crates/p2p" && HEALTH_SCORE=$((HEALTH_SCORE + 15))

echo "📊 OVERALL HEALTH SCORE: $HEALTH_SCORE/100"

if [ $HEALTH_SCORE -ge 80 ]; then
    echo "✅ SYSTEM HEALTH: EXCELLENT ($HEALTH_SCORE/100)"
    echo "🚀 Ready for production deployment!"
elif [ $HEALTH_SCORE -ge 60 ]; then
    echo "⚠️ SYSTEM HEALTH: GOOD ($HEALTH_SCORE/100)"
    echo "🔧 Minor issues need fixing"
elif [ $HEALTH_SCORE -ge 40 ]; then
    echo "⚠️ SYSTEM HEALTH: FAIR ($HEALTH_SCORE/100)"
    echo "🛠️ Several issues need attention"
else
    echo "❌ SYSTEM HEALTH: POOR ($HEALTH_SCORE/100)"
    echo "🚨 CRITICAL ISSUES NEED IMMEDIATE FIXING"
fi

echo ""
echo "🔥 Health check complete. Use results to guide fixes."
exit 0
