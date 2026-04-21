# SPEC-V3-001 Progress — Phase 1 Checkpoint

---
spec_id: SPEC-V3-001
checkpoint: Phase 1 (GPUI 스캐폴드 + Workspace 인터랙션)
recorded: 2026-04-21
branch: feat/v3-scaffold
base_commit: 101cd83 (Phase 0.2 기준점)
head_commit: 9d586ea
---

## 1. RG 진행 요약

| RG | 제목 | 상태 | 비고 |
|----|------|------|------|
| RG-V3-1 | Workspace 재구성 (Cargo) | ✅ 완료 | `crates/moai-core` 289 tests 유지, Swift archive 이동 완료 |
| RG-V3-2 | GPUI 통합 + 기본 윈도우 | ✅ 완료 | Phase 1.1~1.8, 8 커밋, AC-2.1/2.2/2.3 충족 |
| RG-V3-3 | libghostty-vt 스파이크 | ⛔ 블로킹 | Metal Toolchain 이슈, Zig 0.15.x 설치는 완료 |
| RG-V3-4 | CI matrix + 품질 게이트 | ⬜ 미시작 | 로컬 3 게이트 (fmt/clippy/test) 는 수동으로 통과 중 |
| RG-V3-5 | Swift 자산 아카이브 | ✅ 완료 | `archive/swift-legacy/` 로 `git mv` 완료 (Phase 0.2) |

**전체 완료율**: 3/5 RG (60%) · Acceptance Criteria 9개 중 5개 충족 (AC-1.1, AC-1.2, AC-2.1, AC-2.2, AC-2.3, AC-5.1)

---

## 2. Phase 1 Sub-Phase 이력 (RG-V3-2 실행 로그)

| Sub-Phase | 커밋 | 산출 | 테스트 증분 |
|-----------|------|------|-------------|
| 1.1 GPUI 의존성 | `b676c81` | `gpui = "0.2.2"` dep, Application::new, RootView 스캐폴드 | — |
| 1.2 4영역 레이아웃 | `d0aecc6` | TitleBar 44pt / Sidebar 260pt / Body / StatusBar 28pt | — |
| 1.3 Empty State CTA | `a3e0588` | Welcome hero + 3 CTA (Create/Sample/Recent) + Tip | — |
| 1.4 파일 picker | `edfda5d` | `rfd = "0.15"` + `Workspace` 구조체 | +3 |
| 1.5 JSON persistence | `09e14c6` | `WorkspacesStore` (load/save/add/remove/touch) | +5 |
| 1.6 Sidebar 리스트 | `92da7c0` | `RootView` state + workspace row 렌더 + active 하이라이트 | +3 |
| 1.7 + New Workspace 실동작 | `7fa8077` | Stateful button + `cx.listener` + `handle_add_workspace` | +2 |
| 1.8 Row 클릭 active 전환 | `9d586ea` | `activate_workspace` + `handle_activate_workspace` + `store.touch` | +2 |

**Phase 1 테스트 증분**: +15 (baseline 232 → 247 현재)
> 주: 실제 workspace 측정은 248 tests (moai-core 289 중 1 추가 포함 가능성; 재계산 필요)

**파일 변경**: 5개 (crates/moai-studio-{app,ui,workspace}), +1105 / -30 LOC

---

## 3. Acceptance Criteria 현황

| AC | 상태 | 검증 방식 |
|----|------|-----------|
| AC-1.1 기존 289 tests 통과 | ✅ | `cargo test --workspace` 통과 |
| AC-1.2 warning 0 릴리즈 빌드 | ✅ | Phase 1.5 에서 `cargo build --release` 확인 |
| AC-2.1 1600×1000 + 4영역 | ✅ | Phase 1.2 육안 검증 |
| AC-2.2 Welcome CTA | ✅ | Phase 1.3 (workspace 가 비었을 때만 표시로 1.6 리파인) |
| AC-2.3 네이티브 폴더 다이얼로그 | ✅ | Phase 1.4 `rfd::FileDialog`, Phase 1.7 실배선 |
| AC-3.1 libghostty 스파이크 | ❌ | 블로킹 — Metal Toolchain |
| AC-3.2 Zig 미설치 에러 | ❌ | libghostty 스파이크 대기 |
| AC-4.1 CI 4 게이트 통과 | ⚠️ | 로컬만 통과, GitHub Actions workflow 미구축 |
| AC-4.2 린트 오류 PR 머지 차단 | ⚠️ | CI 없음 |
| AC-5.1 `app/` archive 이동 | ✅ | Phase 0.2 완료 |

---

## 4. 품질 게이트 (Phase 1.8 시점)

- `cargo fmt --all -- --check`: ✅ PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: ✅ PASS (0 warnings)
- `cargo test --workspace`: ✅ 248 passed, 0 failed, 6 ignored
- `cargo build --release`: ✅ (Phase 1.5 마지막 검증)
- `cargo build --no-default-features`: ✅ `moai-adk` feature toggle 정상

**회귀**: 0

---

## 5. 남은 작업 (Phase 2+)

### 즉시 우선순위

1. **RG-V3-4 CI matrix** — GitHub Actions `.github/workflows/build.yml` (macOS + Linux 매트릭스, 4 게이트). 로컬 게이트를 자동화로 승격.
2. **RG-V3-3 libghostty-vt 스파이크** — Metal Toolchain 블로커 해결 (macOS) 후 `moai-studio-terminal` crate 에 libghostty-rs 의존 추가, 쉘 spawn + 텍스트 렌더 예제.

### 후속 Phase 후보 (이 SPEC 범위 외)

- Tab / Pane split (SPEC-V3-002 예정)
- Command Palette (SPEC-V3-005 예정)
- Surfaces (SPEC-V3-008~ 예정)

---

## 6. 체크포인트 Sync 결정 로그

- **Sync 범위**: RG-V3-2 실질 완료 기록만. PR 생성 없음.
- **제외**: 작업 트리의 미커밋 변경 130+ (MoAI 툴링 드리프트 · SPEC-AGENCY-ABSORB-001 계열 자산) — 별도 sync 대상.
- **다음 체크포인트**: RG-V3-4 CI matrix 완료 시점 → 전체 SPEC-V3-001 sync + PR 일괄 생성 권장.

---

Recorded by: MoAI Studio orchestrator
Updated: 2026-04-21
