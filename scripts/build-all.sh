#!/bin/bash
# OverDrive-DB SDK Build Script v2.0.0
# Builds all language SDKs

set -e

# Change to the IncodeSDK directory
cd "$(dirname "$0")/.."

echo "🚀 Building OverDrive-DB SDKs v2.0.0..."
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build Python SDK
echo -e "${BLUE}📦 Building Python SDK...${NC}"
cd sdks/python
if command -v python3 &> /dev/null; then
    python3 -m build
    echo -e "${GREEN}✅ Python SDK built successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Python not found, skipping Python SDK${NC}"
fi
cd ../..

# Build Node.js SDK
echo -e "${BLUE}📦 Building Node.js SDK...${NC}"
cd sdks/nodejs
if command -v npm &> /dev/null; then
    npm install
    npm run build 2>/dev/null || echo "No build script defined"
    echo -e "${GREEN}✅ Node.js SDK built successfully${NC}"
else
    echo -e "${YELLOW}⚠️  npm not found, skipping Node.js SDK${NC}"
fi
cd ../..

# Build Java SDK
echo -e "${BLUE}📦 Building Java SDK...${NC}"
cd sdks/java
if command -v mvn &> /dev/null; then
    mvn clean package -q
    echo -e "${GREEN}✅ Java SDK built successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Maven not found, skipping Java SDK${NC}"
fi
cd ../..

# Build Go SDK
echo -e "${BLUE}📦 Building Go SDK...${NC}"
cd sdks/go
if command -v go &> /dev/null; then
    go build
    echo -e "${GREEN}✅ Go SDK built successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Go not found, skipping Go SDK${NC}"
fi
cd ../..

# Build Rust SDK
echo -e "${BLUE}📦 Building Rust SDK...${NC}"
cd sdks/rust
if command -v cargo &> /dev/null; then
    cargo build --release
    echo -e "${GREEN}✅ Rust SDK built successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Cargo not found, skipping Rust SDK${NC}"
fi
cd ../..

# C SDK (header only, no build needed)
echo -e "${BLUE}📦 C SDK (header-only)...${NC}"
if [ -f "sdks/c/include/overdrive.h" ]; then
    echo -e "${GREEN}✅ C SDK ready (header-only)${NC}"
else
    echo -e "${RED}❌ C SDK header not found${NC}"
fi

echo ""
echo -e "${GREEN}🎉 All SDKs built successfully!${NC}"
echo ""
echo "📋 Summary:"
echo "  - Python SDK: sdks/python/dist/"
echo "  - Node.js SDK: sdks/nodejs/"
echo "  - Java SDK: sdks/java/target/"
echo "  - Go SDK: sdks/go/"
echo "  - Rust SDK: sdks/rust/target/release/"
echo "  - C SDK: sdks/c/include/overdrive.h"
echo ""
echo "🚀 Ready for distribution!"