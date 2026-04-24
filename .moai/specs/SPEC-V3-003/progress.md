## SPEC-V3-003 Progress

- Started: 2026-04-24T (run phase entry — ultrathink 키워드 감지, Adaptive Thinking 활성)
- Branch: feat/v3-scaffold (4 commits ahead of origin)
- SPEC status: approved v1.0.0 (annotation cycle 승인, NM-1/Nm-1/Nm-2/Nm-3 해소)
- Prior artifacts: spec.md (62KB), plan.md (41KB), acceptance.md (43KB), research.md (37KB)

### Phase 0.5 (memory_guard): SKIPPED
- quality.yaml `memory_guard.enabled: false` → skip environment memory assessment

### Phase 0.9 (JIT Language Detection): complete
- Detected: Cargo.toml workspace at repo root (resolver=2, rust-version=1.93, edition=2024, members=["crates/*"])
- Primary language skill: moai-lang-rust
- Additional language contexts: Zig 0.15.2 (libghostty-vt FFI, SPEC-V3-002 상속, Terminal Core 는 변경 금지이므로 Zig 직접 편집 없음)

### Phase 0.95 (Scale-Based Execution Mode): complete
- SPEC scope files: 8 신규 (panes/mod+tree+splitter+divider+focus+constraints, tabs/mod+container+bar) + 5 수정 (lib.rs 4지점 + workspace/persistence.rs) = 13 파일
- Domains: 3 (UI panes, UI tabs, workspace persistence)
- Complexity signals: 37 REQ-P + 29 AC + 3 milestones + 4 Plan Spikes
- Selected mode: **Full Pipeline** (SPEC scope >= 10 files AND >= 3 domains)
- Agents: manager-strategy (Phase 1) + manager-tdd (Phase 2, development_mode=tdd) + manager-quality (Phase 2.5) + evaluator-active (Phase 2.8a) + manager-git (Phase 3)

### Harness Level: thorough
- Rationale: complex multi-domain SPEC (3 domains, 29 AC, 4 Spikes) + ultrathink 요청
- Sprint Contract: enabled (Phase 2.0 contract.md 생성 예정)
- evaluator-active mode: per-sprint (Phase 2.0 + Phase 2.8a 양쪽)

### UltraThink Activation
- Trigger: user included `ultrathink` keyword in `/moai run SPEC-V3-003 ultrathink`
- Mode: Claude native extended reasoning (Adaptive Thinking for claude-opus-4-7)
- Applied to: Phase 1 manager-strategy 위임 (deeper architectural analysis + 4 Spike 배치 전략)

### Initial codebase state (targets)
- crates/moai-studio-ui/src/: lib.rs, terminal/{mod,clipboard,input}.rs (기존 SPEC-V3-001/002 산출)
- crates/moai-studio-workspace/src/: lib.rs only
- crates/moai-studio-terminal/src/: SPEC-V3-002 산출물 (74 tests, 변경 금지)

### Phase 1 (Analysis & Planning): complete
- Delegated to manager-strategy subagent (foreground, isolation=none, ultrathink via Adaptive Thinking)
- Agent ID: a2cdfe3cd65326793 (retained for potential SendMessage follow-up)
- Output: `.moai/specs/SPEC-V3-003/strategy.md` (9 sections, 29 AC coverage verified, 14 tasks TDD graph, 4 Spikes placement, 3 USER-DECISION-REQUIRED markers, 7 reuse patterns, 5 new risks R-P1~R-P5)
- Key findings:
  - Codebase reality check: plan.md line ref `lib.rs:290-299` → actual `:286-300` (minor drift, T7 must re-verify during RED phase)
  - Reuse opportunities: `$schema` versioned JSON + HOME/APPDATA split + ID pattern from `WorkspacesStore` + MX 주석 5-종 pattern from `worker.rs`
  - YAGNI flagged: `uuid` crate adoption (use existing nanos+prefix), `PaneConstraints::new(...)` (AC-P-21 forbids mutable API)
  - New risk R-P1: GPUI 0.2.2 headless test feasibility unverified — measure in T6/T8/T10 RED
  - New risk R-P2: tmux CI dependency — `apt install tmux` / `brew install tmux` in ci-v3-pane.yml

### Decision Point 1 (HUMAN GATE Plan Approval): APPROVED
- AskUserQuestion response: "승인하고 Phase 1.5부터 진행" + "실제 결정 시점에 개별 AskUserQuestion"
- Timestamp: 2026-04-24 run session
- Implication: USER-DECISION-REQUIRED 3 markers will be raised at their respective decision points (S4 completion, S2 completion, T10 RED phase), not preemptively

### Phase 1.5 (Task Decomposition): complete
- Output: `.moai/specs/SPEC-V3-003/tasks.md` persist (14 tasks × mapped AC × planned files × tier × status)
- Branch target: feat/v3-scaffold (per spec.md §11.4, per git-strategy.manual)
- All 29 AC coverage confirmed in Task 표 AC column

### Phase 1.6 (Acceptance Criteria Initialization): complete
- 29 AC 전체 TaskList 등록 (Batch 1: AC-P-1~10 = Tasks #11-15, 17-20; Batch 2: AC-P-11~19+13a = Tasks #21-31; Batch 3: AC-P-20~27 = Tasks #32-39)
- Failing checklist pattern — 모두 pending 상태. 각 AC 를 구현하며 completed 로 전환.
- 검증: acceptance.md §2-§5 scenarios 참조로 test location / failure mode / requirement 매핑 확인.

### Phase 1.7 (File Structure Scaffolding): complete
- Created 9 stub files with module-level documentation comments (Korean per `code_comments: ko`) + MX:TODO markers referencing task IDs:
  - crates/moai-studio-ui/src/panes/mod.rs
  - crates/moai-studio-ui/src/panes/tree.rs
  - crates/moai-studio-ui/src/panes/splitter.rs
  - crates/moai-studio-ui/src/panes/divider.rs
  - crates/moai-studio-ui/src/panes/focus.rs
  - crates/moai-studio-ui/src/panes/constraints.rs
  - crates/moai-studio-ui/src/tabs/mod.rs
  - crates/moai-studio-ui/src/tabs/container.rs
  - crates/moai-studio-ui/src/tabs/bar.rs
- lib.rs 는 무변경 (T1 RED phase 에서 `pub mod panes;` + T7 에서 전면 재배선). Rust 는 lib.rs 에서 참조하지 않는 파일도 컴파일 OK 이므로 stub 단계 baseline 불변.
- LSP baseline: cargo check 는 manager-tdd 가 Phase 2 진입 시 실행. 현재 repository 는 `cargo test -p moai-studio-terminal` 74 tests GREEN 상태 (SPEC-V3-002 post-completion).

### Phase 1.8 (Pre-Implementation MX Context Scan): complete
- MX Context Map (existing files):
  - `crates/moai-studio-ui/src/terminal/mod.rs:3` — `@MX:ANCHOR: terminal-surface-render` + `@MX:REASON: GPUI 렌더 진입점` (fan_in 높음, 변경 금지 — SPEC-V3-002 산출)
  - `crates/moai-studio-ui/src/terminal/mod.rs:19, 159` — `@MX:NOTE: font-metric-coord-mapping`
  - `crates/moai-studio-ui/src/terminal/mod.rs:76, 112` — `@MX:TODO` 2건 (SPEC-V3-002 후속, 본 SPEC 범위 외)
  - `crates/moai-studio-ui/src/lib.rs` — MX 태그 없음. T7 수정 시 `tab_container: Option<Entity<TabContainer>>` 필드에 `@MX:ANCHOR(root-view-content-binding)` 추가 필요 (strategy.md §5.1 T7 계획대로)
  - `crates/moai-studio-workspace/src/lib.rs` — MX 태그 없음. T12 `persistence.rs` 신규에서 `@MX:WARN(race-condition-on-concurrent-write)` + `@MX:ANCHOR(persistence-restore-entry)` 예정
- 제약 전달: Phase 2 agent prompt 에 terminal crate `@MX:ANCHOR: terminal-surface-render` 는 "절대 수정 금지" 계약으로 명시 (RG-P-7 AC-P-16 재확인)

### Spike 3 (PaneId / TabId 생성 방식): 결정 완료
- 조사: `crates/moai-studio-workspace/src/lib.rs:60-67` 기존 workspace ID 패턴 `format!("ws-{:x}", nanos)` 확인
- 결정: **기존 패턴 차용** — `PaneId = format!("pane-{:x}", nanos)`, `TabId = format!("tab-{:x}", nanos)`
- 근거: (1) workspace/terminal/pane/tab 의 ID 네이밍 consistency, (2) uuid crate 추가 불필요 (YAGNI 회피, Cargo.toml 변경 없음), (3) 충돌 가능성 무시 가능 (nanos precision 이면 동일 mill-sec 내 여러 pane 생성 시에도 carrier 가 다름, 필요시 counter 추가 fallback)
- 산출: 본 progress.md 기록 + tasks.md 반영. 별도 spike 보고서 미생성 (read-only design decision).

### Phase 2.0 (Sprint Contract, thorough harness): complete
- Output: `.moai/specs/SPEC-V3-003/contract.md` MS-1 sprint 계약 생성
- Scope: T1~T7 + Spike 1 + Spike 3 (완료)
- Priority: Functionality 40% / Security 25% (Phase 2.8a full audit) / Craft 20% / Consistency 15%
- Acceptance checklist: 14 MS-1 AC (AC-P-1~7, 9a/9b MS-1 부분, 16, 17, 18, 20, 21, 22, 23)
- Hard thresholds: 85% coverage, 0 LSP errors, 0 clippy warnings, SPEC-V3-002 regression 0
- Escalation: 3 연속 RED 실패 → re-planning gate, Spike 1 FAIL → AskUserQuestion
- evaluator-active 호출 전략: Phase 2.0 skip (strategy.md 가 이미 plan review 완료), Phase 2.8a 에서 1회 full 4-dim 평가 (Opus 4.7 "fewer sub-agents" 원칙)

### Phase 2 T1 (PaneTree RED-GREEN-REFACTOR): complete
- Agent: manager-tdd (subagent_type, isolation=worktree, foreground)
- Agent ID: acaf7776814bc1279 (retained for T2 SendMessage resume)
- Scope: T1 only (T2/T3/T4 범위 침범 없음)
- files modified:
  - crates/moai-studio-ui/src/panes/tree.rs (stub → 제네릭 PaneTree<L> + 13 unit tests, ~540 LOC)
  - crates/moai-studio-ui/src/panes/mod.rs (stub → pub re-export: PaneTree/PaneId/SplitNodeId/SplitDirection/SplitError/RatioError/Leaf)
  - crates/moai-studio-ui/src/lib.rs (`pub mod panes;` 1줄 추가, 다른 부분 무수정 — drive-by refactor 금지 준수)
- 구현 결정:
  - **제네릭 PaneTree<L>** 채택: prod `L = Entity<TerminalSurface>` (T4 통합), test `L = String` (GPUI context 없이 단위 검증). Rationale: doc comment 명시.
  - **PaneId/SplitNodeId 패턴**: Spike 3 결정대로 `format!("pane-{:x}", nanos)` / `format!("split-{:x}", nanos)` — workspace/src/lib.rs:60-67 차용.
  - **PaneId::new_from_literal(&str)** 추가: 테스트 편의 메서드. clippy `should_implement_trait` 회피 목적.
  - **Leaf<L>** 래퍼 구조체: PaneId + payload 분리 — close 알고리즘의 ownership 이전 단순화.
- test results:
  - `cargo test -p moai-studio-ui --lib panes::tree`: **13/13 PASS**
  - `cargo test -p moai-studio-terminal`: **74/74 PASS** (AC-P-16 regression gate GREEN)
  - Coverage (llvm-cov): panes/tree.rs **line 90.10% / branch 85.59%** (목표 85% 초과)
- MX tags added:
  - `panes/tree.rs:111` ANCHOR `pane-tree-invariant` + REASON (fan_in >= 4)
  - `panes/tree.rs:170` ANCHOR `pane-split-api` + REASON (fan_in >= 3: T4/T7/T9)
  - `panes/tree.rs:78` NOTE `horizontal-is-left-right-not-top-bottom` (spec.md §7.1 C-3)
  - `panes/tree.rs:19` TODO T4 PaneLeafHandle GPUI Entity 통합
- TRUST 5 self-check: T/R/U/S/T 전원 PASS
- implementation_divergence:
  - planned vs actual files: 완전 일치 (0% drift)
  - additional_features: PaneId::new_from_literal (test helper)
  - new_dependencies: 없음 (Cargo.toml 변경 없음)
- AC 통과 (T1 범위):
  - **AC-P-1** ✅ split_horizontal_from_leaf / split_vertical_from_leaf / split_direction_first_second_semantics 검증
  - **AC-P-3** ✅ close_last_leaf_is_noop 검증
  - **AC-P-20** ✅ ratio_boundary_rejected (0.0, 1.0, NaN, Inf 모두 Err)
  - **AC-P-2 (단위 부분)** ✅ close_promotes_sibling (integration FD count 는 T4 범위)
- blockers: 없음

### Next (후속 session resume): T2 PaneConstraints → T3 traits → T4 PaneSplitter 구체 구현 (Spike 1 blocker)
- T4 의존성: S1 (GPUI divider drag API 검증) 먼저 수행 필요
- session 이어받을 때: `/moai run SPEC-V3-003` 호출 → progress.md 읽고 T2 부터 resume

