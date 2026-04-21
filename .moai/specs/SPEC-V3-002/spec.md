# SPEC-V3-002: Terminal Core — libghostty-vt + PTY + Shell 통합 (stub)

---
id: SPEC-V3-002
version: 0.1.0-stub
status: draft (placeholder — full EARS SPEC via /moai plan SPEC-V3-002 예정)
created: 2026-04-21
updated: 2026-04-21
author: MoAI (SPEC-V3-001 RG-V3-3 rescope)
priority: High (Phase 2 Terminal Core)
issue_number: 0
depends_on: SPEC-V3-001
---

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 0.1.0-stub | 2026-04-21 | SPEC-V3-001 RG-V3-3 rescope stub. Full SPEC 은 /moai plan SPEC-V3-002 로 작성 예정 |

---

## 1. 개요

SPEC-V3-001 RG-V3-3 (libghostty-vt 스파이크) 가 재진단 결과 **"Metal blocker"가 아닌 작업 미시작 상태** 임이 확인되어, 터미널 통합 자체를 별도 SPEC 으로 분리.

원 SPEC-V3-001 §9 Exclusions 에 명시된 "Phase 2" Terminal Core 작업과 통합하여 본 SPEC 에서 일괄 다룬다.

**rescope 근거** (SPEC-V3-001/progress.md 참조):
- Metal toolchain 환경 ✅ 작동 확인 (`xcrun -sdk macosx metal`, cryptex MobileAsset v17.5)
- Zig 0.15.2 ✅ 설치 확인
- libghostty-rs 의존성 ❌ 미추가 (TODO 주석 상태)
- pinned commit ❌ 미결정 (alpha 상태)
- 실제 작업 규모: FFI wrapping + portable-pty + shell spawn + GPUI 텍스트 렌더 = 독립 SPEC 가치

---

## 2. 예비 범위 (full SPEC 작성 전 스케치)

### 요구사항 후보

- **RG-V3-002-1**: libghostty-rs pinned commit 선정 + `moai-studio-terminal` crate Cargo.toml 통합
- **RG-V3-002-2**: portable-pty 0.9+ 연동 + `$SHELL` spawn + 표준 입출력 바인딩
- **RG-V3-002-3**: GPUI 윈도우 내부 terminal surface (SPEC-V3-001 RootView 확장) + 최소 텍스트 렌더
- **RG-V3-002-4**: Zig 0.15.x CI 매트릭스 확장 (macOS + Linux runner 에 setup-zig action)
- **RG-V3-002-5**: Zig 미설치 환경 명확한 에러 메시지 (SPEC-V3-001 원 AC-3.2 계승)

### 예비 Acceptance Criteria

| AC | 내용 | 비고 |
|----|------|------|
| AC-T-1 | `cargo run --example ghostty-spike` 가 `$SHELL` prompt 렌더 | SPEC-V3-001 원 AC-3.1 계승 |
| AC-T-2 | Zig 미설치 시 명확한 빌드 에러 + exit 1 | SPEC-V3-001 원 AC-3.2 계승 |
| AC-T-3 | CI matrix (macOS + Linux) Zig 설치 + build 통과 | 신규 |

---

## 3. 전제 의존성

- **SPEC-V3-001 완료**: GPUI 윈도우 + RootView (✅ Phase 1.8)
- **SPEC-V3-001 CI**: GitHub Actions matrix (✅ Phase 1.9, run 24708460052)
- **libghostty-rs upstream**: https://github.com/Uzaaft/libghostty-rs — pinned commit 조사 필요
- **portable-pty**: crates.io 최신 (2024-10 기준 0.9.x)
- **Zig**: 0.15.x (CI: `mlugg/setup-zig@v1` action)

---

## 4. 다음 단계

1. 사용자 승인 후 `/moai plan SPEC-V3-002` 실행
2. manager-spec 에이전트가 EARS-format 전체 SPEC 작성
3. research.md 에서 libghostty-rs latest commit + crush 사례 검증
4. Annotation cycle 거쳐 Plan 확정
5. `/moai run SPEC-V3-002` → 실제 Terminal Core 구현

---

## 5. 참조

- SPEC-V3-001 § RG-V3-3: 원 요구사항
- SPEC-V3-001 progress.md § 5: rescope 근거
- `.moai/design/master-plan.md` § Phase 2: Terminal Core 설계 방향
- `crates/moai-studio-terminal/Cargo.toml` 내 TODO 주석: 시작점
- [Uzaaft/libghostty-rs](https://github.com/Uzaaft/libghostty-rs)
- [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty)

---

Version: 0.1.0-stub
상태: Stub placeholder — full SPEC 작성 보류 (사용자 승인 + /moai plan 필요)
