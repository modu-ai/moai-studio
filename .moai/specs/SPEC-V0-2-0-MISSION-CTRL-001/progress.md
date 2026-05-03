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

### Carry to MS-2 (별 PR — 다음 세션)

- MissionControlView GPUI Entity (REQ-MC-020 ~ REQ-MC-024)
- RootView 의 mission_control field
- 4-cell grid render

### Carry to MS-3 (별 PR — 다음 세션 또는 별 SPEC 분리 검토)

- SseIngest::pump_into_registry helper (REQ-MC-030)
- HTTP client 실 hook server subscribe (REQ-MC-031, USER-DECISION-MC-A)
