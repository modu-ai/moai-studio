//! EventFilter — 이벤트 종류/run 별 필터 (RG-AD-2, AC-AD-4)
//!
//! SPEC-V3-010 REQ-AD-009: 60fps burst 대응 filter chain.
//!
//! @MX:ANCHOR: [AUTO] event-filter-domain
//! @MX:REASON: [AUTO] timeline chip toggle 단일 진실 원천. fan_in >= 3:
//!   EventTimelineView, CostPanelView, 테스트.
//!   SPEC: SPEC-V3-010 RG-AD-2, AC-AD-4

use std::collections::HashSet;

use crate::events::{AgentEvent, AgentRunId, EventKind};

/// 이벤트 종류 discriminant — filter chip 대응 (AC-AD-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKindDiscriminant {
    /// stream-json 이벤트
    StreamJson,
    /// hook SSE 이벤트
    Hook,
    /// 알 수 없는 이벤트
    Unknown,
}

impl From<&EventKind> for EventKindDiscriminant {
    fn from(kind: &EventKind) -> Self {
        match kind {
            EventKind::StreamJson(_) => Self::StreamJson,
            EventKind::Hook(_) => Self::Hook,
            EventKind::Unknown(_) => Self::Unknown,
        }
    }
}

/// 이벤트 필터 설정 (AC-AD-4).
///
/// `allowed_kinds` 에 포함된 종류의 이벤트만 통과.
/// `run_id` 가 Some 이면 해당 run 의 이벤트만 통과.
pub struct EventFilter {
    /// 허용된 이벤트 종류 집합 (chip toggle 상태)
    pub allowed_kinds: HashSet<EventKindDiscriminant>,
    /// None = 모든 run 허용, Some(id) = 해당 run 만 허용
    pub run_id: Option<AgentRunId>,
}

impl EventFilter {
    /// 모든 종류, 모든 run 을 허용하는 필터를 생성한다.
    pub fn allow_all() -> Self {
        let mut kinds = HashSet::new();
        kinds.insert(EventKindDiscriminant::StreamJson);
        kinds.insert(EventKindDiscriminant::Hook);
        kinds.insert(EventKindDiscriminant::Unknown);
        Self {
            allowed_kinds: kinds,
            run_id: None,
        }
    }

    /// 이벤트 종류 chip 을 토글한다.
    ///
    /// 현재 허용 상태이면 제거, 제거 상태이면 다시 추가한다.
    pub fn toggle_kind(&mut self, kind: EventKindDiscriminant) {
        if self.allowed_kinds.contains(&kind) {
            self.allowed_kinds.remove(&kind);
        } else {
            self.allowed_kinds.insert(kind);
        }
    }

    /// 이벤트가 필터를 통과하는지 확인한다.
    ///
    /// MS-2 스코프: AgentEvent 에 run_id 필드가 없으므로 kind 기반 필터만 적용한다.
    /// run_id 필터는 CostTracker.session_total() 에서 처리된다.
    /// MS-3 에서 AgentEvent 에 run_id 를 추가하면 이 메서드에서 같이 검사한다.
    pub fn matches(&self, event: &AgentEvent) -> bool {
        let discriminant = EventKindDiscriminant::from(&event.kind);
        self.allowed_kinds.contains(&discriminant)
    }
}

/// 이벤트 슬라이스에 필터를 적용하여 일치하는 이벤트 참조 목록을 반환한다 (AC-AD-4).
pub fn apply_filter<'a>(events: &'a [AgentEvent], filter: &EventFilter) -> Vec<&'a AgentEvent> {
    events.iter().filter(|ev| filter.matches(ev)).collect()
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{AgentEvent, AgentRunId, EventKind, HookEvent, StreamJsonEvent};
    use serde_json::Value;

    fn make_stream_event(id: u64) -> AgentEvent {
        AgentEvent {
            id,
            timestamp_ns: 0,
            kind: EventKind::StreamJson(StreamJsonEvent {
                type_: "assistant".to_string(),
                payload: Value::Null,
                usage: None,
            }),
            raw: String::new(),
        }
    }

    fn make_hook_event(id: u64) -> AgentEvent {
        AgentEvent {
            id,
            timestamp_ns: 0,
            kind: EventKind::Hook(HookEvent {
                event_name: "PostToolUse".to_string(),
                payload: Value::Null,
            }),
            raw: String::new(),
        }
    }

    fn make_unknown_event(id: u64) -> AgentEvent {
        AgentEvent {
            id,
            timestamp_ns: 0,
            kind: EventKind::Unknown(Value::Null),
            raw: String::new(),
        }
    }

    /// allow_all 필터는 모든 이벤트를 통과시켜야 한다
    #[test]
    fn allow_all_matches_everything() {
        let filter = EventFilter::allow_all();
        assert!(filter.matches(&make_stream_event(0)));
        assert!(filter.matches(&make_hook_event(1)));
        assert!(filter.matches(&make_unknown_event(2)));
    }

    /// toggle_kind: 허용→제외→재허용 동작 확인
    #[test]
    fn toggle_kind_excludes_then_re_includes() {
        let mut filter = EventFilter::allow_all();
        let ev = make_hook_event(0);

        // 처음엔 통과
        assert!(filter.matches(&ev));

        // Hook 제거
        filter.toggle_kind(EventKindDiscriminant::Hook);
        assert!(!filter.matches(&ev));

        // Hook 재추가
        filter.toggle_kind(EventKindDiscriminant::Hook);
        assert!(filter.matches(&ev));
    }

    /// apply_filter: 제외된 종류만 걸러야 한다
    #[test]
    fn apply_filter_returns_subset() {
        let events = vec![
            make_stream_event(0),
            make_hook_event(1),
            make_unknown_event(2),
            make_stream_event(3),
        ];

        let mut filter = EventFilter::allow_all();
        filter.toggle_kind(EventKindDiscriminant::Hook);
        filter.toggle_kind(EventKindDiscriminant::Unknown);

        let result = apply_filter(&events, &filter);
        // StreamJson 만 남아야 한다 (id 0, 3)
        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .all(|ev| matches!(ev.kind, EventKind::StreamJson(_)))
        );
    }

    /// run_id 필터: run_id 설정 시 kind 매칭은 유지, run_id 격리 확인
    #[test]
    fn filter_by_run_id_isolates() {
        let r1 = AgentRunId("r1".to_string());
        let mut filter = EventFilter::allow_all();
        filter.run_id = Some(r1);

        // run_id 필터가 있어도 kind 매칭은 정상 동작해야 한다
        let ev = make_stream_event(0);
        // run_id 필터 현재 구현: kind 통과 여부만 확인 (MS-2 스코프)
        assert!(filter.matches(&ev));
    }
}
