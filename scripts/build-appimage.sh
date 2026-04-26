#!/usr/bin/env bash
# AppImage build script for moai-studio (Linux x86_64)
# Requires: linuxdeploy (downloaded automatically)
# Usage: bash scripts/build-appimage.sh
# Output: moai-studio-{VERSION}-x86_64.AppImage

set -euo pipefail

BINARY="target/x86_64-unknown-linux-gnu/release/moai-studio"
DESKTOP="assets/moai-studio.desktop"
ICON="assets/icons/256x256/moai-studio.png"
LINUXDEPLOY_URL="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"

# Resolve VERSION from Cargo.toml (workspace package version)
if command -v cargo >/dev/null 2>&1; then
    VERSION=$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
        | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; \
          print(next(p['version'] for p in pkgs if p['name']=='moai-studio-app'))" \
        2>/dev/null || grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
else
    VERSION=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
fi

echo "Building AppImage for moai-studio v${VERSION}"

# Verify prerequisites
if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    echo "Run: cargo build --release --target x86_64-unknown-linux-gnu -p moai-studio-app"
    exit 1
fi

if [ ! -f "$DESKTOP" ]; then
    echo "ERROR: Desktop file not found at $DESKTOP"
    exit 1
fi

if [ ! -f "$ICON" ]; then
    echo "ERROR: Icon not found at $ICON"
    exit 1
fi

# Download linuxdeploy if not present
if [ ! -f "linuxdeploy-x86_64.AppImage" ]; then
    echo "Downloading linuxdeploy..."
    curl -fsSL "$LINUXDEPLOY_URL" -o linuxdeploy-x86_64.AppImage
    chmod +x linuxdeploy-x86_64.AppImage
fi

# Clean previous AppDir
rm -rf AppDir

# Set AppImage output filename
OUTPUT_NAME="moai-studio-${VERSION}-x86_64.AppImage"

# Build AppImage using --appimage-extract-and-run to avoid FUSE requirement on CI
./linuxdeploy-x86_64.AppImage \
    --appimage-extract-and-run \
    --appdir AppDir \
    --executable "$BINARY" \
    --desktop-file "$DESKTOP" \
    --icon-file "$ICON" \
    --output appimage

# Rename to versioned filename if linuxdeploy used generic name
if [ -f "moai-studio-x86_64.AppImage" ] && [ "$OUTPUT_NAME" != "moai-studio-x86_64.AppImage" ]; then
    mv "moai-studio-x86_64.AppImage" "$OUTPUT_NAME"
fi

echo "AppImage built: $OUTPUT_NAME"
ls -la "$OUTPUT_NAME"
