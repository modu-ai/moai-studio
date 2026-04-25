//! EventTimelineView GPUI Entity (RG-AD-2 minimal, REQ-AD-008)
//!
//! MS-1: event count + 최근 10 개 이벤트 type 을 div 텍스트로 렌더한다.
//! filter / 60fps throttle / detail panel 연동은 MS-2/3 에서 추가한다.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_agent::events::AgentEvent;

use crate::tokens;

// @MX:ANCHOR: [AUTO] timeline-view-entity-ui
// @MX:REASON: [AUTO] GPUI 렌더 진입점. fan_in >= 3:
//   AgentDashboardView, RootView(future), test code.
//   SPEC: SPEC-V3-010 REQ-AD-008

/// EventTimelineView GPUI Entity — ring buffer 이벤트 목록 렌더 (REQ-AD-008).
///
/// MS-1: event count + 최근 10개 이벤트 type 문자열 표시.
pub struct EventTimelineView {
    /// 이벤트 목록 (ring buffer 스냅샷).
    events: Vec<AgentEvent>,
}

impl EventTimelineView {
    /// 새 EventTimelineView 를 생성한다.
    pub fn new() -> Self {
        Self { events: Vec::new() }
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
}

impl Default for EventTimelineView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for EventTimelineView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.events.len();

        // 최근 10 개 이벤트 kind 문자열 (역순)
        let recent: Vec<String> = self
            .events
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

        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(tokens::BG_SURFACE))
            .p_3()
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(tokens::FG_MUTED))
                    .child(format!("Events: {}", count)),
            );

        for kind_str in recent {
            container = container.child(
                div()
                    .text_xs()
                    .text_color(rgb(tokens::FG_SECONDARY))
                    .py(px(1.))
                    .child(kind_str),
            );
        }

        container
    }
}
