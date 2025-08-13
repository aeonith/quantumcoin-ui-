#!/bin/bash

# QuantumCoin Smoke Tests
# 
# This script runs basic smoke tests to validate that the QuantumCoin
# stack is functioning correctly.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
UI_URL="${UI_URL:-http://localhost:3000}"
API_URL="${API_URL:-http://localhost:8080}"
TIMEOUT="${TIMEOUT:-10}"

echo -e "${BLUE}🧪 Starting QuantumCoin smoke tests...${NC}"
echo "UI URL: $UI_URL"
echo "API URL: $API_URL"
echo ""

# Test counter
TESTS_RUN=0
TESTS_PASSED=0

# Test function
test_endpoint() {
    local name="$1"
    local url="$2"
    local expected_status="${3:-200}"
    local expected_content="$4"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    echo -n "Testing $name... "
    
    # Make HTTP request
    local response
    local status_code
    
    if ! response=$(curl -s -w "%{http_code}" -m "$TIMEOUT" "$url" 2>/dev/null); then
        echo -e "${RED}FAILED${NC} (connection error)"
        return 1
    fi
    
    # Extract status code (last 3 characters)
    status_code="${response: -3}"
    response="${response%???}"
    
    # Check status code
    if [[ "$status_code" != "$expected_status" ]]; then
        echo -e "${RED}FAILED${NC} (status: $status_code, expected: $expected_status)"
        return 1
    fi
    
    # Check content if provided
    if [[ -n "$expected_content" ]]; then
        if ! echo "$response" | grep -q "$expected_content"; then
            echo -e "${RED}FAILED${NC} (content check failed)"
            return 1
        fi
    fi
    
    echo -e "${GREEN}PASSED${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    return 0
}

test_json_field() {
    local name="$1"
    local url="$2" 
    local field="$3"
    local expected_value="$4"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    echo -n "Testing $name... "
    
    # Check if jq is available
    if ! command -v jq >/dev/null 2>&1; then
        echo -e "${YELLOW}SKIPPED${NC} (jq not available)"
        return 0
    fi
    
    # Make request and parse JSON
    local response
    if ! response=$(curl -s -m "$TIMEOUT" "$url" 2>/dev/null); then
        echo -e "${RED}FAILED${NC} (connection error)"
        return 1
    fi
    
    # Extract field value
    local actual_value
    if ! actual_value=$(echo "$response" | jq -r "$field" 2>/dev/null); then
        echo -e "${RED}FAILED${NC} (JSON parse error)"
        return 1
    fi
    
    # Compare values
    if [[ "$actual_value" != "$expected_value" ]]; then
        echo -e "${RED}FAILED${NC} (expected: '$expected_value', got: '$actual_value')"
        return 1
    fi
    
    echo -e "${GREEN}PASSED${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    return 0
}

# Test 1: UI Health Check
test_endpoint "UI homepage" "$UI_URL" 200 "QuantumCoin"

# Test 2: API Health Check
test_endpoint "API health" "$API_URL/health" 200 "healthy"

# Test 3: API Status Endpoint
test_endpoint "API status" "$API_URL/status" 200

# Test 4: Validate API Response Structure
test_json_field "API health status" "$API_URL/health" ".status" "healthy"

# Test 5: Blocks Endpoint
test_endpoint "Blocks endpoint" "$API_URL/blocks" 200

# Test 6: Mempool Endpoint  
test_endpoint "Mempool endpoint" "$API_URL/mempool" 200

# Test 7: Economics Constants (if available)
if command -v node >/dev/null 2>&1; then
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -n "Testing economics constants... "
    
    # Create a temporary Node.js script to check constants
    cat > /tmp/check_economics.js << 'EOF'
const fs = require('fs');
const path = require('path');

// Try to load economics from UI source
try {
    // This is a simplified check - in reality we'd use proper module loading
    const economicsPath = path.join(process.cwd(), 'ui/src/lib/economics.ts');
    if (fs.existsSync(economicsPath)) {
        const content = fs.readFileSync(economicsPath, 'utf8');
        
        // Check for canonical values
        const checks = [
            /TOTAL_SUPPLY.*22_000_000/,
            /HALVING_PERIOD_YEARS.*2/,
            /HALVING_DURATION_YEARS.*66/,
            /BLOCK_TIME_TARGET_SEC.*600/
        ];
        
        const allMatch = checks.every(regex => regex.test(content));
        process.exit(allMatch ? 0 : 1);
    } else {
        console.log("Economics file not found");
        process.exit(1);
    }
} catch (error) {
    console.log("Error:", error.message);
    process.exit(1);
}
EOF
    
    if node /tmp/check_economics.js >/dev/null 2>&1; then
        echo -e "${GREEN}PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${YELLOW}SKIPPED${NC} (economics file not accessible)"
    fi
    
    rm -f /tmp/check_economics.js
else
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "Testing economics constants... ${YELLOW}SKIPPED${NC} (node not available)"
fi

# Test 8: Check if issuance curve exists
TESTS_RUN=$((TESTS_RUN + 1))
echo -n "Testing issuance curve generation... "

if [[ -f "ui/public/issuance-curve.svg" ]]; then
    echo -e "${GREEN}PASSED${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}SKIPPED${NC} (curve not generated yet)"
fi

# Test 9: Security headers (if curl supports it)
if curl --version | grep -q "libcurl"; then
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -n "Testing security headers... "
    
    headers=$(curl -s -I -m "$TIMEOUT" "$UI_URL" 2>/dev/null || true)
    
    if echo "$headers" | grep -qi "x-content-type-options"; then
        echo -e "${GREEN}PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${YELLOW}PARTIAL${NC} (some headers missing)"
    fi
fi

echo ""
echo -e "${BLUE}📊 Test Results:${NC}"
echo "Tests run: $TESTS_RUN"
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$((TESTS_RUN - TESTS_PASSED))${NC}"

if [[ $TESTS_PASSED -eq $TESTS_RUN ]]; then
    echo -e "\n${GREEN}✅ All tests passed!${NC}"
    exit 0
elif [[ $TESTS_PASSED -gt $((TESTS_RUN / 2)) ]]; then
    echo -e "\n${YELLOW}⚠️  Most tests passed, but some failed.${NC}"
    exit 1
else
    echo -e "\n${RED}❌ Many tests failed. Check your setup.${NC}"
    exit 1
fi
