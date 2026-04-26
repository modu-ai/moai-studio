//! AgentDashboardView GPUI Entity (MS-1 골격, MS-2 cost panel 슬롯 추가)
//!
//! MS-1: EventTimelineView 를 포함하는 컨테이너.
//! MS-2: CostPanelView 슬롯 추가. split layout + instructions / control 은 MS-3 에서 추가한다.

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, rgb,
};

use crate::design::tokens as tok;

use super::cost_panel_view::CostPanelView;
use super::timeline_view::EventTimelineView;

// @MX:TODO(MS-3-instructions-graph): InstructionsGraphView Entity 추가 (MS-3 담당)

/// AgentDashboardView — agent progress dashboard 최상위 컨테이너.
///
/// MS-2: CostPanelView Entity 를 슬롯으로 가진다.
/// MS-3 에서 instructions graph / control bar 를 추가한다.
pub struct AgentDashboardView {
    /// EventTimelineView Entity
    pub timeline: Entity<EventTimelineView>,
    /// CostPanelView Entity (MS-2, AC-AD-5/6)
    pub cost_panel: Entity<CostPanelView>,
}

impl AgentDashboardView {
    /// 새 AgentDashboardView 를 생성한다. timeline + cost panel Entity 를 초기화한다.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let timeline = cx.new(|_cx| EventTimelineView::new());
        let cost_panel = cx.new(|_cx| CostPanelView::new());
        Self {
            timeline,
            cost_panel,
        }
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
            // cost panel — 상단 고정 (MS-2)
            .child(self.cost_panel.clone())
            // timeline — cost panel 아래
            .child(self.timeline.clone())
    }
}
