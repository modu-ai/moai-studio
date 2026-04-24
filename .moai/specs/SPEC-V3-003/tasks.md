# SPEC-V3-003 Task Decomposition

SPEC: SPEC-V3-003 v1.0.0
Source: strategy.md §5 (manager-strategy analysis, 2026-04-24)
Mode: TDD (RED-GREEN-REFACTOR), harness=thorough
Branch: feat/v3-scaffold (manual mode, auto_commit=true, auto_branch=false)

## Task 개요

총 14 tasks × 3 milestones (MS-1 T1-T7, MS-2 T8-T11, MS-3 T12-T14)
+ 4 Plan Spikes (S1/S2/S3/S4) 실행 시점: Run Phase 2 진입 직후, S1/S3/S4 병렬, S2는 S1 FAIL 조건부.

## Task 상세표

| Task ID | Description | Requirement (REQ-P) | Dependencies | Planned Files (spec.md §9 canonical) | Mapped AC | Tier | Status |
|---------|-------------|---------------------|--------------|---------------------------------------|-----------|------|--------|
| T1 | PaneTree enum + in-order iterator + split/close 알고리즘 + unit tests | REQ-P-001, 002, 003, 004, 005 (RG-P-1) | - | crates/moai-studio-ui/src/panes/mod.rs, panes/tree.rs, lib.rs (pub mod panes 추가) | AC-P-1, AC-P-3, AC-P-20, AC-P-2(부분) | M | pending |
| T2 | PaneConstraints associated const + negative API surface test | REQ-P-010, 011, 014 (RG-P-2) | T1 | panes/constraints.rs | AC-P-21(직접), AC-P-4(간접) | S | pending |
| T3 | PaneSplitter + ResizableDivider traits + Mock impls (#[cfg(test)]) | REQ-P-061, 062 (RG-P-7) | T1, T2 | panes/splitter.rs, panes/divider.rs | AC-P-17 | S | pending |
| T4 | PaneSplitter 구체 구현 (S1 PASS→native / S2 PASS→gpui-component) + PtyWorker 연동 | REQ-P-001, 002, 003, 011, 013 | T3, **S1** (조건부 S2) | panes/splitter.rs (impl), TerminalSurface 재사용 | AC-P-1(통합), AC-P-2, AC-P-5, AC-P-16, AC-P-18 | L | pending |
| T5 | ResizableDivider 구체 구현 + drag clamp | REQ-P-005, 012 (RG-P-1, RG-P-2) | T3, T4, **S1** | panes/divider.rs (impl) | AC-P-6, AC-P-4 | M | pending |
| T6 | Focus routing + MS-1 키 바인딩 (prev/next pane, mouse click, platform_mod macro) | REQ-P-020, 021, 022, 024 (RG-P-3) + REQ-P-030, 031, 032(부분), 033 (RG-P-4) | T4 | panes/focus.rs | AC-P-7, AC-P-22, AC-P-23, AC-P-9a/9b(MS-1 부분) | L | pending |
| T7 | RootView 통합 + content_area 재배선 (lib.rs 4지점, drive-by refactor 금지) | REQ-P-060 (RG-P-7) | T4, T5, T6 | lib.rs (:75 필드 교체, :184 main_body, :286-300 시그니처, :410-444 content_area) | AC-P-16, AC-P-24(부분) | M | pending |
| T8 | TabContainer + Tab + new_tab/switch_tab/close_tab + last_focused_pane 복원 | REQ-P-023, 040, 041, 042, 043, 045 (RG-P-5) | T7 | tabs/mod.rs, tabs/container.rs | AC-P-8, AC-P-10, AC-P-11, AC-P-24, AC-P-25 | M | pending |
| T9 | MS-2 키 바인딩 + tmux 중첩 integration (Cmd/Ctrl+T, 1-9, \{, \}) | REQ-P-030, 031, 032(MS-2 전체), 034 (RG-P-4) | T8, **S4 사용자 결정** | tabs/container.rs, tests/integration_tmux_nested.rs | AC-P-9a, AC-P-9b, AC-P-26(v1.0.0 Nm-1) | M | pending |
| T10 | 탭 바 UI + toolbar.tab.active.background design token + bold active indicator | REQ-P-044 (RG-P-5) + §6.3 (접근성) | T8, design token | tabs/bar.rs, .moai/design/v3/system.md (toolbar token 추가) | AC-P-27(v1.0.0 Nm-2), AC-P-24 | M | pending |
| T11 | 탭 성능 bench (Cmd/Ctrl+1↔9 50 cycles, avg <= 50ms) | §6.1 성능 목표 | T8, T9 | benches/tab_switch.rs | AC-P-19 | S | pending |
| T12 | Persistence schema (moai-studio/panes-v1) + atomic write + cwd fallback to $HOME | REQ-P-050, 051, 052, 053, 054, 055, 056 (RG-P-6) | T8 | crates/moai-studio-workspace/src/persistence.rs | AC-P-12, AC-P-13, AC-P-13a, AC-P-14, AC-P-15 | L | pending |
| T13 | Persistence e2e (shutdown/startup hook, WindowCloseEvent → save_panes, app main → restore_panes) | REQ-P-050 ~ 056 (integration) | T12 | lib.rs (hook), moai-studio-app/src/main.rs (restore entry), tests/integration_persistence.rs | AC-P-12(e2e), AC-P-13(e2e) | M | pending |
| T14 | CI regression gate `.github/workflows/ci-v3-pane.yml` (5 job × macos-14/ubuntu-22.04 matrix + tmux/Zig setup) | §11.3 milestone regression gate | T1-T13 | .github/workflows/ci-v3-pane.yml | AC-P-16, 전체 AC 실행 경로 | S | pending |

## 4 Plan Spikes (Run Phase 2 진입 직후)

| Spike | Priority | Parallel with | PASS criterion | FAIL fallback | 산출 |
|-------|----------|----------------|----------------|---------------|------|
| **S1** GPUI 0.2.2 divider drag API | High | T1, S3, S4 | ≤ 200 LOC 예제 + 60 fps 유지 + native API only | → S2 자동 escalate | docs/spikes/SPIKE-V3-003-01-gpui-divider.md |
| **S2** longbridge/gpui-component Resizable | High (조건부) | S1 FAIL 시만 | cargo build 통과 + 60 fps + commit SHA pin | → user escalation, plan 재조정 | docs/spikes/SPIKE-V3-003-02-gpui-component.md |
| **S3** PaneId / TabId 생성 방식 | Medium | T1, S1, S4 | trade-off 표 + 기존 workspace ID consistency | 경로 2 (counter) 또는 3 (nanos) 기본값, uuid YAGNI | docs/spikes/SPIKE-V3-003-03-id-generation.md |
| **S4** Linux Ctrl+D/W/\\ shell 관례 UX | Medium | T1, S1, S3 | 측정 완료 + (a) 현행 유지 or (b) Shift escalation 이원 경로 | (a) 기본값. **[USER-DECISION-REQUIRED]** | docs/spikes/SPIKE-V3-003-04-linux-shell-conventions.md |

## USER-DECISION-REQUIRED 3건 (실제 결정 시점에 개별 AskUserQuestion, 2026-04-24 승인)

| Marker | 시점 | 옵션 요약 | 기본값 |
|--------|------|-----------|--------|
| spike-4-linux-shell-path | S4 보고서 완성 후 / T6, T9 Linux 구현 직전 | (a) 현행 Ctrl 유지 + Customization SPEC 연기 / (b) Ctrl+Shift escalation + annotation cycle 재개 | (a) |
| gpui-component-adoption | S2 완료 직후 / T4 구현 직전 | (a) S1 PASS→자체 구현 / (b) S2 PASS→gpui-component+SHA pin / (c) 양쪽 FAIL→escalation | S1 PASS 시 (a) 자동 |
| design-token-color-value | T10 RED phase 직전 | (a) 기존 BG_SURFACE_3 계열 / (b) BG_SURFACE_2 / (c) 사용자 커스텀 | (a) |

## Milestone 전이 Gate

| Gate | 통과 조건 |
|------|-----------|
| MS-1 → MS-2 | AC-P-1~7, 16, 17, 18, 20, 21, 22, 23 GREEN + AC-P-9a/9b MS-1 부분 GREEN + SPEC-V3-002 74 tests regression 0 |
| MS-2 → MS-3 | MS-1 전체 + AC-P-8, 9a(전체), 9b(전체 — **S4 사용자 결정 확정 후**), 10, 11, 19, 24, 25, 26(tmux runner), 27 |
| Post-MS-3 | 전체 29 AC GREEN + 기존 134 tests regression 0 + 신규 test ≥ 20 + coverage ≥ 85% + clippy -D warnings + fmt check + ci-v3-pane.yml 5 jobs × matrix GREEN + MX tags (ANCHOR 9+ / WARN 3+ / NOTE 7+ / TODO 2+) |

---

REQ coverage: REQ-P-001 ~ REQ-P-062 (37 REQ 전체)
AC coverage: AC-P-1 ~ AC-P-27 + AC-P-9a + AC-P-9b + AC-P-13a (29 AC 전체, Section 5.2 strategy.md 에서 누락 0건 확인)
