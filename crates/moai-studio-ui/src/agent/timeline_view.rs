//! EventTimelineView GPUI Entity (RG-AD-2, REQ-AD-008)
//!
//! MS-1: event count + 최근 10개 이벤트 type 을 div 텍스트로 렌더한다.
//! MS-2: EventFilter chip row + filter 적용 렌더 추가 (AC-AD-4).

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_agent::events::AgentEvent;
use moai_studio_agent::{EventFilter, EventKindDiscriminant, apply_filter};

use crate::design::tokens as tok;

// @MX:ANCHOR: [AUTO] timeline-view-entity-ui
// @MX:REASON: [AUTO] GPUI 렌더 진입점. fan_in >= 3:
//   AgentDashboardView, RootView(future), test code.
//   SPEC: SPEC-V3-010 REQ-AD-008

/// EventTimelineView GPUI Entity — ring buffer 이벤트 목록 렌더 (REQ-AD-008).
///
/// MS-2: filter chip row 를 상단에 표시하고, EventFilter 를 적용하여 이벤트를 렌더한다 (AC-AD-4).
pub struct EventTimelineView {
    /// 이벤트 목록 (ring buffer 스냅샷).
    events: Vec<AgentEvent>,
    /// 이벤트 필터 상태 (chip toggle, AC-AD-4)
    pub filter: EventFilter,
}

impl EventTimelineView {
    /// 새 EventTimelineView 를 생성한다.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            filter: EventFilter::allow_all(),
        }
    }

    /// 새 이벤트를 push 하고 렌더를 트리거한다 (REQ-AD-008).
    pub fn push_event(&mut self, ev: AgentEvent, cx: &mut Context<Self>) {
        self.events.push(ev);
        cx.notify();
    }

    /// 이벤트 수를 반환한다.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// filter chip 을 토글한다 (AC-AD-4).
    ///
    /// 토글 후 cx.notify() 로 렌더를 갱신한다.
    pub fn toggle_filter_chip(&mut self, kind: EventKindDiscriminant, cx: &mut Context<Self>) {
        self.filter.toggle_kind(kind);
        cx.notify();
    }

    /// 현재 필터를 적용한 이벤트 목록을 반환한다.
    fn filtered_events(&self) -> Vec<&AgentEvent> {
        apply_filter(&self.events, &self.filter)
    }
}

impl Default for EventTimelineView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for EventTimelineView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let filtered = self.filtered_events();
        let count = filtered.len();

        // 최근 10개 이벤트 kind 문자열 (역순)
        let recent: Vec<String> = filtered
            .iter()
            .rev()
            .take(10)
            .map(|ev| match &ev.kind {
                moai_studio_agent::events::EventKind::StreamJson(s) => {
                    format!("stream:{}", s.type_)
                }
                moai_studio_agent::events::EventKind::Hook(h) => {
                    format!("hook:{}", h.event_name)
                }
                moai_studio_agent::events::EventKind::Unknown(_) => "unknown".to_string(),
            })
            .collect();

        // filter chip row (StreamJson / Hook / Unknown)
        let chip_stream = self.filter.allowed_kinds.contains(&EventKindDiscriminant::StreamJson);
        let chip_hook = self.filter.allowed_kinds.contains(&EventKindDiscriminant::Hook);
        let chip_unknown = self.filter.allowed_kinds.contains(&EventKindDiscriminant::Unknown);

        let chip_color = |active: bool| -> u32 {
            if active {
                tok::FG_PRIMARY
            } else {
                tok::FG_DISABLED
            }
        };

        let chip_row = div()
            .flex()
            .flex_row()
            .gap(px(8.))
            .py(px(4.))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(chip_color(chip_stream)))
                    .child("stream"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(chip_color(chip_hook)))
                    .child("hook"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(chip_color(chip_unknown)))
                    .child("unknown"),
            );

        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(tok::BG_SURFACE))
            .p_3()
            .gap_1()
            // filter chip row (MS-2)
            .child(chip_row)
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(tok::FG_MUTED))
                    .child(format!("Events: {}", count)),
            );

        for kind_str in recent {
            container = container.child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_SECONDARY))
                    .py(px(1.))
                    .child(kind_str),
            );
        }

        container
    }
}

// ================================================================
// 테스트 (RED-GREEN 사이클 — MS-2 filter)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::events::{EventKind, HookEvent, StreamJsonEvent};
    use serde_json::Value;

    fn make_stream(id: u64) -> AgentEvent {
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

    fn make_hook(id: u64) -> AgentEvent {
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

    /// Hook chip 토글 시 filtered_events 에서 hook 이벤트가 제외되어야 한다
    #[test]
    fn filtered_events_excludes_dimmed_kinds() {
        let mut view = EventTimelineView::new();
        view.events.push(make_stream(0));
        view.events.push(make_hook(1));
        view.events.push(make_stream(2));

        // Hook 제거
        view.filter.toggle_kind(EventKindDiscriminant::Hook);

        let filtered = view.filtered_events();
        assert_eq!(filtered.len(), 2, "hook 제거 후 stream 2개만 남아야 한다");
        for ev in &filtered {
            assert!(matches!(ev.kind, EventKind::StreamJson(_)));
        }
    }

    /// allow_all 상태에서는 모든 이벤트가 통과해야 한다
    #[test]
    fn all_events_pass_with_allow_all_filter() {
        let mut view = EventTimelineView::new();
        view.events.push(make_stream(0));
        view.events.push(make_hook(1));
        view.events.push(AgentEvent {
            id: 2,
            timestamp_ns: 0,
            kind: EventKind::Unknown(Value::Null),
            raw: String::new(),
        });

        let filtered = view.filtered_events();
        assert_eq!(filtered.len(), 3);
    }
}
