#!/bin/bash
#
# BoxMux Multi-Platform Build Script
# Builds release binaries for all supported platforms
#

set -e

echo "🚀 BoxMux Multi-Platform Build Script"
echo "======================================"

# Build targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if rust targets are installed
echo "🔍 Checking installed Rust targets..."
for target in "${TARGETS[@]}"; do
    if rustup target list --installed | grep -q "$target"; then
        echo -e "  ${GREEN}✓${NC} $target"
    else
        echo -e "  ${YELLOW}+${NC} Installing $target..."
        rustup target add "$target"
    fi
done

# Clean previous builds
echo ""
echo "🧹 Cleaning previous builds..."
cargo clean

# Create output directory
mkdir -p dist

# Build each target
echo ""
echo "🔨 Building release binaries..."
for target in "${TARGETS[@]}"; do
    echo -e "\n${BLUE}Building for $target...${NC}"
    
    # Build the target
    if cargo build --release --target "$target"; then
        echo -e "${GREEN}✓ Build successful for $target${NC}"
        
        # Copy binary to dist directory
        if [[ "$target" == *"windows"* ]]; then
            BINARY_NAME="boxmux.exe"
            OUTPUT_NAME="boxmux-${target}.exe"
        else
            BINARY_NAME="boxmux"
            OUTPUT_NAME="boxmux-${target}"
        fi
        
        cp "target/$target/release/$BINARY_NAME" "dist/$OUTPUT_NAME"
        echo -e "  ${GREEN}→${NC} Binary saved as dist/$OUTPUT_NAME"
        
        # Get binary size
        BINARY_SIZE=$(stat -f%z "dist/$OUTPUT_NAME" 2>/dev/null || stat -c%s "dist/$OUTPUT_NAME")
        BINARY_SIZE_MB=$((BINARY_SIZE / 1024 / 1024))
        echo -e "  ${BLUE}→${NC} Size: ${BINARY_SIZE_MB}MB"
        
    else
        echo -e "${RED}✗ Build failed for $target${NC}"
        exit 1
    fi
done

# Generate checksums
echo ""
echo "🔐 Generating checksums..."
cd dist
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum boxmux-* > checksums.txt
elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 boxmux-* > checksums.txt
else
    echo -e "${YELLOW}⚠ Warning: No checksum utility found${NC}"
fi
cd ..

# List generated files
echo ""
echo "📦 Generated release artifacts:"
echo "==============================="
ls -la dist/
echo ""

# Success message
echo -e "${GREEN}🎉 Multi-platform build completed successfully!${NC}"
echo -e "${BLUE}📁 All binaries are available in the 'dist/' directory${NC}"
echo ""
echo "To test a binary:"
echo "  ./dist/boxmux-x86_64-unknown-linux-gnu --version"
echo ""
echo "To create a release:"
echo "  gh release create v0.1.0 dist/* --title 'BoxMux v0.1.0' --notes 'Release notes here'"