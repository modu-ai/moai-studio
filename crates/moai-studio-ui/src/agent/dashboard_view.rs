//! AgentDashboardView GPUI Entity (MS-1 골격)
//!
//! MS-1: EventTimelineView 를 포함하는 컨테이너. split layout + cost / instructions / control 은
//! MS-2/3 에서 추가한다.

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, rgb,
};

use crate::design::tokens as tok;

use super::timeline_view::EventTimelineView;

// @MX:TODO(MS-2-cost-panel): CostPanelView Entity 추가 (MS-2 담당)
// @MX:TODO(MS-3-instructions-graph): InstructionsGraphView Entity 추가 (MS-3 담당)

/// AgentDashboardView — agent progress dashboard 최상위 컨테이너 (MS-1 골격).
///
/// timeline Entity 를 포함하며, MS-2/3 에서 cost panel / instructions graph / control bar 를 추가한다.
pub struct AgentDashboardView {
    /// EventTimelineView Entity
    pub timeline: Entity<EventTimelineView>,
}

impl AgentDashboardView {
    /// 새 AgentDashboardView 를 생성한다. timeline Entity 를 초기화한다.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let timeline = cx.new(|_cx| EventTimelineView::new());
        Self { timeline }
    }
}

impl Render for AgentDashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(tok::BG_APP))
            // timeline placeholder — MS-2/3 에서 split layout 으로 교체
            .child(self.timeline.clone())
    }
}
