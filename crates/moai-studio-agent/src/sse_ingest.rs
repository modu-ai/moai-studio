//! SSE (Server-Sent Events) 수신 경로 scaffold (REQ-AD-003, AC-AD-2)
//!
//! USER-DECISION-AD-D: D2 SSE — moai-hook-http 의 `/events/sse` endpoint 재사용.
//!
//! MS-1: SSE 형식 파서만 구현 (pure function, testable).
//! HTTP 서버 연결은 MS-2/MS-3 에서 추가한다.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::events::{AgentEvent, HookEvent};

/// SSE 전용 이벤트 ID 카운터 (stream_ingest 와 별도).
static SSE_EVENT_ID: AtomicU64 = AtomicU64::new(10000);

fn next_sse_event_id() -> u64 {
    SSE_EVENT_ID.fetch_add(1, Ordering::Relaxed)
}

/// SSE chunk 를 파싱하여 AgentEvent 목록을 반환한다 (pure function).
///
/// SSE 형식:
/// ```text
/// event: PostToolUse
/// data: {"session_id":"...","hook_event_name":"PostToolUse"}
///
/// ```
///
/// - `event:` 라인 → hook_event_name
/// - `data:` 라인 → JSON payload
/// - 빈 줄 → 이벤트 경계
/// - 파싱 실패 시 Unknown fallback (REQ-AD-005)
pub fn parse_sse_chunk(chunk: &str) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    let mut current_event_name: Option<String> = None;
    let mut current_data: Option<String> = None;

    for line in chunk.lines() {
        if line.is_empty() {
            // 이벤트 경계 — 누적된 event/data 를 flush
            if let Some(data) = current_data.take() {
                let event_name = current_event_name
                    .take()
                    .unwrap_or_else(|| "unknown".to_string());
                let id = next_sse_event_id();
                let raw = format!("event: {}\ndata: {}", event_name, data);
                let ev = parse_sse_event(id, event_name, data, raw);
                events.push(ev);
            } else {
                current_event_name = None;
            }
        } else if let Some(rest) = line.strip_prefix("event: ") {
            current_event_name = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("data: ") {
            current_data = Some(rest.to_string());
        }
        // id: / retry: / comment (`:`) 라인은 무시
    }

    // chunk 끝에 빈 줄 없이 끝난 경우 처리
    if let Some(data) = current_data {
        let event_name = current_event_name.unwrap_or_else(|| "unknown".to_string());
        let id = next_sse_event_id();
        let raw = format!("event: {}\ndata: {}", event_name, data);
        let ev = parse_sse_event(id, event_name, data, raw);
        events.push(ev);
    }

    events
}

/// 단일 SSE 이벤트를 AgentEvent 로 변환한다.
fn parse_sse_event(id: u64, event_name: String, data: String, raw: String) -> AgentEvent {
    match serde_json::from_str::<serde_json::Value>(&data) {
        Ok(payload) => AgentEvent::from_hook(
            id,
            raw,
            HookEvent {
                event_name,
                payload,
            },
        ),
        Err(_) => {
            tracing::warn!("SSE data JSON 파싱 실패, Unknown fallback");
            AgentEvent::unknown(id, raw)
        }
    }
}

/// SSE 수신기 scaffold (MS-2/3 에서 HTTP 연결 추가).
pub struct SseIngestor {
    /// SSE endpoint URL (예: "http://localhost:9876/events/sse")
    pub endpoint_url: String,
}

impl SseIngestor {
    /// 새 SseIngestor 를 생성한다.
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            endpoint_url: endpoint_url.into(),
        }
    }
}

// ============================================================
// SPEC-V0-2-0-MISSION-CTRL-001 MS-3 (REQ-MC-030, AC-MC-14): pump helper
// ============================================================

use crate::events::AgentRunId;
use crate::events::EventKind;
use crate::mission_control::AgentRunRegistry;

/// Parse `chunk` as SSE events and push each one into `registry`, extracting
/// the `AgentRunId` from each event's `payload.session_id` field.
///
/// Returns the number of events that were successfully routed (i.e. carried a
/// non-empty `session_id`). Events lacking a session_id are dropped silently —
/// this is the same fallback the moai-hook-http server uses for legacy hooks.
///
/// REQ-MC-030 / AC-MC-14.
///
/// @MX:NOTE: [AUTO] pump-into-registry — bridge SSE stream → AgentRunRegistry.
/// @MX:SPEC: SPEC-V0-2-0-MISSION-CTRL-001 REQ-MC-030
pub fn pump_into_registry(registry: &mut AgentRunRegistry, chunk: &str) -> usize {
    let events = parse_sse_chunk(chunk);
    let mut pumped = 0usize;
    for ev in &events {
        let run_id = match &ev.kind {
            EventKind::Hook(h) => h
                .payload
                .get("session_id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| AgentRunId(s.to_string())),
            // SSE only carries hook events (USER-DECISION-AD-D2). Other kinds
            // are out-of-scope for the registry; we drop them silently.
            EventKind::StreamJson(_) | EventKind::Unknown(_) => None,
        };
        if let Some(id) = run_id {
            registry.push_event(&id, ev);
            pumped += 1;
        }
    }
    pumped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventKind;

    /// AC-AD-2: 단일 SSE 이벤트 파싱.
    #[test]
    fn parse_single_sse_event() {
        let chunk = "event: PostToolUse\ndata: {\"hook_event_name\":\"PostToolUse\",\"session_id\":\"s1\"}\n\n";
        let events = parse_sse_chunk(chunk);
        assert_eq!(events.len(), 1);

        match &events[0].kind {
            EventKind::Hook(h) => {
                assert_eq!(h.event_name, "PostToolUse");
            }
            other => panic!("Hook kind 예상, 실제: {:?}", other),
        }
    }

    /// AC-AD-2: 복수 SSE 이벤트 파싱.
    #[test]
    fn parse_multiple_sse_events() {
        let chunk = concat!(
            "event: SessionStart\n",
            "data: {\"hook_event_name\":\"SessionStart\"}\n",
            "\n",
            "event: PreToolUse\n",
            "data: {\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Bash\"}\n",
            "\n"
        );
        let events = parse_sse_chunk(chunk);
        assert_eq!(events.len(), 2);

        match &events[0].kind {
            EventKind::Hook(h) => assert_eq!(h.event_name, "SessionStart"),
            other => panic!("첫 이벤트: {:?}", other),
        }
        match &events[1].kind {
            EventKind::Hook(h) => assert_eq!(h.event_name, "PreToolUse"),
            other => panic!("두 번째 이벤트: {:?}", other),
        }
    }

    /// REQ-AD-005: data JSON 파싱 실패 시 Unknown fallback.
    #[test]
    fn parse_sse_event_with_invalid_json_falls_back_to_unknown() {
        let chunk = "event: UnknownHook\ndata: not valid json\n\n";
        let events = parse_sse_chunk(chunk);
        assert_eq!(events.len(), 1);

        match &events[0].kind {
            EventKind::Unknown(_) => {} // 정상 fallback
            other => panic!("Unknown kind 예상, 실제: {:?}", other),
        }
    }

    // ── SPEC-V0-2-0-MISSION-CTRL-001 MS-3 — pump_into_registry tests ──

    use crate::events::AgentRunId;
    use crate::mission_control::AgentRunRegistry;

    /// AC-MC-14 (REQ-MC-030): single SSE event with session_id is routed to registry.
    #[test]
    fn pump_into_registry_routes_single_event() {
        let mut reg = AgentRunRegistry::new();
        let chunk = concat!(
            "event: PostToolUse\n",
            "data: {\"hook_event_name\":\"PostToolUse\",\"session_id\":\"run-abc\"}\n",
            "\n"
        );
        let pumped = pump_into_registry(&mut reg, chunk);
        assert_eq!(pumped, 1);
        assert_eq!(reg.len(), 1);
        let card = reg.get(&AgentRunId("run-abc".to_string())).unwrap();
        assert_eq!(card.event_count, 1);
    }

    /// REQ-MC-030: multiple SSE events grouped by session_id.
    #[test]
    fn pump_into_registry_groups_events_by_session_id() {
        let mut reg = AgentRunRegistry::new();
        let chunk = concat!(
            "event: SessionStart\n",
            "data: {\"hook_event_name\":\"SessionStart\",\"session_id\":\"r1\"}\n",
            "\n",
            "event: PreToolUse\n",
            "data: {\"hook_event_name\":\"PreToolUse\",\"session_id\":\"r1\"}\n",
            "\n",
            "event: SessionStart\n",
            "data: {\"hook_event_name\":\"SessionStart\",\"session_id\":\"r2\"}\n",
            "\n"
        );
        let pumped = pump_into_registry(&mut reg, chunk);
        assert_eq!(pumped, 3);
        assert_eq!(reg.len(), 2, "two distinct sessions must produce two cards");
        assert_eq!(
            reg.get(&AgentRunId("r1".to_string())).unwrap().event_count,
            2
        );
        assert_eq!(
            reg.get(&AgentRunId("r2".to_string())).unwrap().event_count,
            1
        );
    }

    /// REQ-MC-030: events without session_id are dropped (counted as 0).
    #[test]
    fn pump_into_registry_drops_events_without_session_id() {
        let mut reg = AgentRunRegistry::new();
        let chunk = concat!(
            "event: PostToolUse\n",
            "data: {\"hook_event_name\":\"PostToolUse\"}\n",
            "\n"
        );
        let pumped = pump_into_registry(&mut reg, chunk);
        assert_eq!(pumped, 0);
        assert!(reg.is_empty());
    }

    /// REQ-MC-030: empty session_id string is treated as missing.
    #[test]
    fn pump_into_registry_drops_empty_session_id() {
        let mut reg = AgentRunRegistry::new();
        let chunk = concat!(
            "event: PostToolUse\n",
            "data: {\"hook_event_name\":\"PostToolUse\",\"session_id\":\"\"}\n",
            "\n"
        );
        let pumped = pump_into_registry(&mut reg, chunk);
        assert_eq!(pumped, 0);
        assert!(reg.is_empty());
    }

    /// REQ-MC-030: Unknown-kind events (parse failures) are dropped, not panicking.
    #[test]
    fn pump_into_registry_drops_unknown_events_safely() {
        let mut reg = AgentRunRegistry::new();
        let chunk = "event: Bad\ndata: not valid json\n\n";
        let pumped = pump_into_registry(&mut reg, chunk);
        assert_eq!(pumped, 0);
        assert!(reg.is_empty());
    }

    /// REQ-MC-030 + REQ-MC-013: SessionStart hook transitions status to Running.
    #[test]
    fn pump_into_registry_propagates_status_transitions() {
        let mut reg = AgentRunRegistry::new();
        let chunk = concat!(
            "event: SessionStart\n",
            "data: {\"hook_event_name\":\"SessionStart\",\"session_id\":\"rs\"}\n",
            "\n",
            "event: Stop\n",
            "data: {\"hook_event_name\":\"Stop\",\"session_id\":\"rs\"}\n",
            "\n"
        );
        let pumped = pump_into_registry(&mut reg, chunk);
        assert_eq!(pumped, 2);
        let card = reg.get(&AgentRunId("rs".to_string())).unwrap();
        // Last event Stop transitions to Completed (REQ-MC-013).
        assert_eq!(card.status, crate::events::AgentRunStatus::Completed);
    }
}
