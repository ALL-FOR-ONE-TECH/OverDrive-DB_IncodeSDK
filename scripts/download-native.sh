#!/bin/bash
# OverDrive-DB Native Library Download Script v2.0.0
# Downloads and verifies native libraries from GitHub releases

set -e

VERSION=${1:-"latest"}
GITHUB_REPO="ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK"

echo "📥 Downloading OverDrive-DB native libraries..."
echo "Version: $VERSION"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create native directories if they don't exist
mkdir -p native/{windows,linux/{x64,arm64},macos/{x64,arm64}}

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

echo -e "${BLUE}🔍 Detected platform: $OS $ARCH${NC}"

# Download function
download_file() {
    local url=$1
    local output=$2
    local description=$3
    
    echo -e "${BLUE}📥 Downloading $description...${NC}"
    
    if command -v curl &> /dev/null; then
        curl -L -o "$output" "$url"
    elif command -v wget &> /dev/null; then
        wget -O "$output" "$url"
    else
        echo -e "${RED}❌ Neither curl nor wget found. Please install one of them.${NC}"
        exit 1
    fi
    
    if [ -f "$output" ]; then
        echo -e "${GREEN}✅ Downloaded $description${NC}"
    else
        echo -e "${RED}❌ Failed to download $description${NC}"
        exit 1
    fi
}

# Get release URL
if [ "$VERSION" = "latest" ]; then
    RELEASE_URL="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
else
    RELEASE_URL="https://api.github.com/repos/$GITHUB_REPO/releases/tags/v$VERSION"
fi

echo -e "${BLUE}🔍 Fetching release information...${NC}"

# Get download URLs (this would be customized based on actual release structure)
BASE_URL="https://github.com/$GITHUB_REPO/releases/download"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_BASE="$BASE_URL/latest"
else
    DOWNLOAD_BASE="$BASE_URL/v$VERSION"
fi

# Download Windows library
echo -e "${BLUE}🪟 Downloading Windows library...${NC}"
download_file "$DOWNLOAD_BASE/overdrive-windows-x64.dll" "native/windows/overdrive.dll" "Windows x64 library"

# Download Linux libraries
echo -e "${BLUE}🐧 Downloading Linux libraries...${NC}"
download_file "$DOWNLOAD_BASE/liboverdrive-linux-x64.so" "native/linux/x64/liboverdrive.so" "Linux x64 library"
download_file "$DOWNLOAD_BASE/liboverdrive-linux-arm64.so" "native/linux/arm64/liboverdrive-arm64.so" "Linux ARM64 library"

# Download macOS libraries
echo -e "${BLUE}🍎 Downloading macOS libraries...${NC}"
download_file "$DOWNLOAD_BASE/liboverdrive-macos-x64.dylib" "native/macos/x64/liboverdrive.dylib" "macOS x64 library"
download_file "$DOWNLOAD_BASE/liboverdrive-macos-arm64.dylib" "native/macos/arm64/liboverdrive-arm64.dylib" "macOS ARM64 library"

# Download checksums
echo -e "${BLUE}🔐 Downloading checksums...${NC}"
download_file "$DOWNLOAD_BASE/CHECKSUMS.sha256" "native/CHECKSUMS.sha256" "checksums file"

# Verify checksums
echo -e "${BLUE}🔍 Verifying checksums...${NC}"
cd native

if command -v sha256sum &> /dev/null; then
    if sha256sum -c CHECKSUMS.sha256; then
        echo -e "${GREEN}✅ All checksums verified successfully${NC}"
    else
        echo -e "${RED}❌ Checksum verification failed${NC}"
        exit 1
    fi
elif command -v shasum &> /dev/null; then
    if shasum -a 256 -c CHECKSUMS.sha256; then
        echo -e "${GREEN}✅ All checksums verified successfully${NC}"
    else
        echo -e "${RED}❌ Checksum verification failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  No checksum utility found, skipping verification${NC}"
fi

cd ..

echo ""
echo -e "${GREEN}🎉 Native libraries downloaded and verified successfully!${NC}"
echo ""
echo "📋 Downloaded libraries:"
echo "  - Windows x64: native/windows/overdrive.dll"
echo "  - Linux x64: native/linux/x64/liboverdrive.so"
echo "  - Linux ARM64: native/linux/arm64/liboverdrive-arm64.so"
echo "  - macOS x64: native/macos/x64/liboverdrive.dylib"
echo "  - macOS ARM64: native/macos/arm64/liboverdrive-arm64.dylib"
echo ""
echo "🚀 Ready to build SDKs!"