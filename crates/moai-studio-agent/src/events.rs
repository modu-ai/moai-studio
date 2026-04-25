//! AgentEvent 도메인 타입 정의 (RG-AD-1, AC-AD-1)
//!
//! SPEC-V3-010 REQ-AD-001: AgentRunId — 단일 소스 of truth
//! SPEC-V3-010 REQ-AD-005: Unknown fallback — panic 방지

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::SystemTime;

// @MX:ANCHOR: [AUTO] agent-event-domain
// @MX:REASON: [AUTO] 이벤트 도메인 단일 소스. fan_in >= 4:
//   stream_ingest, sse_ingest, EventTimelineView, future CostPanel.
//   SPEC: SPEC-V3-010 REQ-AD-002/003/005

// @MX:NOTE: [AUTO] agentrunid-single-source
// @MX:SPEC: SPEC-V3-010 REQ-AD-001
// V3-009 충돌 방지 — AgentRunId 는 이 crate 가 single source of truth.
// V3-009 의 MoaiCommandClient::spawn 이 이 ID 를 소비한다.

/// Agent run 고유 식별자 — SPEC-V3-009 와 공유 (REQ-AD-001).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentRunId(pub String);

impl AgentRunId {
    /// 새 고유 ID 를 생성한다.
    pub fn new_unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("run-{:x}-{:x}", nanos, seq))
    }
}

impl std::fmt::Display for AgentRunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// stream-json 메시지 wrapper (REQ-AD-002).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamJsonEvent {
    /// SDKMessage 의 type 필드 (assistant, system, result 등)
    pub type_: String,
    /// 원본 JSON payload
    pub payload: Value,
    /// usage 정보 (B1 self-report, REQ-AD-014)
    pub usage: Option<TokenUsage>,
}

/// hook-http 이벤트 wrapper (REQ-AD-003).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    /// hook event 이름 (SessionStart, PostToolUse 등 27 종)
    pub event_name: String,
    /// hook event payload (JSON)
    pub payload: Value,
}

/// token 사용량 정보 — B1 self-report (REQ-AD-014, USER-DECISION-AD-B).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// API self-report USD 비용 (있을 경우, REQ-AD-014)
    pub cost_usd: Option<f64>,
}

/// 이벤트 종류 분류 (REQ-AD-002/003/005).
// @MX:NOTE: [AUTO] unknown-kind-fallback
// @MX:SPEC: SPEC-V3-010 REQ-AD-005
// 알려지지 않은 stream/hook kind 에 panic 하지 않는다. Unknown 으로 fallback.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EventKind {
    /// stream-json 한 줄로 파싱된 이벤트 (REQ-AD-002)
    StreamJson(StreamJsonEvent),
    /// hook-http SSE 로 수신된 hook 이벤트 (REQ-AD-003)
    Hook(HookEvent),
    /// 알 수 없는 종류 — panic fallback (REQ-AD-005)
    Unknown(Value),
}

/// Agent 이벤트 — ring buffer 의 저장 단위 (AC-AD-1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    /// 단조 증가 순서 ID
    pub id: u64,
    /// 이벤트 발생 시각 (Unix nanoseconds)
    pub timestamp_ns: u128,
    /// 이벤트 종류
    pub kind: EventKind,
    /// 원본 raw 문자열 (디버그/detail view 용)
    pub raw: String,
}

impl AgentEvent {
    /// stream-json 줄로 AgentEvent 를 생성한다.
    pub fn from_stream_json(id: u64, raw: String, ev: StreamJsonEvent) -> Self {
        Self {
            id,
            timestamp_ns: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
            kind: EventKind::StreamJson(ev),
            raw,
        }
    }

    /// hook 이벤트로 AgentEvent 를 생성한다.
    pub fn from_hook(id: u64, raw: String, ev: HookEvent) -> Self {
        Self {
            id,
            timestamp_ns: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
            kind: EventKind::Hook(ev),
            raw,
        }
    }

    /// 알 수 없는 raw 데이터로 Unknown AgentEvent 를 생성한다 (REQ-AD-005).
    pub fn unknown(id: u64, raw: String) -> Self {
        let val = serde_json::from_str::<Value>(&raw).unwrap_or(Value::String(raw.clone()));
        Self {
            id,
            timestamp_ns: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
            kind: EventKind::Unknown(val),
            raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-AD-1: AgentEvent serde round-trip 검증
    #[test]
    fn agent_event_round_trip_serde() {
        let ev = AgentEvent {
            id: 1,
            timestamp_ns: 12345678,
            kind: EventKind::StreamJson(StreamJsonEvent {
                type_: "assistant".to_string(),
                payload: serde_json::json!({"content": "hello"}),
                usage: Some(TokenUsage {
                    input_tokens: 10,
                    output_tokens: 5,
                    cost_usd: Some(0.0001),
                }),
            }),
            raw: r#"{"type":"assistant"}"#.to_string(),
        };

        let json = serde_json::to_string(&ev).expect("직렬화 실패");
        let decoded: AgentEvent = serde_json::from_str(&json).expect("역직렬화 실패");

        assert_eq!(decoded.id, ev.id);
        assert_eq!(decoded.timestamp_ns, ev.timestamp_ns);
        assert_eq!(decoded.raw, ev.raw);

        match decoded.kind {
            EventKind::StreamJson(s) => {
                assert_eq!(s.type_, "assistant");
                let usage = s.usage.expect("usage 누락");
                assert_eq!(usage.input_tokens, 10);
            }
            other => panic!("예상치 못한 kind: {:?}", other),
        }
    }

    /// AC-AD-1: Unknown kind fallback — panic 하지 않아야 한다 (REQ-AD-005)
    #[test]
    fn unknown_kind_fallback() {
        let ev = AgentEvent::unknown(0, "not valid json at all !!!".to_string());
        match ev.kind {
            EventKind::Unknown(_) => {} // 정상
            other => panic!("Unknown fallback 실패: {:?}", other),
        }
    }

    #[test]
    fn agent_run_id_is_unique() {
        let a = AgentRunId::new_unique();
        let b = AgentRunId::new_unique();
        assert_ne!(a, b, "AgentRunId 는 항상 고유해야 한다");
    }
}
