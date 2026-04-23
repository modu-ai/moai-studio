#!/bin/sh
# check-history-sha.sh — AC-T-11 (ii) HISTORY SHA 검증
#
# libghostty-rs rev= 라인이 변경되는 PR 에서 실행.
# spec.md HISTORY 섹션에 이전 SHA와 신규 SHA가 모두 포함되어 있는지 확인.
# 누락 시 exit 1 로 merge 차단.
#
# 의존 도구: git, grep, sed (CI 러너 기본 설치)

set -euo pipefail

CARGO_TOML="crates/moai-studio-terminal/Cargo.toml"
SPEC_MD=".moai/specs/SPEC-V3-002/spec.md"

# ------------------------------------------------------------------
# 현재 브랜치 Cargo.toml 에서 신규 SHA 추출
# ------------------------------------------------------------------
NEW_SHA=$(grep 'rev = ' "$CARGO_TOML" | sed 's/.*rev = "\([^"]*\)".*/\1/')

if [ -z "$NEW_SHA" ]; then
  echo "ERROR: $CARGO_TOML 에서 rev = 라인을 파싱할 수 없습니다." >&2
  exit 1
fi

# ------------------------------------------------------------------
# origin/main 브랜치 Cargo.toml 에서 이전 SHA 추출
# ------------------------------------------------------------------
OLD_SHA=$(git show origin/main:"$CARGO_TOML" 2>/dev/null | grep 'rev = ' | sed 's/.*rev = "\([^"]*\)".*/\1/' || true)

if [ -z "$OLD_SHA" ]; then
  echo "ERROR: origin/main:$CARGO_TOML 에서 rev = 라인을 파싱할 수 없습니다." >&2
  echo "       (origin/main 에 $CARGO_TOML 이 없거나 rev = 라인이 없는 경우)" >&2
  exit 1
fi

echo "OLD SHA: $OLD_SHA"
echo "NEW SHA: $NEW_SHA"

# 같은 SHA 로 PR 이 왔을 경우 조기 종료 (변경 없음)
if [ "$OLD_SHA" = "$NEW_SHA" ]; then
  echo "INFO: rev = 변경 없음. SHA 검증 생략."
  exit 0
fi

# ------------------------------------------------------------------
# spec.md 에 두 SHA 모두 포함되어 있는지 확인
# ------------------------------------------------------------------
MISSING=0

if ! grep -q "$OLD_SHA" "$SPEC_MD"; then
  echo "ERROR: $SPEC_MD HISTORY 에 이전 SHA 가 없습니다: $OLD_SHA" >&2
  MISSING=1
fi

if ! grep -q "$NEW_SHA" "$SPEC_MD"; then
  echo "ERROR: $SPEC_MD HISTORY 에 신규 SHA 가 없습니다: $NEW_SHA" >&2
  MISSING=1
fi

if [ "$MISSING" -eq 1 ]; then
  echo "" >&2
  echo "ERROR: pin bump PR 은 spec.md HISTORY 섹션에 이전/신규 SHA 를 모두 기록해야 합니다." >&2
  echo "       pin 정책 요건 (c): HISTORY 에 이전/신규 SHA 기록" >&2
  exit 1
fi

echo "OK: $SPEC_MD HISTORY 에 OLD SHA와 NEW SHA가 모두 포함되어 있습니다."
