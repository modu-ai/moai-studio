#!/bin/bash
# build-ghostty-xcframework.sh — Ghostty 1.3.0+ 의 ghostty-vt.xcframework 빌드
# SPEC: SPEC-M1-001 / Task: T-019
#
# 사전 조건:
#   - scripts/check-metal-toolchain.sh 통과 (자동 실행)
#
# Ghostty 소스 위치 (우선순위):
#   1. 환경변수 $GHOSTTY_SRC (절대경로)
#   2. vendor/ghostty (submodule — git submodule update --init 필요)
#   3. ../ghostty (인접 디렉토리 fallback, 개발자 로컬 편의)
#
# 출력: app/Frameworks/ghostty-vt.xcframework (overwrite)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 1) 툴체인 사전 검증
echo ">> Step 1: 툴체인 검증"
if ! "$SCRIPT_DIR/check-metal-toolchain.sh"; then
    echo "ERROR: 툴체인 검증 실패. 빌드를 중단합니다."
    exit 1
fi
echo

# 2) Ghostty 소스 위치 결정
echo ">> Step 2: Ghostty 소스 위치 확인"
GHOSTTY_DIR=""
if [ -n "${GHOSTTY_SRC:-}" ]; then
    GHOSTTY_DIR="$GHOSTTY_SRC"
    echo "   (env GHOSTTY_SRC 사용: $GHOSTTY_DIR)"
elif [ -d "$PROJECT_ROOT/vendor/ghostty/.git" ] || [ -f "$PROJECT_ROOT/vendor/ghostty/build.zig" ]; then
    GHOSTTY_DIR="$PROJECT_ROOT/vendor/ghostty"
    echo "   (submodule 사용: $GHOSTTY_DIR)"
elif [ -f "$PROJECT_ROOT/../ghostty/build.zig" ]; then
    GHOSTTY_DIR="$(cd "$PROJECT_ROOT/../ghostty" && pwd)"
    echo "   (인접 디렉토리 사용: $GHOSTTY_DIR)"
else
    echo "ERROR: Ghostty 소스를 찾을 수 없습니다."
    echo "다음 중 하나를 수행하세요:"
    echo "  A) git submodule add https://github.com/ghostty-org/ghostty vendor/ghostty"
    echo "  B) export GHOSTTY_SRC=/path/to/ghostty"
    echo "  C) git clone https://github.com/ghostty-org/ghostty ../ghostty"
    exit 1
fi

if [ ! -f "$GHOSTTY_DIR/build.zig" ]; then
    echo "ERROR: $GHOSTTY_DIR 에 build.zig 가 없습니다. 손상된 소스입니다."
    exit 1
fi

# 3) zig build
echo
echo ">> Step 3: zig build -Doptimize=ReleaseFast (수 분 소요 가능)"
BUILD_START="$(date +%s)"
cd "$GHOSTTY_DIR"
zig build -Doptimize=ReleaseFast
BUILD_END="$(date +%s)"
ELAPSED="$((BUILD_END - BUILD_START))"

XCFW_SRC="$GHOSTTY_DIR/zig-out/lib/ghostty-vt.xcframework"
if [ ! -d "$XCFW_SRC" ]; then
    echo "ERROR: 기대한 xcframework 가 생성되지 않았습니다: $XCFW_SRC"
    echo "zig build 출력을 확인하세요."
    exit 1
fi

# 4) 결과 복사
echo
echo ">> Step 4: app/Frameworks/ 로 복사"
DEST_DIR="$PROJECT_ROOT/app/Frameworks"
mkdir -p "$DEST_DIR"
rm -rf "$DEST_DIR/ghostty-vt.xcframework"
cp -R "$XCFW_SRC" "$DEST_DIR/"

SIZE="$(du -sh "$DEST_DIR/ghostty-vt.xcframework" | cut -f1)"
echo
echo "============================================================"
echo "OK: ghostty-vt.xcframework 빌드 성공"
echo "   경로: $DEST_DIR/ghostty-vt.xcframework"
echo "   크기: $SIZE"
echo "   빌드 시간: ${ELAPSED}s"
echo "============================================================"
exit 0
