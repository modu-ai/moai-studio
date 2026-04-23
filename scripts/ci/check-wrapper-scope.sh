#!/bin/sh
# check-wrapper-scope.sh — AC-T-11 (iii) wrapper 외 파일 변경 감지
#
# libghostty-rs rev= 라인이 변경되는 PR 에서 실행.
# pin bump PR 에서 허용 파일(libghostty_ffi.rs, Cargo.toml, Cargo.lock) 외
# 다른 파일이 변경된 경우 annotation-cycle-required 라벨을 부착하고 경고 출력.
# 항상 exit 0 (비차단 — 경고만).
#
# 환경변수:
#   GH_TOKEN   — 설정된 경우 gh 로 PR 라벨 부착
#   PR_NUMBER  — 라벨 부착 대상 PR 번호
#
# 의존 도구: git, gh (선택적)

set -euo pipefail

# ------------------------------------------------------------------
# PR diff 파일 목록 (origin/main 대비 전체 변경 파일)
# ------------------------------------------------------------------
CHANGED_FILES=$(git diff --name-only origin/main...HEAD 2>/dev/null || true)

if [ -z "$CHANGED_FILES" ]; then
  echo "INFO: 변경된 파일이 없습니다."
  exit 0
fi

# ------------------------------------------------------------------
# 허용 파일 제외: libghostty_ffi.rs, Cargo.toml (모든 경로), Cargo.lock
# 이 파일들 외에 다른 파일이 변경되면 annotation cycle 필요
# ------------------------------------------------------------------
OTHER_FILES=""
for f in $CHANGED_FILES; do
  case "$f" in
    "crates/moai-studio-terminal/src/libghostty_ffi.rs")
      # FFI wrapper — pin bump 시 API 변경 허용
      ;;
    *"Cargo.toml" | "Cargo.lock")
      # 의존성 파일 — pin bump 시 정상 변경
      ;;
    *)
      OTHER_FILES="${OTHER_FILES:+$OTHER_FILES }$f"
      ;;
  esac
done

# ------------------------------------------------------------------
# 허용 파일 외 변경 없으면 정상 종료
# ------------------------------------------------------------------
if [ -z "$OTHER_FILES" ]; then
  echo "OK: pin bump 허용 파일만 변경되었습니다. annotation cycle 불필요."
  exit 0
fi

# ------------------------------------------------------------------
# 경고 출력 및 라벨 부착 (비차단)
# ------------------------------------------------------------------
echo "WARNING: libghostty_ffi.rs 외 파일이 변경되었습니다." >&2
echo "         annotation cycle 재개가 필요할 수 있습니다 (pin 정책 요건 d)." >&2
echo "" >&2
echo "변경된 파일 (허용 파일 제외):" >&2
for f in $OTHER_FILES; do
  echo "  - $f" >&2
done

# GH_TOKEN 과 PR_NUMBER 가 설정된 경우 annotation-cycle-required 라벨 부착
if [ -n "${GH_TOKEN:-}" ] && [ -n "${PR_NUMBER:-}" ]; then
  echo ""
  echo "annotation-cycle-required 라벨 부착 중 (PR #${PR_NUMBER})..."
  if gh pr edit "${PR_NUMBER}" --add-label "annotation-cycle-required" 2>/dev/null; then
    echo "OK: annotation-cycle-required 라벨이 부착되었습니다."
  else
    echo "WARNING: 라벨 부착 실패 (비차단 — 수동으로 확인하세요)." >&2
  fi
else
  echo "INFO: GH_TOKEN 또는 PR_NUMBER 미설정 — 라벨 부착 생략."
fi

# 항상 성공 (비차단 경고)
exit 0
