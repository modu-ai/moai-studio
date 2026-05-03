//! agent 모듈 — Agent Progress Dashboard UI 컴포넌트 (SPEC-V3-010)
//!
//! - `timeline_view`: EventTimelineView GPUI Render 골격 (MS-1, filter 추가 MS-2)
//! - `dashboard_view`: AgentDashboardView GPUI Render 골격
//! - `cost_panel_view`: CostPanelView GPUI Render — session/daily/weekly 비용 표시 (MS-2)
//! - `instructions_graph_view`: InstructionsGraphView — 6-layer instruction tree (MS-3, RG-AD-4)
//! - `control_bar`: AgentControlBar — pause/resume/kill + confirm modal (MS-3, RG-AD-5)
//! - `detail_view`: EventDetailView — JSON full payload + collapse + Copy (MS-3, RG-AD-6)
//! - `mission_control_view`: MissionControlView — read-only 4-cell parallel-agents grid
//!   (SPEC-V0-2-0-MISSION-CTRL-001 MS-2, audit Top 8 #2)

pub mod control_bar;
pub mod cost_panel_view;
pub mod dashboard_view;
pub mod detail_view;
pub mod instructions_graph_view;
pub mod mission_control_view;
pub mod timeline_view;

pub use control_bar::AgentControlBar;
pub use cost_panel_view::CostPanelView;
pub use detail_view::{CopyError, EventDetailView};
pub use instructions_graph_view::{
    InstructionRow, InstructionsGraphView, flatten_tree, open_in_editor,
};
pub use mission_control_view::{
    CellData, DEFAULT_MAX_CELLS, MissionControlView, format_cost, status_pill_color,
    status_pill_label,
};
