#!/bin/bash
# OverDrive-DB SDK Test Script v2.0.0
# Tests all language SDKs

set -e

echo "🧪 Testing OverDrive-DB SDKs v2.0.0..."
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test Python SDK
echo -e "${BLUE}🐍 Testing Python SDK...${NC}"
cd sdks/python
if command -v python3 &> /dev/null && [ -d "tests" ]; then
    if python3 -m pytest tests/ -v; then
        echo -e "${GREEN}✅ Python SDK tests passed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ Python SDK tests failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${YELLOW}⚠️  Python or tests not found, skipping Python SDK tests${NC}"
fi
cd ../..

# Test Node.js SDK
echo -e "${BLUE}📦 Testing Node.js SDK...${NC}"
cd sdks/nodejs
if command -v npm &> /dev/null && [ -d "test" ]; then
    if npm test; then
        echo -e "${GREEN}✅ Node.js SDK tests passed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ Node.js SDK tests failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${YELLOW}⚠️  npm or tests not found, skipping Node.js SDK tests${NC}"
fi
cd ../..

# Test Java SDK
echo -e "${BLUE}☕ Testing Java SDK...${NC}"
cd sdks/java
if command -v mvn &> /dev/null; then
    if mvn test -q; then
        echo -e "${GREEN}✅ Java SDK tests passed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ Java SDK tests failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${YELLOW}⚠️  Maven not found, skipping Java SDK tests${NC}"
fi
cd ../..

# Test Go SDK
echo -e "${BLUE}🐹 Testing Go SDK...${NC}"
cd sdks/go
if command -v go &> /dev/null; then
    if go test ./...; then
        echo -e "${GREEN}✅ Go SDK tests passed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ Go SDK tests failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${YELLOW}⚠️  Go not found, skipping Go SDK tests${NC}"
fi
cd ../..

# Test Rust SDK
echo -e "${BLUE}🦀 Testing Rust SDK...${NC}"
cd sdks/rust
if command -v cargo &> /dev/null; then
    if cargo test; then
        echo -e "${GREEN}✅ Rust SDK tests passed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ Rust SDK tests failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${YELLOW}⚠️  Cargo not found, skipping Rust SDK tests${NC}"
fi
cd ../..

# Test Examples
echo -e "${BLUE}📚 Testing Examples...${NC}"
if [ -d "examples" ]; then
    # Test Python examples
    if command -v python3 &> /dev/null; then
        echo "Testing Python examples..."
        for example in examples/python/*.py; do
            if [ -f "$example" ]; then
                if python3 "$example"; then
                    echo -e "${GREEN}✅ $(basename $example) passed${NC}"
                else
                    echo -e "${RED}❌ $(basename $example) failed${NC}"
                fi
            fi
        done
    fi
    
    # Test Node.js examples
    if command -v node &> /dev/null; then
        echo "Testing Node.js examples..."
        for example in examples/nodejs/*.js; do
            if [ -f "$example" ]; then
                if node "$example"; then
                    echo -e "${GREEN}✅ $(basename $example) passed${NC}"
                else
                    echo -e "${RED}❌ $(basename $example) failed${NC}"
                fi
            fi
        done
    fi
fi

echo ""
echo -e "${BLUE}📊 Test Summary:${NC}"
echo "================"
echo "Total SDK tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi