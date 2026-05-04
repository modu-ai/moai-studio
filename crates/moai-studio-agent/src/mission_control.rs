//! Mission Control domain — per-run summary registry for the parallel-agents grid.
//!
//! SPEC-V0-2-0-MISSION-CTRL-001 MS-1 (audit Top 8 #2, v0.2.0 cycle Sprint 8).
//!
//! Provides `AgentCard` (per-run summary cache) and `AgentRunRegistry`
//! (HashMap container) that the future `MissionControlView` (MS-2) will read
//! to render a 4-cell grid of active runs. SseIngest will feed events here in
//! MS-3 (carry to a separate PR).
//!
//! V3-010 reuse:
//! - `AgentRunId` / `AgentRunStatus` from `events.rs` (single source of truth)
//! - `AgentEvent` / `EventKind` / `HookEvent` from `events.rs`
//! - `CostSnapshot` from `cost.rs`
//!
//! Frozen zone (REQ-MC-040 ~ REQ-MC-042):
//! - moai-studio-terminal/** unchanged
//! - existing public API of events.rs / ring_buffer.rs / cost.rs unchanged

use std::collections::HashMap;
use std::time::SystemTime;

use crate::cost::CostSnapshot;
use crate::events::{AgentEvent, AgentRunId, AgentRunStatus, EventKind};

// ============================================================
// AgentCard — per-run summary cache (REQ-MC-001 ~ REQ-MC-003)
// ============================================================

/// Per-run summary card surfaced in the Mission Control grid.
///
/// @MX:NOTE: [AUTO] agent-card-summary
/// @MX:SPEC: SPEC-V0-2-0-MISSION-CTRL-001 REQ-MC-001
/// `AgentCard` is intentionally a small `Clone` value so the Mission Control
/// render path can snapshot the registry cheaply. PartialEq is not derived
/// because the optional `cost: CostSnapshot` from V3-010 does not implement
/// PartialEq and this SPEC must not modify the V3-010 frozen zone.
#[derive(Debug, Clone)]
pub struct AgentCard {
    /// Single source of truth identifier (V3-010 carry).
    pub run_id: AgentRunId,
    /// Human-readable label shown in the card header.
    pub label: String,
    /// Lifecycle state — drives the status pill colour.
    pub status: AgentRunStatus,
    /// Last event one-line preview (truncated to 120 chars in the renderer).
    pub last_event_summary: String,
    /// Unix nanoseconds at last update (None until the first push_event).
    pub last_event_at: Option<u128>,
    /// Most recent cost roll-up — None until the supervisor self-reports.
    pub cost: Option<CostSnapshot>,
    /// Total events seen for this run (monotonically increasing).
    pub event_count: u64,
}

impl AgentCard {
    /// Build a fresh card with default-Running state and empty history.
    /// REQ-MC-002.
    pub fn new(run_id: AgentRunId, label: impl Into<String>) -> Self {
        Self {
            run_id,
            label: label.into(),
            status: AgentRunStatus::Running,
            last_event_summary: String::new(),
            last_event_at: None,
            cost: None,
            event_count: 0,
        }
    }
}

// ============================================================
// AgentRunRegistry — HashMap of AgentRunId → AgentCard (REQ-MC-010 ~ REQ-MC-018)
// ============================================================

/// In-memory registry of per-run summaries.
///
/// @MX:ANCHOR: [AUTO] agent-run-registry
/// @MX:REASON: [AUTO] SPEC-V0-2-0-MISSION-CTRL-001 REQ-MC-010. Single owner of
///   the AgentCard map. fan_in >= 3: push_event (hook ingestion), top_n_active
///   (MissionControlView render source), clear_terminal (user dispatch).
/// @MX:SPEC: SPEC-V0-2-0-MISSION-CTRL-001
#[derive(Debug, Default, Clone)]
pub struct AgentRunRegistry {
    cards: HashMap<AgentRunId, AgentCard>,
}

impl AgentRunRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or update the label of a run while preserving its history.
    /// REQ-MC-011.
    pub fn register(&mut self, run_id: AgentRunId, label: impl Into<String>) {
        let label = label.into();
        self.cards
            .entry(run_id.clone())
            .and_modify(|c| c.label = label.clone())
            .or_insert_with(|| AgentCard::new(run_id, label));
    }

    /// Push an event into the registry, creating the card on demand.
    ///
    /// REQ-MC-012: auto-register if absent (label = id Display).
    /// REQ-MC-013: status auto-transition for SessionStart / Stop /
    ///   Notification(severity=error). Other event names preserve status.
    pub fn push_event(&mut self, run_id: &AgentRunId, event: &AgentEvent) {
        let summary = event_summary(event);
        let timestamp = if event.timestamp_ns == 0 {
            now_unix_ns()
        } else {
            event.timestamp_ns
        };
        let transition = derive_status_transition(event);

        let card = self.cards.entry(run_id.clone()).or_insert_with(|| {
            // REQ-MC-012: auto-register with id Display as label.
            AgentCard::new(run_id.clone(), run_id.to_string())
        });
        card.event_count = card.event_count.saturating_add(1);
        card.last_event_summary = summary;
        card.last_event_at = Some(timestamp);
        if let Some(next) = transition {
            card.status = next;
        }
    }

    /// Explicit status override (REQ-MC-014). Bypasses transition rules.
    pub fn set_status(&mut self, run_id: &AgentRunId, status: AgentRunStatus) {
        if let Some(card) = self.cards.get_mut(run_id) {
            card.status = status;
        }
    }

    /// Update the cost snapshot of an existing card. No-op when the card is
    /// absent (cost arrives independently of run lifecycle).
    /// REQ-MC-015.
    pub fn set_cost(&mut self, run_id: &AgentRunId, cost: CostSnapshot) {
        if let Some(card) = self.cards.get_mut(run_id) {
            card.cost = Some(cost);
        }
    }

    /// Iterate all cards in arbitrary order (HashMap iteration order).
    /// REQ-MC-016.
    pub fn cards(&self) -> impl Iterator<Item = &AgentCard> {
        self.cards.values()
    }

    /// Lookup a card by id.
    pub fn get(&self, run_id: &AgentRunId) -> Option<&AgentCard> {
        self.cards.get(run_id)
    }

    /// Number of cards currently registered (active + terminal).
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// True when no card has been registered.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Top-N active (Running / Paused) cards sorted by last_event_at desc.
    /// Cards without a last_event_at sort to the end (treated as oldest).
    /// REQ-MC-017.
    pub fn top_n_active(&self, n: usize) -> Vec<&AgentCard> {
        let mut active: Vec<&AgentCard> = self
            .cards
            .values()
            .filter(|c| !c.status.is_terminal())
            .collect();
        active.sort_by(|a, b| {
            // None goes to the end → reverse Option<u128> ordering trick.
            match (b.last_event_at, a.last_event_at) {
                (Some(b_t), Some(a_t)) => b_t.cmp(&a_t),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        active.truncate(n);
        active
    }

    /// Drop terminal cards (Completed / Failed / Killed). REQ-MC-018.
    pub fn clear_terminal(&mut self) {
        self.cards.retain(|_, c| !c.status.is_terminal());
    }
}

// ============================================================
// Helpers — event summary + status transition rules
// ============================================================

/// Build a one-line preview from an `AgentEvent` (truncated to 120 chars).
fn event_summary(event: &AgentEvent) -> String {
    let raw = match &event.kind {
        EventKind::StreamJson(s) => format!("[{}] {}", s.type_, s.payload),
        EventKind::Hook(h) => format!("hook:{} {}", h.event_name, h.payload),
        EventKind::Unknown(v) => format!("unknown {}", v),
    };
    truncate_chars(&raw, 120)
}

/// Truncate `s` to at most `max` characters (not bytes), appending "…" when
/// truncated.
fn truncate_chars(s: &str, max: usize) -> String {
    let mut out = String::with_capacity(max + 1);
    for (count, c) in s.chars().enumerate() {
        if count >= max {
            out.push('…');
            return out;
        }
        out.push(c);
    }
    out
}

/// Derive the next `AgentRunStatus` from an event, or None to keep current.
/// REQ-MC-013: SessionStart → Running, Stop → Completed,
/// Notification(severity=error) → Failed. Other events: no transition.
fn derive_status_transition(event: &AgentEvent) -> Option<AgentRunStatus> {
    match &event.kind {
        EventKind::Hook(h) => match h.event_name.as_str() {
            "SessionStart" => Some(AgentRunStatus::Running),
            "Stop" => Some(AgentRunStatus::Completed),
            "Notification" => {
                let severity = h
                    .payload
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if severity.eq_ignore_ascii_case("error") {
                    Some(AgentRunStatus::Failed)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// Returns the current Unix epoch in nanoseconds, or 0 on the (extremely
/// unlikely) clock-before-epoch case.
fn now_unix_ns() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

// ============================================================
// Unit tests — SPEC-V0-2-0-MISSION-CTRL-001 MS-1 (AC-MC-1 ~ AC-MC-10)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::CostSnapshot;
    use crate::events::{HookEvent, StreamJsonEvent};
    use serde_json::json;

    fn mk_id(s: &str) -> AgentRunId {
        AgentRunId(s.to_string())
    }

    fn mk_hook_event(id_seq: u64, name: &str, payload: serde_json::Value) -> AgentEvent {
        AgentEvent::from_hook(
            id_seq,
            format!("{{\"event\":\"{name}\"}}"),
            HookEvent {
                event_name: name.to_string(),
                payload,
            },
        )
    }

    fn mk_stream_event(id_seq: u64, type_: &str) -> AgentEvent {
        AgentEvent::from_stream_json(
            id_seq,
            format!("{{\"type\":\"{type_}\"}}"),
            StreamJsonEvent {
                type_: type_.to_string(),
                payload: json!({"content": "x"}),
                usage: None,
            },
        )
    }

    /// AC-MC-1 (REQ-MC-001/002/003): AgentCard::new initial state.
    #[test]
    fn agent_card_new_initial_state() {
        let card = AgentCard::new(mk_id("run-1"), "Demo");
        assert_eq!(card.run_id, mk_id("run-1"));
        assert_eq!(card.label, "Demo");
        assert_eq!(card.status, AgentRunStatus::Running);
        assert_eq!(card.event_count, 0);
        assert_eq!(card.last_event_at, None);
        assert_eq!(card.last_event_summary, "");
        assert!(card.cost.is_none());
    }

    /// AC-MC-2 (REQ-MC-010/011): register twice updates label, preserves history.
    #[test]
    fn registry_register_twice_updates_label_preserves_history() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-1");
        reg.register(id.clone(), "A");
        // Push an event so event_count becomes 1.
        let ev = mk_hook_event(1, "SessionStart", json!({}));
        reg.push_event(&id, &ev);
        let count_before = reg.get(&id).unwrap().event_count;
        assert_eq!(count_before, 1);

        // Re-register with a new label.
        reg.register(id.clone(), "B");
        let card = reg.get(&id).unwrap();
        assert_eq!(card.label, "B");
        assert_eq!(
            card.event_count, count_before,
            "event_count must be preserved"
        );
        assert_eq!(reg.len(), 1);
    }

    /// AC-MC-3 (REQ-MC-012): push_event auto-registers + updates summary/timestamp/count.
    #[test]
    fn push_event_auto_registers_and_updates_summary() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-fresh");
        let ev = mk_hook_event(1, "SessionStart", json!({"session": "abc"}));
        reg.push_event(&id, &ev);

        let card = reg.get(&id).expect("must auto-register");
        assert_eq!(card.event_count, 1);
        assert!(card.last_event_at.is_some());
        assert!(
            !card.last_event_summary.is_empty(),
            "summary must not be empty after push"
        );
        // Default label is the id Display.
        assert_eq!(card.label, id.to_string());
    }

    /// AC-MC-4 (REQ-MC-013): hook Stop transitions status to Completed.
    #[test]
    fn push_event_stop_transitions_to_completed() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-stop");
        reg.register(id.clone(), "Demo");
        let ev = mk_hook_event(1, "Stop", json!({}));
        reg.push_event(&id, &ev);
        assert_eq!(reg.get(&id).unwrap().status, AgentRunStatus::Completed);
    }

    /// AC-MC-5 (REQ-MC-013): hook Notification(severity=error) → Failed.
    #[test]
    fn push_event_notification_error_transitions_to_failed() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-fail");
        reg.register(id.clone(), "Demo");
        let ev = mk_hook_event(1, "Notification", json!({"severity": "error"}));
        reg.push_event(&id, &ev);
        assert_eq!(reg.get(&id).unwrap().status, AgentRunStatus::Failed);
    }

    /// REQ-MC-013 negative: Notification(severity=info) does NOT transition.
    #[test]
    fn push_event_notification_info_does_not_transition() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-info");
        reg.register(id.clone(), "Demo");
        let ev = mk_hook_event(1, "Notification", json!({"severity": "info"}));
        reg.push_event(&id, &ev);
        assert_eq!(reg.get(&id).unwrap().status, AgentRunStatus::Running);
    }

    /// REQ-MC-013 negative: arbitrary hook event preserves status.
    #[test]
    fn push_event_arbitrary_hook_preserves_status() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-pretool");
        reg.register(id.clone(), "Demo");
        let ev = mk_hook_event(1, "PreToolUse", json!({"tool_name": "Bash"}));
        reg.push_event(&id, &ev);
        assert_eq!(reg.get(&id).unwrap().status, AgentRunStatus::Running);
        assert_eq!(reg.get(&id).unwrap().event_count, 1);
    }

    /// REQ-MC-013: stream-json events do not transition status.
    #[test]
    fn push_event_stream_json_does_not_transition() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-stream");
        reg.register(id.clone(), "Demo");
        let ev = mk_stream_event(1, "assistant");
        reg.push_event(&id, &ev);
        assert_eq!(reg.get(&id).unwrap().status, AgentRunStatus::Running);
        assert_eq!(reg.get(&id).unwrap().event_count, 1);
    }

    /// AC-MC-6 (REQ-MC-014): set_status overrides regardless of transition rules.
    #[test]
    fn set_status_overrides_directly() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-kill");
        reg.register(id.clone(), "Demo");
        reg.set_status(&id, AgentRunStatus::Killed);
        assert_eq!(reg.get(&id).unwrap().status, AgentRunStatus::Killed);
    }

    /// REQ-MC-014: set_status on unknown id is a no-op (no panic).
    #[test]
    fn set_status_unknown_id_is_noop() {
        let mut reg = AgentRunRegistry::new();
        reg.set_status(&mk_id("ghost"), AgentRunStatus::Failed);
        assert!(reg.is_empty());
    }

    /// AC-MC-7 (REQ-MC-015): set_cost stores a snapshot.
    #[test]
    fn set_cost_stores_snapshot() {
        let mut reg = AgentRunRegistry::new();
        let id = mk_id("run-cost");
        reg.register(id.clone(), "Demo");
        let snapshot = CostSnapshot {
            timestamp: SystemTime::UNIX_EPOCH,
            usd: 0.0001,
            run_id: id.clone(),
        };
        reg.set_cost(&id, snapshot);
        let card_cost = reg.get(&id).and_then(|c| c.cost.clone()).unwrap();
        assert!((card_cost.usd - 0.0001).abs() < f64::EPSILON);
        assert_eq!(card_cost.run_id, id);
    }

    /// REQ-MC-015: set_cost on unknown id is a no-op.
    #[test]
    fn set_cost_unknown_id_is_noop() {
        let mut reg = AgentRunRegistry::new();
        let snapshot = CostSnapshot {
            timestamp: SystemTime::UNIX_EPOCH,
            usd: 0.5,
            run_id: mk_id("ghost"),
        };
        reg.set_cost(&mk_id("ghost"), snapshot);
        assert!(reg.is_empty());
    }

    /// AC-MC-8 (REQ-MC-017): top_n_active filters terminal + sorts by last_event_at desc.
    #[test]
    fn top_n_active_filters_terminal_and_sorts_recency() {
        let mut reg = AgentRunRegistry::new();
        // 5 cards: 3 Running (with varied last_event_at) + 1 Completed + 1 Failed.
        let id1 = mk_id("r1");
        let id2 = mk_id("r2");
        let id3 = mk_id("r3");
        let id_done = mk_id("done");
        let id_fail = mk_id("fail");
        for (i, (id, _)) in [(&id1, 100), (&id2, 200), (&id3, 300)].iter().enumerate() {
            reg.register((*id).clone(), format!("R{i}"));
            let mut ev = mk_hook_event(i as u64, "PreToolUse", json!({}));
            ev.timestamp_ns = match id.0.as_str() {
                "r1" => 100,
                "r2" => 200,
                "r3" => 300,
                _ => 0,
            };
            reg.push_event(id, &ev);
        }
        reg.register(id_done.clone(), "Done");
        reg.set_status(&id_done, AgentRunStatus::Completed);
        reg.register(id_fail.clone(), "Fail");
        reg.set_status(&id_fail, AgentRunStatus::Failed);

        let top = reg.top_n_active(4);
        assert_eq!(top.len(), 3, "only Running cards must be returned");
        assert_eq!(top[0].run_id, id3, "most recent first");
        assert_eq!(top[1].run_id, id2);
        assert_eq!(top[2].run_id, id1);
    }

    /// REQ-MC-017: top_n_active respects the limit n.
    #[test]
    fn top_n_active_respects_limit() {
        let mut reg = AgentRunRegistry::new();
        for i in 0..6u64 {
            let id = mk_id(&format!("run-{i}"));
            reg.register(id.clone(), format!("R{i}"));
            let mut ev = mk_hook_event(i, "PreToolUse", json!({}));
            ev.timestamp_ns = 1000 + i as u128;
            reg.push_event(&id, &ev);
        }
        let top = reg.top_n_active(4);
        assert_eq!(top.len(), 4);
    }

    /// AC-MC-9 (REQ-MC-018): clear_terminal removes terminal cards only.
    #[test]
    fn clear_terminal_removes_terminal_cards_only() {
        let mut reg = AgentRunRegistry::new();
        let id_run1 = mk_id("run1");
        let id_run2 = mk_id("run2");
        let id_done1 = mk_id("done1");
        let id_done2 = mk_id("done2");
        reg.register(id_run1.clone(), "Run1");
        reg.register(id_run2.clone(), "Run2");
        reg.register(id_done1.clone(), "Done1");
        reg.set_status(&id_done1, AgentRunStatus::Completed);
        reg.register(id_done2.clone(), "Done2");
        reg.set_status(&id_done2, AgentRunStatus::Killed);

        reg.clear_terminal();
        assert_eq!(reg.len(), 2, "only the 2 Running cards must remain");
        assert!(reg.get(&id_run1).is_some());
        assert!(reg.get(&id_run2).is_some());
        assert!(reg.get(&id_done1).is_none());
        assert!(reg.get(&id_done2).is_none());
    }

    /// AC-MC-10 (REQ-MC-016): cards iterator + lookup.
    #[test]
    fn cards_iterator_and_lookup() {
        let mut reg = AgentRunRegistry::new();
        for i in 0..3u64 {
            let id = mk_id(&format!("c{i}"));
            reg.register(id.clone(), format!("Card{i}"));
        }
        assert_eq!(reg.cards().count(), 3);
        for i in 0..3u64 {
            let id = mk_id(&format!("c{i}"));
            assert!(reg.get(&id).is_some());
        }
        assert_eq!(reg.len(), 3);
        assert!(!reg.is_empty());
    }

    /// truncate_chars boundary: short string is unchanged.
    #[test]
    fn truncate_chars_short_string_unchanged() {
        let s = "hello";
        assert_eq!(truncate_chars(s, 120), "hello");
    }

    /// truncate_chars boundary: long string gets ellipsis.
    #[test]
    fn truncate_chars_long_string_ellipsis() {
        let s = "a".repeat(200);
        let out = truncate_chars(&s, 10);
        // 10 chars + "…"
        assert!(out.ends_with('…'));
        assert_eq!(out.chars().count(), 11);
    }
}
