#!/bin/bash
# OverDrive-DB Version Sync Script v2.0.0
# Synchronizes version across all SDKs

if [ $# -eq 0 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 2.0.0"
    exit 1
fi

VERSION=$1
echo "🔄 Syncing version to $VERSION across all SDKs..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Update Python SDK
echo -e "${BLUE}🐍 Updating Python SDK version...${NC}"
if [ -f "sdks/python/pyproject.toml" ]; then
    sed -i "s/version = \".*\"/version = \"$VERSION\"/" sdks/python/pyproject.toml
    echo -e "${GREEN}✅ Updated sdks/python/pyproject.toml${NC}"
fi

# Update Node.js SDK
echo -e "${BLUE}📦 Updating Node.js SDK version...${NC}"
if [ -f "sdks/nodejs/package.json" ]; then
    sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" sdks/nodejs/package.json
    echo -e "${GREEN}✅ Updated sdks/nodejs/package.json${NC}"
fi

# Update Java SDK
echo -e "${BLUE}☕ Updating Java SDK version...${NC}"
if [ -f "sdks/java/pom.xml" ]; then
    sed -i "s/<version>.*<\/version>/<version>$VERSION<\/version>/" sdks/java/pom.xml
    echo -e "${GREEN}✅ Updated sdks/java/pom.xml${NC}"
fi

# Update Go SDK
echo -e "${BLUE}🐹 Updating Go SDK version...${NC}"
if [ -f "sdks/go/go.mod" ]; then
    # Go modules use git tags for versioning
    echo "// Version: v$VERSION" > sdks/go/version.go
    echo -e "${GREEN}✅ Updated sdks/go/version.go${NC}"
fi

# Update Rust SDK
echo -e "${BLUE}🦀 Updating Rust SDK version...${NC}"
if [ -f "sdks/rust/Cargo.toml" ]; then
    sed -i "s/version = \".*\"/version = \"$VERSION\"/" sdks/rust/Cargo.toml
    echo -e "${GREEN}✅ Updated sdks/rust/Cargo.toml${NC}"
fi

# Update root Cargo.toml
echo -e "${BLUE}📦 Updating root Cargo.toml...${NC}"
if [ -f "Cargo.toml" ]; then
    sed -i "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    echo -e "${GREEN}✅ Updated Cargo.toml${NC}"
fi

# Update README.md
echo -e "${BLUE}📖 Updating README.md...${NC}"
if [ -f "README.md" ]; then
    sed -i "s/v[0-9]\+\.[0-9]\+\.[0-9]\+/v$VERSION/g" README.md
    echo -e "${GREEN}✅ Updated README.md${NC}"
fi

# Update CHANGELOG.md
echo -e "${BLUE}📝 Updating CHANGELOG.md...${NC}"
if [ -f "CHANGELOG.md" ]; then
    # Add new version entry at the top
    DATE=$(date +%Y-%m-%d)
    sed -i "1i\\## v$VERSION ($DATE)\\n" CHANGELOG.md
    echo -e "${GREEN}✅ Updated CHANGELOG.md${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Version sync complete!${NC}"
echo "All SDKs now use version: $VERSION"
echo ""
echo "📋 Updated files:"
echo "  - sdks/python/pyproject.toml"
echo "  - sdks/nodejs/package.json"
echo "  - sdks/java/pom.xml"
echo "  - sdks/go/version.go"
echo "  - sdks/rust/Cargo.toml"
echo "  - Cargo.toml"
echo "  - README.md"
echo "  - CHANGELOG.md"
echo ""
echo "🚀 Ready to commit and tag: git tag v$VERSION"