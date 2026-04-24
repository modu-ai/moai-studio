# MS-1 Sprint Contract — SPEC-V3-003 Pane Core

Source: strategy.md §5-§7 (manager-strategy 분석, 2026-04-24 승인)
Harness level: thorough
Sprint scope: MS-1 Pane core (T1 ~ T7) + Spike 1 (GPUI divider drag) + Spike 3 (PaneId/TabId 생성)
Contract 작성: MoAI orchestrator (strategy.md 기반 직접 생성, evaluator-active 호출은 Phase 2.8a 최종 평가로 통합 — Opus 4.7 "fewer sub-agents by default" 원칙)
Contract 적용 버전: v1.0.0

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
