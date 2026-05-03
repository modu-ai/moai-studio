# SPEC-V0-2-0-MISSION-CTRL-001 Progress

**Started**: 2026-05-04
**Branch**: feature/SPEC-V0-2-0-MISSION-CTRL-001-ms1
**SPEC status**: implemented (MS-1 complete)
**Completion date**: 2026-05-04
**Predecessor**: SPEC-V3-010 (Agent Dashboard)
**audit reference**: feature-audit.md §3 Tier E v0.2.0 critical gap E-5 + §4 Top 8 #2 (⭐⭐⭐⭐⭐)

## MS-1 (2026-05-04 sess 12+) — AgentRunRegistry domain ✅

### Implementation

- `crates/moai-studio-agent/src/mission_control.rs` (신규):
  - `AgentCard` struct (run_id / label / status / last_event_summary / last_event_at / cost / event_count) — REQ-MC-001/002/003
  - `AgentRunRegistry` struct + Default impl — REQ-MC-010
  - `register / push_event / set_status / set_cost / cards / top_n_active / clear_terminal` API — REQ-MC-011~018
  - hook event_name → AgentRunStatus auto-transition rules (SessionStart/Stop/Notification) — REQ-MC-013
  - 단위 테스트 ~12개 (AC-MC-1 ~ AC-MC-10)
- `crates/moai-studio-agent/src/lib.rs`:
  - `pub mod mission_control;`
  - re-export `AgentCard`, `AgentRunRegistry`

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-MC-1 | AgentCard::new 초기 상태 (Running, count=0, no event) | ✅ |
| AC-MC-2 | register 두 번 호출 시 label 갱신 + history 보존 | ✅ |
| AC-MC-3 | push_event 자동 register + count/timestamp/summary 갱신 | ✅ |
| AC-MC-4 | hook Stop → status=Completed transition | ✅ |
| AC-MC-5 | hook Notification(error) → status=Failed transition | ✅ |
| AC-MC-6 | set_status 직접 갱신 (transition 무관) | ✅ |
| AC-MC-7 | set_cost snapshot 저장 | ✅ |
| AC-MC-8 | top_n_active 정렬 + active 필터 | ✅ |
| AC-MC-9 | clear_terminal 가 Completed/Failed/Killed 제거 | ✅ |
| AC-MC-10 | cards iterator + lookup | ✅ |

### Test count

- 신규: 16 (mission_control.rs T-MC 블록)
  - agent_card_new_initial_state (AC-MC-1)
  - registry_register_twice_updates_label_preserves_history (AC-MC-2)
  - push_event_auto_registers_and_updates_summary (AC-MC-3)
  - push_event_stop_transitions_to_completed (AC-MC-4)
  - push_event_notification_error_transitions_to_failed (AC-MC-5)
  - push_event_notification_info_does_not_transition (REQ-MC-013 negative)
  - push_event_arbitrary_hook_preserves_status (REQ-MC-013 negative)
  - push_event_stream_json_does_not_transition (REQ-MC-013)
  - set_status_overrides_directly (AC-MC-6)
  - set_status_unknown_id_is_noop (REQ-MC-014 robustness)
  - set_cost_stores_snapshot (AC-MC-7)
  - set_cost_unknown_id_is_noop (REQ-MC-015 robustness)
  - top_n_active_filters_terminal_and_sorts_recency (AC-MC-8)
  - top_n_active_respects_limit (REQ-MC-017)
  - clear_terminal_removes_terminal_cards_only (AC-MC-9)
  - cards_iterator_and_lookup (AC-MC-10)
  - truncate_chars_short_string_unchanged (helper)
  - truncate_chars_long_string_ellipsis (helper)
- moai-studio-agent crate tests: 기존 → 123 (+18 incl. 2 helper edge tests)
- 회귀 0 (events/ring_buffer/cost/sse_ingest/etc 무변경)
- ui crate: 1246 GREEN (회귀 0)
- terminal crate: 36 GREEN (회귀 0)
- workspace crate: 26 GREEN (회귀 0)
- clippy 0 warning, fmt clean

### Spec amendment (2026-05-04 in-progress)

REQ-MC-003 의 `PartialEq` 요구를 제거함. 사유: cost field 의 `CostSnapshot` (V3-010)
이 PartialEq 미구현 + REQ-MC-040 frozen zone 보호. AgentCard 는 `Clone + Debug` 만
derive. 향후 PartialEq 가 필요해지면 `#[derive(PartialEq)]` 를 V3-010 cost.rs 에 추가
(추가 derive 는 additive 이므로 V3-010 frozen zone 위반 아님 — 별 SPEC 으로 결정).

### Carry to MS-2 (별 PR — 다음 세션)

## MS-2 (2026-05-04 sess 12+) — MissionControlView GPUI Entity ✅

### Implementation

- `crates/moai-studio-ui/src/agent/mission_control_view.rs` (신규):
  - `CellData { card: Option<AgentCard> }` struct + `is_empty()` helper
  - `MissionControlView { snapshot: Vec<AgentCard>, max_cells: usize }` struct
  - `MissionControlView::new` (max_cells=4) + `set_snapshot` + `populated_cells` + `placeholder_cells` + `cell_data` API
  - 순수 helpers: `status_pill_color` (ADR-MC-4 mapping), `status_pill_label`, `format_cost` ("$0.0001" 4-decimal)
  - `impl Render`: 2x2 grid (`max_cells.div_ceil(2)` rows × 2 cells)
  - `render_cell` helper: status pill + label + last_event_summary + cost roll-up footer
  - `DEFAULT_MAX_CELLS = 4` constant
- `crates/moai-studio-ui/src/agent/mod.rs`:
  - `pub mod mission_control_view;`
  - re-export: `CellData`, `DEFAULT_MAX_CELLS`, `MissionControlView`, `format_cost`, `status_pill_color`, `status_pill_label`
- `crates/moai-studio-ui/src/lib.rs`:
  - RootView 에 `mission_control: Option<Entity<MissionControlView>>` 신규 필드 (R3)
  - new() 초기화 = None (lazy)
  - 4 helpers: `ensure_mission_control` (idempotent mount) + `dismiss_mission_control` + `update_mission_control_snapshot` + (cancel 추가 없음)
  - `Render for RootView` chain 끝 `.children(self.mission_control.clone())` 분기

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-MC-11 | 빈 view → 4 placeholder cells | ✅ |
| AC-MC-12 | 2 active cards → 2 filled + 2 placeholder | ✅ |
| AC-MC-13 | RootView::new mission_control == None | ✅ |

### Implementation choice (ADR-MC-1 deviation)

`MissionControlView` 가 `Vec<AgentCard>` 스냅샷을 owned 로 보유하는 것을 채택 (Arc<RwLock<...>> 또는 Entity<AgentRunRegistry> 공유 대신). 사유: 더 단순한 ownership, lock contention 없음, 테스트 주입 용이. RootView 가 registry 의 single owner 이며 `update_mission_control_snapshot` 으로 view 에 push.

### Test count

- 신규: 16 (mission_control_view.rs T-MS2 9개 + lib.rs T9 7개)
  - mission_control_view.rs:
    - empty_view_yields_four_placeholder_cells (AC-MC-11)
    - two_active_cards_yield_two_filled_and_two_placeholder (AC-MC-12)
    - snapshot_longer_than_max_is_truncated (REQ-MC-021 invariant)
    - status_pill_color_matches_adr_mc_4 (ADR-MC-4)
    - status_pill_label_returns_human_label
    - format_cost_none_returns_empty
    - format_cost_some_returns_dollar_4_decimal
    - default_yields_empty_view_with_default_max_cells
    - set_snapshot_replaces_existing
    - cell_data_is_empty_mirrors_card_presence
  - lib.rs T9:
    - test_mission_control_is_none_on_root_view_new (AC-MC-13)
    - test_ensure_mission_control_creates_entity
    - test_ensure_mission_control_is_idempotent
    - test_dismiss_mission_control_clears_entity
    - test_update_mission_control_snapshot_pushes_cards
    - test_update_mission_control_snapshot_noop_when_not_mounted
- moai-studio-ui lib tests: 1246 → 1262 (+16)
- 회귀 0:
  - moai-studio-agent: 123 GREEN
  - moai-studio-terminal: 36 GREEN
  - moai-studio-workspace: 26 GREEN
- clippy 0 warning (manual_div_ceil → div_ceil 정정)
- fmt clean

### Carry to MS-3 (별 PR — 다음 세션 또는 별 SPEC 분리 검토)

- SseIngest::pump_into_registry helper (REQ-MC-030)
- HTTP client 실 hook server subscribe (REQ-MC-031, USER-DECISION-MC-A)
- Mission Control 진입 트리거 (Command Palette `mission.toggle` entry / 키 바인딩) — MS-2 후속 PR 또는 MS-3 묶음 검토

### DoD ✅

- AgentRunRegistry (MS-1) → MissionControlView (MS-2) 데이터 파이프라인 완성. 사용자가 MS-3 의 hook wire 도입 시 즉시 4-cell grid 가시화.
- 본 PR 완료 시 audit Top 8 #2 E-5 가 logic + render 80% 진행 (남은 20% = MS-3 hook wire + 진입 트리거).
