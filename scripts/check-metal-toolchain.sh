#!/bin/bash
# check-metal-toolchain.sh — GhosttyKit 빌드를 위한 macOS 툴체인 사전 검증
# SPEC: SPEC-M1-001 / Task: T-019
# 검증 항목 (fatal):
#   1. Xcode 15+ (xcodebuild -version)
#   2. Metal Toolchain (xcrun metal --version)
#   3. zig 0.15.x (zig version)
# 검증 항목 (warning only):
#   4. xcodegen (T-020 에서 사용)
# 실패 시: 설치 가이드를 출력하고 exit 1

set -euo pipefail

RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
BOLD=$'\033[1m'
RESET=$'\033[0m'

FAIL=0

echo "${BOLD}=== GhosttyKit 툴체인 사전 검증 ===${RESET}"
echo

# --- 1. Xcode ---
echo -n "[1/4] Xcode 15+ ... "
if ! command -v xcodebuild >/dev/null 2>&1; then
    echo "${RED}FAIL${RESET}"
    echo "  xcodebuild 를 찾을 수 없습니다."
    echo "  설치: Mac App Store 에서 Xcode 설치 후 'sudo xcode-select -s /Applications/Xcode.app' 실행"
    FAIL=1
else
    XC_VER_LINE="$(xcodebuild -version 2>/dev/null | head -n1)"
    XC_MAJOR="$(echo "$XC_VER_LINE" | awk '{print $2}' | cut -d. -f1)"
    if [ -z "${XC_MAJOR:-}" ] || [ "$XC_MAJOR" -lt 15 ] 2>/dev/null; then
        echo "${RED}FAIL${RESET} (detected: $XC_VER_LINE)"
        echo "  Xcode 15 이상이 필요합니다. App Store 에서 업데이트하세요."
        FAIL=1
    else
        echo "${GREEN}OK${RESET} ($XC_VER_LINE)"
    fi
fi

# --- 2. Metal Toolchain ---
echo -n "[2/4] Metal Toolchain ... "
if xcrun -sdk macosx metal --version >/dev/null 2>&1; then
    METAL_VER="$(xcrun -sdk macosx metal --version 2>&1 | head -n1)"
    echo "${GREEN}OK${RESET} ($METAL_VER)"
    echo "  Metal Toolchain: OK"
else
    echo "${RED}FAIL${RESET}"
    echo "  Metal Toolchain 이 설치되지 않았습니다."
    echo "  설치 옵션:"
    echo "    1) CLI: xcodebuild -downloadComponent MetalToolchain"
    echo "    2) GUI: Xcode > Settings > Components > Metal Toolchain"
    echo "    3) https://developer.apple.com/download/all/ 에서 Metal Developer Tools 다운로드"
    FAIL=1
fi

# --- 3. zig ---
echo -n "[3/4] zig 0.15.x ... "
if ! command -v zig >/dev/null 2>&1; then
    echo "${RED}FAIL${RESET}"
    echo "  zig 를 찾을 수 없습니다."
    echo "  설치: brew install zig"
    FAIL=1
else
    ZIG_VER="$(zig version 2>/dev/null)"
    ZIG_MAJOR="$(echo "$ZIG_VER" | cut -d. -f1)"
    ZIG_MINOR="$(echo "$ZIG_VER" | cut -d. -f2)"
    if [ "${ZIG_MAJOR:-0}" = "0" ] && [ "${ZIG_MINOR:-0}" = "15" ]; then
        echo "${GREEN}OK${RESET} ($ZIG_VER)"
    else
        echo "${RED}FAIL${RESET} (detected: $ZIG_VER, required: 0.15.x)"
        echo "  zig 0.15.x 가 필요합니다. 'brew upgrade zig' 또는 https://ziglang.org/download/ 참고"
        FAIL=1
    fi
fi

# --- 4. xcodegen (non-fatal) ---
echo -n "[4/4] xcodegen (T-020 용, optional) ... "
if command -v xcodegen >/dev/null 2>&1; then
    XG_VER="$(xcodegen --version 2>/dev/null | head -n1 || echo 'unknown')"
    echo "${GREEN}OK${RESET} ($XG_VER)"
else
    echo "${YELLOW}MISSING${RESET}"
    echo "  xcodegen 미설치 (T-020 에서 필요). 설치: brew install xcodegen"
    echo "  (non-fatal — 현재 단계는 통과 처리)"
fi

echo
if [ "$FAIL" -ne 0 ]; then
    echo "${RED}${BOLD}✗ 툴체인 검증 실패${RESET} — 위의 설치 지침을 따라 해결 후 재시도하세요."
    exit 1
fi

echo "${GREEN}${BOLD}✓ 모든 필수 툴체인 검증 통과${RESET}"
exit 0
