//! EventTimelineView + AgentDashboardView 골격 (RG-AD-2 minimal, REQ-AD-008)
//!
//! MS-1 에서는 ring buffer events 의 count + 최근 10 개 이벤트 타입을 텍스트로 표시.
//! filter chip / cost panel / instructions graph 는 MS-2/3 에서 추가한다.
//!
//! GPUI Entity 패턴은 SPEC-V3-004 / SPEC-V3-005 의 FileExplorer 를 참조한다.

// @MX:ANCHOR: [AUTO] timeline-view-entity
// @MX:REASON: [AUTO] agent surface 진입점. fan_in >= 3:
//   AgentDashboardView, StreamIngestor channel consumer, future filter layer.
//   SPEC: SPEC-V3-010 REQ-AD-008

/// EventTimelineView — ring buffer 이벤트를 시간 역순으로 렌더 (REQ-AD-008).
///
/// MS-1: event count + 최근 10 개 이벤트 type 문자열 목록.
/// MS-2/3: filter chip, 60fps throttle, detail panel 연동.
pub struct EventTimelineView {
    /// 표시할 이벤트 목록 (ring buffer 의 스냅샷).
    /// MS-1 에서는 view 가 직접 보유. MS-2 에서 shared state 로 이관.
    pub events: Vec<crate::events::AgentEvent>,
}

impl EventTimelineView {
    /// 새 EventTimelineView 를 생성한다.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// 새 이벤트를 push 한다.
    /// GPUI 환경에서는 `cx.notify()` 를 호출하여 렌더를 트리거해야 한다.
    pub fn push_event(&mut self, ev: crate::events::AgentEvent) {
        self.events.push(ev);
    }

    /// 이벤트 수를 반환한다.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// 최근 N 개 이벤트의 kind 문자열 목록 (역순, 최신 → 오래됨).
    pub fn recent_event_kinds(&self, n: usize) -> Vec<String> {
        self.events
            .iter()
            .rev()
            .take(n)
            .map(|ev| match &ev.kind {
                crate::events::EventKind::StreamJson(s) => {
                    format!("stream:{}", s.type_)
                }
                crate::events::EventKind::Hook(h) => {
                    format!("hook:{}", h.event_name)
                }
                crate::events::EventKind::Unknown(_) => "unknown".to_string(),
            })
            .collect()
    }
}

impl Default for EventTimelineView {
    fn default() -> Self {
        Self::new()
    }
}

// @MX:TODO(MS-2-cost-panel): CostPanelView 추가 (MS-2 담당)
// @MX:TODO(MS-3-instructions-graph): InstructionsGraphView 추가 (MS-3 담당)

/// AgentDashboardView — 전체 agent dashboard 컨테이너 골격 (MS-1 stub).
///
/// MS-1: timeline 만 보유. split layout + cost / instructions / control panel 은 MS-2/3 에서 추가.
pub struct AgentDashboardView {
    /// timeline view (ms-1 에서는 직접 보유).
    pub timeline: EventTimelineView,
}

impl AgentDashboardView {
    /// 새 AgentDashboardView 를 생성한다.
    pub fn new() -> Self {
        Self {
            timeline: EventTimelineView::new(),
        }
    }
}

impl Default for AgentDashboardView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{AgentEvent, HookEvent};

    fn make_hook_event(id: u64, name: &str) -> AgentEvent {
        AgentEvent::from_hook(
            id,
            format!("raw-{}", name),
            HookEvent {
                event_name: name.to_string(),
                payload: serde_json::json!({}),
            },
        )
    }

    /// EventTimelineView 생성 테스트.
    #[test]
    fn timeline_view_can_be_created() {
        let view = EventTimelineView::new();
        assert_eq!(view.event_count(), 0);
        assert!(view.events.is_empty());
    }

    /// push_event 후 event_count 증가 확인.
    #[test]
    fn timeline_view_push_event_increments_count() {
        let mut view = EventTimelineView::new();
        view.push_event(make_hook_event(0, "SessionStart"));
        assert_eq!(view.event_count(), 1);

        view.push_event(make_hook_event(1, "PostToolUse"));
        assert_eq!(view.event_count(), 2);
    }

    /// recent_event_kinds 는 최신 우선으로 반환해야 한다.
    #[test]
    fn recent_event_kinds_returns_newest_first() {
        let mut view = EventTimelineView::new();
        for i in 0..5u64 {
            view.push_event(make_hook_event(i, &format!("Hook{}", i)));
        }
        let kinds = view.recent_event_kinds(3);
        assert_eq!(kinds.len(), 3);
        // 가장 최근 (i=4) 이 첫 번째여야 한다
        assert!(kinds[0].contains("Hook4"), "kinds[0]={}", kinds[0]);
        assert!(kinds[1].contains("Hook3"), "kinds[1]={}", kinds[1]);
        assert!(kinds[2].contains("Hook2"), "kinds[2]={}", kinds[2]);
    }

    /// AgentDashboardView 생성 + timeline child 확인.
    #[test]
    fn dashboard_view_creates_with_timeline_child() {
        let dashboard = AgentDashboardView::new();
        assert_eq!(dashboard.timeline.event_count(), 0);
    }
}
