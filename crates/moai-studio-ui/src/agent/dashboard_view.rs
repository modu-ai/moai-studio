//! AgentDashboardView GPUI Entity (MS-1 골격, MS-2 cost panel 슬롯, MS-3 5-pane 통합)
//!
//! MS-3 레이아웃:
//! ```text
//! ┌─────────────────── AgentControlBar (toolbar) ───────────────────┐
//! ├─────────────────────────────┬──────────────────────────────────┤
//! │ EventTimelineView           │ CostPanelView                    │
//! │ (좌측 상단)                  │ (우측 상단)                       │
//! ├─────────────────────────────┼──────────────────────────────────┤
//! │ InstructionsGraphView       │ EventDetailView                  │
//! │ (좌측 하단, RG-AD-4)         │ (우측 하단, RG-AD-6)              │
//! └─────────────────────────────┴──────────────────────────────────┘
//! ```
//!
//! REQ-AD-012: timeline event 클릭 → EventDetailView 라우팅 진입점은
//! `route_event_to_detail` 헬퍼.

use std::path::PathBuf;

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};
use moai_studio_agent::events::AgentEvent;

use crate::design::tokens as tok;

use super::control_bar::AgentControlBar;
use super::cost_panel_view::CostPanelView;
use super::detail_view::EventDetailView;
use super::instructions_graph_view::InstructionsGraphView;
use super::timeline_view::EventTimelineView;

/// AgentDashboardView — agent progress dashboard 최상위 컨테이너.
///
/// MS-3: 5 pane (control bar 상단 + 4-quadrant) split layout.
pub struct AgentDashboardView {
    /// EventTimelineView Entity (좌측 상단)
    pub timeline: Entity<EventTimelineView>,
    /// CostPanelView Entity (우측 상단, AC-AD-5/6)
    pub cost_panel: Entity<CostPanelView>,
    /// InstructionsGraphView Entity (좌측 하단, AC-AD-7/8)
    pub instructions: Entity<InstructionsGraphView>,
    /// AgentControlBar Entity (상단 toolbar, AC-AD-9/10)
    pub control_bar: Entity<AgentControlBar>,
    /// EventDetailView Entity (우측 하단, AC-AD-11)
    pub detail_view: Entity<EventDetailView>,
}

impl AgentDashboardView {
    /// 새 AgentDashboardView 를 생성한다.
    /// `project_root` 는 InstructionsGraphView 의 6-layer 스캔 기준.
    pub fn new(project_root: PathBuf, cx: &mut Context<Self>) -> Self {
        let timeline = cx.new(|_cx| EventTimelineView::new());
        let cost_panel = cx.new(|_cx| CostPanelView::new());
        let instructions = cx.new(|_cx| InstructionsGraphView::new(project_root));
        let control_bar = cx.new(|_cx| AgentControlBar::new());
        let detail_view = cx.new(|_cx| EventDetailView::new());
        Self {
            timeline,
            cost_panel,
            instructions,
            control_bar,
            detail_view,
        }
    }

    /// REQ-AD-012: timeline 에서 선택된 event 를 EventDetailView 로 라우팅한다.
    pub fn route_event_to_detail(&self, event: AgentEvent, cx: &mut Context<Self>) {
        self.detail_view.update(cx, |view, cx| {
            view.select(event);
            cx.notify();
        });
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
            // 상단 toolbar — control bar (RG-AD-5)
            .child(self.control_bar.clone())
            // 본문 — 좌우 2컬럼 split
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_grow()
                    .gap(px(4.))
                    .child(
                        // 좌측 컬럼: timeline (상단) + instructions (하단)
                        div()
                            .flex()
                            .flex_col()
                            .flex_grow()
                            .gap(px(4.))
                            .child(self.timeline.clone())
                            .child(self.instructions.clone()),
                    )
                    .child(
                        // 우측 컬럼: cost panel (상단) + detail view (하단)
                        div()
                            .flex()
                            .flex_col()
                            .flex_grow()
                            .gap(px(4.))
                            .child(self.cost_panel.clone())
                            .child(self.detail_view.clone()),
                    ),
            )
    }
}
