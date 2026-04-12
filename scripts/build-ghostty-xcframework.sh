#!/bin/bash
set -euo pipefail
# Build GhosttyKit.xcframework from vendor/ghostty
# Requires: zig 0.15+, Metal Toolchain

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

if ! command -v zig &> /dev/null; then
    echo "ERROR: zig is not installed. Run: brew install zig"
    exit 1
fi

if ! xcrun -sdk macosx metal --version &> /dev/null 2>&1; then
    echo "ERROR: Metal Toolchain not installed."
    echo "Run: xcodebuild -downloadComponent MetalToolchain"
    exit 1
fi

cd "$PROJECT_ROOT/vendor/ghostty"
zig build -Doptimize=ReleaseFast

mkdir -p "$PROJECT_ROOT/app/Frameworks"
cp -R zig-out/frameworks/GhosttyKit.xcframework "$PROJECT_ROOT/app/Frameworks/"
echo "GhosttyKit.xcframework built successfully"
