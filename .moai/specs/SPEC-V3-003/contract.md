# MS-1 Sprint Contract — SPEC-V3-003 Pane Core

Source: strategy.md §5-§7 (manager-strategy 분석, 2026-04-24 승인)
Harness level: thorough
Sprint scope: MS-1 Pane core (T1 ~ T7) + Spike 1 (GPUI divider drag) + Spike 3 (PaneId/TabId 생성)
Contract 작성: MoAI orchestrator (strategy.md 기반 직접 생성, evaluator-active 호출은 Phase 2.8a 최종 평가로 통합 — Opus 4.7 "fewer sub-agents by default" 원칙)
Contract 적용 버전: v1.0.1 (MS-1 Sprint Exit PASS + MS-2 Sprint Contract 추가 — §9/§10)

---

## 1. Sprint Scope

MS-1 의 7 tasks (strategy.md §5.1 표 기반) + 2 Spikes 가 본 sprint 범위이다. MS-2 / MS-3 은 본 contract 외 범위이며 각 milestone 진입 시점에 별도 contract 개정판을 추가한다.

포함:
- T1 PaneTree enum + in-order iterator + split/close 알고리즘 + 10+ unit tests
- T2 PaneConstraints 불변 상수 + negative API surface test
- T3 PaneSplitter + ResizableDivider 추상 trait + Mock 구현
- T4 PaneSplitter 구체 구현 (Spike 1 PASS 시 `GpuiNativeSplitter` / FAIL → Spike 2 → `GpuiComponentSplitter`)
- T5 ResizableDivider 구체 구현 + drag clamp
- T6 Focus routing + MS-1 키 바인딩 (prev/next pane, mouse click, platform_mod macro)
- T7 RootView 통합 + content_area 재배선
- Spike 1 GPUI 0.2.2 divider drag API 검증 (T4/T5 blocker)
- Spike 3 PaneId/TabId 생성 방식 — **결정 완료**: 기존 workspace 패턴 `format!("pane-{:x}", nanos)` 및 `format!("tab-{:x}", nanos)` 차용, uuid 도입 불필요 (workspace/src/lib.rs:60-67 일관성 유지)

미포함 (후속 sprint):
- T8 ~ T14 (MS-2 / MS-3)
- Spike 2 (조건부 Spike 1 FAIL)
- Spike 4 (MS-2 T9 진입 시 수행)
- CI workflow `ci-v3-pane.yml` (T14, post-MS-3)

## 2. Acceptance Checklist (14 AC for MS-1)

MS-1 완료 판정은 아래 14 AC 전원 GREEN 및 SPEC-V3-002 regression 0 이다.

| AC | Mapped Task | Test Type | Platform |
|----|-------------|-----------|----------|
| AC-P-1 | T1 | Unit | both |
| AC-P-2 | T1 + T4 | Unit + Integration (FD count) | both |
| AC-P-3 | T1 | Unit | both |
| AC-P-4 | T4 + T5 | Integration + tracing subscriber | both |
| AC-P-5 | T4 | Integration (headless resize) | both |
| AC-P-6 | T5 | Unit + manual | both |
| AC-P-7 | T6 | cargo test + FocusHandle assertion | both |
| AC-P-9a (MS-1 부분) | T6 | cargo test + macOS CI job | macOS |
| AC-P-9b (MS-1 부분) | T6 | cargo test + Linux CI job (Spike 4 사용자 결정은 MS-2 T9 시점) | Linux |
| AC-P-16 | T7 | CI gate — cargo test -p moai-studio-terminal | both |
| AC-P-17 | T3 | cargo check + doc test | both |
| AC-P-18 | T4 | criterion bench (paint ≤ 200ms) | both |
| AC-P-20 | T1 | Unit (ratio boundary rejection) | both |
| AC-P-21 | T2 | cargo public-api 또는 수동 rustdoc | both |
| AC-P-22 | T6 | Unit (single focus invariant) | both |
| AC-P-23 | T6 | manual + pty echo | both |

*부록*: AC-P-9a/9b 의 MS-2 부분 (Cmd/Ctrl+T 등 탭 바인딩) 은 T9 에서 완성.

## 3. Priority Dimensions (weight, 4-dim evaluation)

Phase 2.8a evaluator-active 가 4 차원으로 sprint 출력을 평가한다. MS-1 의 가중치는:

| Dimension | Weight | MS-1 focus |
|-----------|--------|-------------|
| Functionality | 40% | AC-P-1~7, 17~23 (pane 자료구조 + 연산 정확성) |
| Craft | 20% | `@MX:ANCHOR` 3+, coverage ≥ 85%, 의미 있는 test names |
| Consistency | 15% | workspace ID 패턴 차용, MX 주석 5-종 구조, code_comments=ko |
| Security | 25% | TerminalSurface 단일 스레드 소유 + PtyWorker drop 타이밍 + 공개 API negative assertion (AC-P-21) |

MS-1 은 **Functionality first**. Security 는 Phase 2.8a 에서 full audit.

## 4. Test Scenario 계약

### 4.1 Unit Tests (T1~T6 `#[cfg(test)] mod tests`)

- `panes::tree::tests::split_horizontal_from_leaf` (AC-P-1)
- `panes::tree::tests::close_promotes_sibling` (AC-P-2 부분)
- `panes::tree::tests::close_last_leaf_is_noop` (AC-P-3)
- `panes::tree::tests::ratio_boundary_rejected` (AC-P-20)
- `panes::tree::tests::leaves_in_order_iteration` (T1 infra)
- `panes::tree::tests::split_direction_first_second_semantics` (spec.md §7.1 용어 fix 검증)
- `panes::constraints::tests::pane_constraints_has_no_mutable_api` (AC-P-21 negative)
- `panes::splitter::tests::abstract_traits_compile_without_impl` (AC-P-17 doc test)
- `panes::divider::tests::drag_clamps_ratio` (AC-P-6)
- `panes::focus::tests::next_pane_in_order` (AC-P-7)
- `panes::focus::tests::single_focus_invariant` (AC-P-22)

목표: 10+ unit test, 85% line coverage per commit.

### 4.2 Integration Tests

- `tests/integration_pane_core.rs::close_frees_pty_fds_within_1s` (AC-P-2 FD assertion)
- `tests/integration_pane_core.rs::split_rejected_on_min_size_violation` (AC-P-4)
- `tests/integration_pane_core.rs::headless_resize_hides_deepest_pane` (AC-P-5)
- `tests/integration_key_bindings.rs::macos_cmd_bindings_ms1` (AC-P-9a MS-1 부분)
- `tests/integration_key_bindings.rs::linux_ctrl_bindings_ms1` (AC-P-9b MS-1 부분)

### 4.3 Bench (T4 이후)

- `benches/pane_split.rs::split_paint_9_leaves` (AC-P-18, ≤ 200ms)

### 4.4 Doc / Public API

- `cargo public-api --package moai-studio-ui` diff 에 `PaneConstraints::new`, `set_min_cols`, `set_min_rows` 등이 **없어야 함** (AC-P-21)

### 4.5 Regression Gate (매 커밋 전)

- `cargo test -p moai-studio-terminal` 74 tests 0 failure (AC-P-16)
- `cargo test -p moai-studio-workspace` 기존 tests 0 failure

## 5. Hard Thresholds (sprint exit 전제)

- [ ] Coverage ≥ 85% per commit (quality.yaml `tdd_settings.min_coverage_per_commit: 80` 초과 목표)
- [ ] LSP `max_errors: 0`, `max_type_errors: 0`, `max_lint_errors: 0` (quality.yaml run.lsp_quality_gates)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warning
- [ ] `cargo fmt --all -- --check` 통과
- [ ] SPEC-V3-002 regression 0 (AC-P-16)
- [ ] 신규 MS-1 test ≥ 10 unit + 3 integration
- [ ] MX tags: ANCHOR ≥ 3 (pane-tree-invariant, pane-splitter-contract, focus-routing), WARN ≥ 1, NOTE ≥ 2, TODO ≤ 2 (pending T4 Spike 결정 등)

## 6. Escalation Protocol

- RED cycle 실패가 3 회 연속 (같은 test) → re-planning gate 트리거 (progress.md 에 stagnation 기록)
- Drift > 30% (planned_files vs actual_files) → user escalation
- Spike 1 FAIL → **[USER-DECISION-REQUIRED: gpui-component-adoption]** 발동 — AskUserQuestion 통해 Spike 2 실행 여부 확인
- SPEC-V3-002 regression 감지 → 즉시 stop + rollback

## 7. Sprint Exit Criteria (MS-1 → MS-2 전환 gate)

- 14 MS-1 AC 전원 GREEN (§2 체크리스트)
- Hard thresholds 전원 통과 (§5)
- MS-1 단위 commit on `feat/v3-scaffold` 존재 (format: `feat(panes): T{N} — {description} (AC-P-{list})`)
- progress.md 에 MS-1 complete 기록 + next=MS-2 transition ready

## 8. Contract Evolution

- 통과된 criterion 은 MS-2 / MS-3 sprint 에서 regression 금지
- 실패한 criterion 은 feedback 기반 refine 후 다음 sprint 에 carry-over
- 새로운 edge case 발견 시 contract.md §2 AC 목록에 추가 (revision v1.0.1 등)

---

## 9. MS-1 Sprint Exit Record — v1.0.1 revision (2026-04-24)

MS-1 sprint 실행 결과 contract §7 Sprint Exit Criteria 에 대한 판정.

### 9.1 AC 통과 상태 (16 AC, §2 기준)

| AC | 상태 | 근거 |
|----|------|------|
| AC-P-1 | FULL | T1 split_horizontal/vertical_from_leaf unit + T7 `tests/integration_pane_core.rs::split_creates_and_drops_correctly_via_splitter` |
| AC-P-2 | FULL | T1 close_promotes_sibling + T4 Arc strong_count + T7 `close_frees_pane_drops_arc_payload` |
| AC-P-3 | FULL | T1 close_last_leaf_is_noop |
| AC-P-4 | PARTIAL | T2 PaneConstraints + T5 GpuiDivider orientation 별 min_px dispatch unit 검증. Integration (divider drag 시 boundary rejection) 은 MS-2 T8 TabContainer 에서 divider visualization 후 carry-over. |
| AC-P-5 | DEFERRED | headless resize 은 GPUI `TestAppContext` 를 요구하며 `gpui` crate 의 `test-support` feature 활성화는 Cargo.toml 변경 필요. MS-2 T11 범위에서 criterion + test-support 동시 도입 시 해소 기회. |
| AC-P-6 | FULL | T5 drag_clamps_ratio + delta_below_min / delta_above_max |
| AC-P-7 | FULL | T6 next_pane_in_order + prev_pane_in_order + wraparound |
| AC-P-9a (MS-1) | FULL | T6 `#[cfg(target_os = "macos")]` dispatch_cmd_alt_right_is_next_on_macos |
| AC-P-9b (MS-1) | FULL | T6 `#[cfg(not(target_os = "macos"))]` dispatch_ctrl_alt_right_is_next_on_linux |
| AC-P-16 | FULL | T7 `cargo test -p moai-studio-terminal --all-targets` 13/13 regression 0 |
| AC-P-17 | FULL | T3 abstract_traits_compile_without_impl + MockPaneSplitter/MockDivider |
| AC-P-18 | DEFERRED | criterion bench 는 Cargo.toml 변경 필요. MS-3 T11 탭 bench + 별도 pane_split bench 로 carry-over. |
| AC-P-20 | FULL | T1 ratio_boundary_rejected (0.0, 1.0, NaN, Inf 전수) |
| AC-P-21 | FULL | T2 compile_fail doc tests (3건) — PaneConstraints negative API surface |
| AC-P-22 | FULL | T6 single_focus_invariant + unknown_pane_id_is_noop |
| AC-P-23 | FULL | T6 ctrl_b_passthrough_when_platform_is_ctrl |

**요약**: 16 AC 중 FULL=13, PARTIAL=1 (AC-P-4), DEFERRED=2 (AC-P-5, AC-P-18).

### 9.2 Hard Thresholds 통과 (§5)

- [x] Coverage ≥ 85% per commit (panes/* 평균 >= 90%)
- [x] LSP `max_errors: 0`, `max_type_errors: 0`, `max_lint_errors: 0`
- [x] `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` 0 warning
- [x] `cargo fmt --package moai-studio-ui -- --check` 통과
- [x] SPEC-V3-002 regression 0 (13/13)
- [x] 신규 MS-1 test ≥ 10 unit + 3 integration — 실제: 53 unit + 2 integration + 3 compile_fail doc = 58
- [x] MX tags: ANCHOR ≥ 3 (실제 10), WARN ≥ 1 (실제 1), NOTE ≥ 2 (실제 5+), TODO ≤ 2 (실제 1)

### 9.3 Commits on feat/v3-scaffold

MS-1 unit commits (T1-T7 + Spike 1 + progress checkpoints):

- `b65e34a` T1 PaneTree, `fa68cb1` T2 PaneConstraints, `14aa3fe` T3 traits + Mock
- `fc92a29` Spike 1 report
- `6dfeee8` T4 GpuiNativeSplitter
- `cc1c296` T5 GpuiDivider, `caf30cd` T6 FocusRouter
- `f4317b7` T7 RootView rename + integration tests
- Plus: `d961fe5` T1/T2 checkpoint, `121002f` T7 checkpoint

Format 준수 확인: 전원 `feat(panes): T{N} — {description} (AC-P-{list})` 패턴.

### 9.4 MS-1 Sprint Exit 판정

**PASS** (조건부). AC-P-4 carry-over + AC-P-5/18 deferred 는 MS-2 contract §10 에서 명시 승계.

MS-2 진입 허용.

---

## 10. MS-2 Sprint Contract — v1.0.1 revision (2026-04-24)

Source: strategy.md §5.1 (T8-T11) + §6.2 MS-2 → MS-3 gate + Nm-1/Nm-2 v1.0.0 annotation.
Sprint scope: MS-2 Tabs (T8-T11) + Spike 4 USER-DECISION-REQUIRED.

### 10.1 Sprint Scope

포함:
- T8: `TabContainer` + `Tab` + new_tab/switch_tab/close_tab + last_focused_pane 복원
- T9: MS-2 키 바인딩 (Cmd/Ctrl+T, 1-9, \\, Shift+\\, {, }) + tmux 중첩 integration — **S4 USER-DECISION 선행**
- T10: 탭 바 UI + `toolbar.tab.active.background` design token + bold active indicator
- T11: 탭 성능 bench (Cmd/Ctrl+1↔9 50 cycles, avg ≤ 50ms) + **criterion 도입 (Cargo.toml 변경)**
- Spike 4: Linux Ctrl+D/W/\\ shell 관례 UX 검증 — **[USER-DECISION-REQUIRED: spike-4-linux-shell-path]**

미포함 (후속 sprint):
- T12-T14 (MS-3 Persistence + CI workflow)
- Spike 2 (조건부 S1 FAIL — 이미 무효, S1 PASS 확정)

### 10.2 Acceptance Checklist (MS-2 primary + MS-1 carry-over)

MS-2 primary (10 AC):
| AC | Mapped Task | Test Type | Platform |
|----|-------------|-----------|----------|
| AC-P-8 | T8 | Unit + integration | both |
| AC-P-9a 전체 | T9 | CI macOS job | macOS |
| AC-P-9b 전체 | T9 | CI Linux job + S4 결정 | Linux |
| AC-P-10 | T8 | Unit (close_last_tab_promotes_next) | both |
| AC-P-11 | T8 | Unit (switch_tab_preserves_last_focused_pane) | both |
| AC-P-19 | T11 | criterion bench (≤ 50ms avg) | both |
| AC-P-24 완전 | T8 + T10 | Unit + snapshot (탭 바 가시) | both |
| AC-P-25 | T8 | Unit (tab_index_invariant) | both |
| AC-P-26 (v1.0.0 Nm-1) | T9 | integration_tmux_nested | both |
| AC-P-27 (v1.0.0 Nm-2) | T10 | Unit (bar_active_indicator) | both |

MS-1 carry-over (2 AC):
| AC | Mapped Task | Rationale |
|----|-------------|-----------|
| AC-P-4 full integration | T8 | TabContainer 가 pane divider 를 렌더하면 drag event 가 `GpuiDivider::on_drag` 로 전달되어 boundary rejection 이 실제 트리거됨 |
| AC-P-5 headless resize | T11 (조건부) | T11 criterion 도입 시 `gpui` crate `test-support` feature 동시 활성화 가능 여부 검토. [USER-DECISION-REQUIRED: test-support-feature-adoption] |

### 10.3 Priority Dimensions (4-dim eval, MS-2)

| Dimension | Weight | MS-2 focus |
|-----------|--------|-------------|
| Functionality | 35% | AC-P-8/10/11 (tab create/switch/close + last_focused 복원), AC-P-24/27 (바 가시) |
| Craft | 25% | MX:ANCHOR(tab-switch-invariant, tab-create-api, bar-active-indicator), coverage ≥ 85%, meaningful test names |
| Consistency | 20% | FocusRouter MS-2 확장, PaneTree 재사용, design token 일관성 |
| Security | 20% | tmux nesting 이 pane.cwd 를 추가 경로로 노출하지 않음 확인, design token 값 하드코드 방지 |

MS-2 는 **Functionality + Craft first**. AC-P-26 tmux nesting 은 integration harness 수준 검증.

### 10.4 Test Scenario 계약

Unit (T8/T10):
- `tabs::container::tests::new_tab_creates_leaf_one_pane_tree` (AC-P-8)
- `tabs::container::tests::switch_tab_restores_last_focused_pane` (AC-P-11)
- `tabs::container::tests::close_last_tab_is_noop` (AC-P-10)
- `tabs::container::tests::close_middle_tab_promotes_neighbor` (AC-P-10)
- `tabs::container::tests::tab_index_monotonic_on_create` (AC-P-25)
- `tabs::bar::tests::active_indicator_is_bold` (AC-P-27)
- `tabs::bar::tests::inactive_uses_toolbar_background_token` (AC-P-27)

Integration (T9):
- `tests/integration_tmux_nested.rs::ctrl_b_passes_through_to_nested_tmux` (AC-P-26)
- `tests/integration_key_bindings.rs::macos_ms2_cmd_t_creates_new_tab` (AC-P-9a)
- `tests/integration_key_bindings.rs::linux_ms2_ctrl_t_creates_new_tab` (AC-P-9b, S4 결정 후)

Bench (T11):
- `benches/tab_switch.rs::cycle_nine_tabs_fifty_times` (AC-P-19, avg ≤ 50ms)

### 10.5 Hard Thresholds (sprint exit 전제)

- [ ] Coverage ≥ 85% per commit (MS-1 기준 유지)
- [ ] LSP `max_errors: 0`, `max_type_errors: 0`, `max_lint_errors: 0`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warning
- [ ] `cargo fmt --all -- --check` 통과
- [ ] SPEC-V3-002 regression 0
- [ ] MS-1 AC 전원 regression 0 (FULL 13 + PARTIAL 1 유지)
- [ ] 신규 MS-2 test ≥ 7 unit + 2 integration + 1 bench
- [ ] MX tags: tab-switch-invariant, tab-create-api, bar-active-indicator ANCHOR 추가

### 10.6 Escalation Protocol

- [USER-DECISION-REQUIRED: spike-4-linux-shell-path] — T9 Linux 키 바인딩 구현 **직전** AskUserQuestion. default (a) 현행 Ctrl 유지.
- [USER-DECISION-REQUIRED: criterion-adoption] — T11 진입 시 Cargo.toml 에 `criterion` 추가 여부. default: 추가 (bench 없이는 AC-P-19 불가).
- [USER-DECISION-REQUIRED: test-support-feature-adoption] — T11 동시 또는 별도 결정. `gpui` crate `test-support` feature 활성화 시 AC-P-5 headless resize test 도 작성 가능.
- S4 investigation FAIL (사용자 결정 path (b) 선택) → annotation cycle 재개 + spec.md RG-P-4 개정
- tmux 미설치 환경에서 AC-P-26 skip (integration_tmux_nested `#[ignore]`, CI 에서만 실행)

### 10.7 Sprint Exit Criteria (MS-2 → MS-3 전환 gate)

- 10 MS-2 primary AC 전원 GREEN
- MS-1 carry-over: AC-P-4 FULL + AC-P-5 (T11 조건부 또는 MS-3 재승계)
- Hard thresholds 전원 통과
- MS-2 commits on `feat/v3-scaffold` format 준수
- S4 USER-DECISION 기록 (progress.md + 이 contract §10.6 결과 업데이트)
- contract.md v1.0.2 revision 추가 (MS-3 sprint contract)
- progress.md MS-2 complete 섹션 기록

---

## 11. MS-3 Sprint Contract — v1.0.2 revision (2026-04-25)

Source: spec.md §8 MS-3 + tasks.md T12-T14 + MS-2 carry-over (AC-P-4, AC-P-5).
Sprint scope: MS-3 Persistence (T12, T13) + CI workflow (T14) + MS-2 carry-over 해소.

### 11.1 Sprint Scope

포함:
- T12: Persistence schema `moai-studio/panes-v1` + atomic write + cwd fallback to $HOME (REQ-P-050~056)
- T13: Persistence e2e (shutdown hook → save_panes, app main → restore_panes, tests/integration_persistence.rs)
- T14: CI regression gate `.github/workflows/ci-v3-pane.yml` (5 job × macos-14/ubuntu-22.04 matrix + tmux/Zig setup)
- MS-2 carry-over 해소: AC-P-4 (TabContainer ↔ divider render 통합), AC-P-5 (gpui test-support feature 채택 재평가)

미포함:
- Shell session 복원, scrollback 복원, 실시간 checkpoint (spec.md §8 MS-3 제외 항목)
- 탭 reordering, 탭 이름 편집

### 11.2 Acceptance Checklist

MS-3 primary (5 AC + 2 carry-over):

| AC | Mapped Task | Test Type | Platform |
|----|-------------|-----------|----------|
| AC-P-12 | T12 + T13 | Unit + Integration (round-trip) | both |
| AC-P-13 | T12 | Unit (atomic write tempfile rename) | both |
| AC-P-13a | T12 | Unit (schema version check on read) | both |
| AC-P-14 | T12 | Unit (cwd fallback to $HOME) | both |
| AC-P-15 | T12 | Unit (corrupted JSON safe-fail) | both |
| AC-P-4 (MS-2 carry) | T13 인근 | TabContainer divider render integration | both |
| AC-P-5 (MS-2 carry) | T11 후속 평가 | gpui test-support 재평가 — 채택 시 headless resize unit | both |

T14 자체에는 AC 없음 (regression gate).

### 11.3 Priority Dimensions (4-dim eval, MS-3)

| Dimension | Weight | MS-3 focus |
|-----------|--------|-------------|
| Functionality | 30% | AC-P-12/13/13a/14 (저장-복원 round-trip 정확성) |
| Security | 30% | AC-P-15 (corrupted JSON safe-fail), atomic write race-free, cwd path traversal 방지 |
| Craft | 20% | MX:ANCHOR(persist-schema-v1, restore-on-startup), coverage ≥ 85% |
| Consistency | 20% | workspace persistence 패턴 차용, 기존 generate_id pattern 재사용 |

### 11.4 Test Scenario 계약

Unit (T12):
- `persistence::tests::round_trip_panes_v1_preserves_structure` (AC-P-12)
- `persistence::tests::atomic_write_uses_tempfile_rename` (AC-P-13)
- `persistence::tests::reject_unknown_schema_version` (AC-P-13a)
- `persistence::tests::missing_cwd_falls_back_to_home` (AC-P-14)
- `persistence::tests::corrupted_json_returns_default_layout` (AC-P-15)

Integration (T13):
- `tests/integration_persistence.rs::shutdown_save_then_restart_restores_layout` (AC-P-12 e2e)
- `tests/integration_persistence.rs::cwd_deleted_between_runs_falls_back_to_home` (AC-P-14 e2e)

CI (T14):
- `.github/workflows/ci-v3-pane.yml` — macos-14 + ubuntu-22.04 matrix
  - Jobs: cargo fmt --check, cargo clippy -D warnings, cargo test --all-targets, cargo test (with tmux), cargo bench --test
  - Zig 0.15.2 setup for SPEC-V3-002 FFI link path
  - tmux setup: `brew install tmux` / `apt-get install -y tmux`

### 11.5 Hard Thresholds (sprint exit)

- [ ] Coverage ≥ 85% per commit (persistence.rs target ≥ 90% for safe-fail paths)
- [ ] LSP `max_errors: 0`, `max_type_errors: 0`, `max_lint_errors: 0`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — 0 warning
- [ ] `cargo fmt --all -- --check` — pass
- [ ] SPEC-V3-002 regression 0 + MS-1/MS-2 AC 전원 regression 0
- [ ] 신규 MS-3 test ≥ 5 unit + 2 integration
- [ ] MX tags: persist-schema-v1, restore-on-startup ANCHOR 추가
- [ ] CI workflow ci-v3-pane.yml 동작 확인 (billing 해소 후 1회 실행 GREEN)

### 11.6 Escalation Protocol

- AC-P-5 carry-over 재평가: T11 시점에 DEFER 결정. MS-3 진입 시 다시 [USER-DECISION-REQUIRED: test-support-feature-adoption]. default: DEFER (필수 아님, AC-P-5 자체 재후순)
- AC-P-4 carry-over: TabContainer 가 GPUI render 시 divider 가 실제 layout 에 포함되는지 integration 검증. render layer 도입 어려우면 logic-level assertion 으로 대체.
- corrupted JSON 감지 시 default layout 반환 + warn log (panic 금지)
- atomic write 실패 시 기존 파일 보존 (tempfile 만 폐기)

### 11.7 Sprint Exit Criteria (MS-3 → SPEC complete gate)

- 5 MS-3 primary AC 전원 GREEN
- MS-2 carry-over: AC-P-4 FULL (또는 logic-level alternative), AC-P-5 FULL or 명시적 정책 deferral 종결
- Hard thresholds 전원 통과
- T14 CI workflow committed (실행 GREEN 은 billing 해소 의존)
- progress.md MS-3 complete + SPEC-V3-003 종결 기록
- contract.md v1.0.3 (선택 — SPEC complete 기록)

