//! MissionControlView — read-only 4-cell grid surface for parallel agent runs.
//!
//! SPEC-V0-2-0-MISSION-CTRL-001 MS-2 (audit Top 8 #2, v0.2.0 cycle Sprint 8).
//!
//! Renders a 2x2 grid (`max_cells = 4`) of `AgentCard` summaries cached in
//! `AgentRunRegistry` (MS-1). Empty slots show a "no active run" placeholder.
//! Each cell exposes:
//!   - status pill (color per `AgentRunStatus` — research §3 ADR-MC-4)
//!   - human label / run id
//!   - last event summary (1-line, 120-char truncated by MS-1 helper)
//!   - cost roll-up (`$0.0001` style; only when cost is Some)
//!
//! ADR-MC-1 deviation (MS-2 implementation choice): MissionControlView keeps a
//! `Vec<AgentCard>` snapshot rather than `Arc<RwLock<AgentRunRegistry>>`. Cleaner
//! ownership, no lock contention, simpler test injection. RootView is the
//! single owner of the registry and pushes snapshots into the view via
//! `set_snapshot`.
//!
//! Frozen zone (REQ-MC-040 ~ REQ-MC-042 + R3): existing agent module surfaces
//! and existing RootView fields are unchanged. Only one new RootView field
//! (`mission_control: Option<Entity<MissionControlView>>`) is added.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_agent::cost::CostSnapshot;
use moai_studio_agent::events::AgentRunStatus;
use moai_studio_agent::mission_control::AgentCard;

use crate::design::tokens as tok;
use crate::design::tokens::semantic;

/// Default number of rendered cells (2x2 grid).
pub const DEFAULT_MAX_CELLS: usize = 4;

/// Single grid-cell render data — `Some(card)` for active run, `None` for placeholder.
///
/// Pure data type — no GPUI types here so unit tests can verify the layout
/// decisions (status pill colour, cost formatting) without instantiating an
/// actual `Render` pass.
#[derive(Debug, Clone)]
pub struct CellData {
    /// The card backing this cell, or `None` when the slot is empty.
    pub card: Option<AgentCard>,
}

impl CellData {
    /// True when this cell has no agent card and should render the placeholder.
    pub fn is_empty(&self) -> bool {
        self.card.is_none()
    }
}

// ============================================================
// MissionControlView — RG-MC-3 (REQ-MC-020 ~ REQ-MC-024)
// ============================================================

/// Read-only 4-cell parallel-agents grid view.
///
/// @MX:NOTE: [AUTO] mission-control-view-snapshot
/// @MX:SPEC: SPEC-V0-2-0-MISSION-CTRL-001 REQ-MC-020
/// `MissionControlView` keeps an owned snapshot of the registry's top-N
/// active cards. RootView pushes updates via `set_snapshot`. The view itself
/// performs no mutation of the registry.
pub struct MissionControlView {
    /// Most recent snapshot of active cards (RootView calls `set_snapshot`).
    pub snapshot: Vec<AgentCard>,
    /// Maximum number of cells the view will render (default 4 = 2x2 grid).
    pub max_cells: usize,
}

impl MissionControlView {
    /// Construct an empty view with `max_cells = 4`.
    pub fn new() -> Self {
        Self {
            snapshot: Vec::new(),
            max_cells: DEFAULT_MAX_CELLS,
        }
    }

    /// Replace the cached snapshot with `cards` (typically the result of
    /// `AgentRunRegistry::top_n_active(self.max_cells)`).
    /// REQ-MC-020.
    pub fn set_snapshot(&mut self, cards: Vec<AgentCard>) {
        self.snapshot = cards;
    }

    /// Number of populated cells (clamped to `max_cells`).
    pub fn populated_cells(&self) -> usize {
        self.snapshot.len().min(self.max_cells)
    }

    /// Number of placeholder (empty) cells = `max_cells - populated`.
    pub fn placeholder_cells(&self) -> usize {
        self.max_cells.saturating_sub(self.populated_cells())
    }

    /// Compute the per-cell render data for the grid.
    ///
    /// Always returns exactly `max_cells` entries: the first `populated_cells()`
    /// hold real cards, the remainder are empty placeholders.
    /// REQ-MC-021.
    pub fn cell_data(&self) -> Vec<CellData> {
        let populated = self.populated_cells();
        let placeholders = self.placeholder_cells();
        let mut out: Vec<CellData> = self
            .snapshot
            .iter()
            .take(populated)
            .cloned()
            .map(|card| CellData { card: Some(card) })
            .collect();
        for _ in 0..placeholders {
            out.push(CellData { card: None });
        }
        out
    }
}

impl Default for MissionControlView {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Pure helpers — status pill colour + cost formatting
// ============================================================

/// Map an `AgentRunStatus` to the design-token colour used for the status pill.
///
/// research §3 ADR-MC-4 mapping:
/// - Running → `tok::ACCENT` (brand teal)
/// - Paused → `tok::FG_MUTED` (dim grey)
/// - Completed → `semantic::SUCCESS` (mint green)
/// - Failed / Killed → `semantic::DANGER` (crimson)
pub fn status_pill_color(status: AgentRunStatus) -> u32 {
    match status {
        AgentRunStatus::Running => tok::ACCENT,
        AgentRunStatus::Paused => tok::FG_MUTED,
        AgentRunStatus::Completed => semantic::SUCCESS,
        AgentRunStatus::Failed | AgentRunStatus::Killed => semantic::DANGER,
    }
}

/// Human label for the status pill text.
pub fn status_pill_label(status: AgentRunStatus) -> &'static str {
    match status {
        AgentRunStatus::Running => "Running",
        AgentRunStatus::Paused => "Paused",
        AgentRunStatus::Completed => "Completed",
        AgentRunStatus::Failed => "Failed",
        AgentRunStatus::Killed => "Killed",
    }
}

/// Format a cost roll-up for the cell footer.
///
/// Returns an empty string when there is no cost snapshot, otherwise
/// `"$0.0001"` style with 4 decimals.
pub fn format_cost(cost: &Option<CostSnapshot>) -> String {
    match cost {
        Some(c) => format!("${:.4}", c.usd),
        None => String::new(),
    }
}

// ============================================================
// GPUI Render impl — REQ-MC-021/022
// ============================================================

impl Render for MissionControlView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let cells = self.cell_data();
        // 2x2 grid: split into two rows of two cells each.
        let row_count = self.max_cells.div_ceil(2);
        let mut rows: Vec<gpui::Div> = Vec::with_capacity(row_count);
        for chunk in cells.chunks(2) {
            let mut row = div().flex().flex_row().flex_grow().gap(px(8.));
            for cell in chunk {
                row = row.child(render_cell(cell));
            }
            rows.push(row);
        }

        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p(px(12.))
            .gap(px(8.))
            .bg(rgb(tok::BG_APP));
        for row in rows {
            container = container.child(row);
        }
        container
    }
}

/// Render a single grid cell (populated card or empty placeholder).
fn render_cell(cell: &CellData) -> gpui::Div {
    let base = div()
        .flex()
        .flex_col()
        .flex_grow()
        .gap(px(6.))
        .p(px(10.))
        .rounded_md()
        .border_1()
        .border_color(rgb(tok::BORDER_SUBTLE))
        .bg(rgb(tok::BG_ELEVATED));

    match &cell.card {
        None => base.child(
            div()
                .flex()
                .flex_grow()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(tok::FG_MUTED))
                .child("no active run"),
        ),
        Some(card) => {
            let pill_bg = status_pill_color(card.status);
            let pill_label = status_pill_label(card.status);
            let cost_label = format_cost(&card.cost);
            let mut footer = div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child(format!("events: {}", card.event_count));
            if !cost_label.is_empty() {
                footer = footer.child(cost_label);
            }
            base.child(
                // Header: status pill + label
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .rounded_md()
                            .bg(rgb(pill_bg))
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(pill_label),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(card.label.clone()),
                    ),
            )
            .child(
                // Body: last event summary (1-line truncated upstream by MS-1)
                div().text_xs().text_color(rgb(tok::FG_SECONDARY)).child(
                    if card.last_event_summary.is_empty() {
                        "(awaiting events)".to_string()
                    } else {
                        card.last_event_summary.clone()
                    },
                ),
            )
            .child(footer)
        }
    }
}

// ============================================================
// Unit tests — SPEC-V0-2-0-MISSION-CTRL-001 MS-2 (AC-MC-11/12)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::events::AgentRunId;
    use std::time::SystemTime;

    fn mk_card(id: &str, label: &str, status: AgentRunStatus) -> AgentCard {
        let mut c = AgentCard::new(AgentRunId(id.to_string()), label);
        c.status = status;
        c.last_event_summary = format!("hook:{} payload", id);
        c.last_event_at = Some(1_000_000_000 + id.len() as u128);
        c.event_count = 5;
        c
    }

    /// AC-MC-11 (REQ-MC-020/021): empty view → 4 placeholder cells.
    #[test]
    fn empty_view_yields_four_placeholder_cells() {
        let view = MissionControlView::new();
        assert_eq!(view.populated_cells(), 0);
        assert_eq!(view.placeholder_cells(), DEFAULT_MAX_CELLS);
        let cells = view.cell_data();
        assert_eq!(cells.len(), DEFAULT_MAX_CELLS);
        for cell in &cells {
            assert!(cell.is_empty(), "all cells must be empty");
        }
    }

    /// AC-MC-12 (REQ-MC-021/022): 2 active cards → 2 filled + 2 placeholder.
    #[test]
    fn two_active_cards_yield_two_filled_and_two_placeholder() {
        let mut view = MissionControlView::new();
        view.set_snapshot(vec![
            mk_card("r1", "Run1", AgentRunStatus::Running),
            mk_card("r2", "Run2", AgentRunStatus::Paused),
        ]);
        assert_eq!(view.populated_cells(), 2);
        assert_eq!(view.placeholder_cells(), 2);

        let cells = view.cell_data();
        assert_eq!(cells.len(), 4);
        assert!(!cells[0].is_empty());
        assert_eq!(cells[0].card.as_ref().unwrap().label, "Run1");
        assert!(!cells[1].is_empty());
        assert_eq!(cells[1].card.as_ref().unwrap().label, "Run2");
        assert!(cells[2].is_empty());
        assert!(cells[3].is_empty());
    }

    /// REQ-MC-021: snapshot longer than max_cells is truncated to max_cells.
    #[test]
    fn snapshot_longer_than_max_is_truncated() {
        let mut view = MissionControlView::new();
        view.set_snapshot(vec![
            mk_card("r1", "Run1", AgentRunStatus::Running),
            mk_card("r2", "Run2", AgentRunStatus::Running),
            mk_card("r3", "Run3", AgentRunStatus::Running),
            mk_card("r4", "Run4", AgentRunStatus::Running),
            mk_card("r5", "Run5", AgentRunStatus::Running),
            mk_card("r6", "Run6", AgentRunStatus::Running),
        ]);
        assert_eq!(view.populated_cells(), DEFAULT_MAX_CELLS);
        assert_eq!(view.placeholder_cells(), 0);
        let cells = view.cell_data();
        assert_eq!(cells.len(), DEFAULT_MAX_CELLS);
        // 5th and 6th cards must NOT appear.
        let labels: Vec<String> = cells
            .iter()
            .filter_map(|c| c.card.as_ref().map(|x| x.label.clone()))
            .collect();
        assert_eq!(labels, vec!["Run1", "Run2", "Run3", "Run4"]);
    }

    /// REQ-MC-022 helper: status_pill_color returns the design-token mapping.
    #[test]
    fn status_pill_color_matches_adr_mc_4() {
        assert_eq!(status_pill_color(AgentRunStatus::Running), tok::ACCENT);
        assert_eq!(status_pill_color(AgentRunStatus::Paused), tok::FG_MUTED);
        assert_eq!(
            status_pill_color(AgentRunStatus::Completed),
            semantic::SUCCESS
        );
        assert_eq!(status_pill_color(AgentRunStatus::Failed), semantic::DANGER);
        assert_eq!(status_pill_color(AgentRunStatus::Killed), semantic::DANGER);
    }

    /// REQ-MC-022 helper: status_pill_label returns the human label.
    #[test]
    fn status_pill_label_returns_human_label() {
        assert_eq!(status_pill_label(AgentRunStatus::Running), "Running");
        assert_eq!(status_pill_label(AgentRunStatus::Paused), "Paused");
        assert_eq!(status_pill_label(AgentRunStatus::Completed), "Completed");
        assert_eq!(status_pill_label(AgentRunStatus::Failed), "Failed");
        assert_eq!(status_pill_label(AgentRunStatus::Killed), "Killed");
    }

    /// REQ-MC-022 helper: format_cost returns empty string for None.
    #[test]
    fn format_cost_none_returns_empty() {
        assert_eq!(format_cost(&None), "");
    }

    /// REQ-MC-022 helper: format_cost returns dollar-prefixed 4-decimal string.
    #[test]
    fn format_cost_some_returns_dollar_4_decimal() {
        let snap = CostSnapshot {
            timestamp: SystemTime::UNIX_EPOCH,
            usd: 0.1234567,
            run_id: AgentRunId("r1".to_string()),
        };
        assert_eq!(format_cost(&Some(snap)), "$0.1235");
    }

    /// MissionControlView::Default is a fresh empty view.
    #[test]
    fn default_yields_empty_view_with_default_max_cells() {
        let view = MissionControlView::default();
        assert_eq!(view.snapshot.len(), 0);
        assert_eq!(view.max_cells, DEFAULT_MAX_CELLS);
    }

    /// set_snapshot replaces (does not append) the cached snapshot.
    #[test]
    fn set_snapshot_replaces_existing() {
        let mut view = MissionControlView::new();
        view.set_snapshot(vec![mk_card("a", "A", AgentRunStatus::Running)]);
        assert_eq!(view.snapshot.len(), 1);
        view.set_snapshot(vec![
            mk_card("b", "B", AgentRunStatus::Running),
            mk_card("c", "C", AgentRunStatus::Running),
        ]);
        assert_eq!(view.snapshot.len(), 2);
        assert_eq!(view.snapshot[0].label, "B");
    }

    /// CellData::is_empty mirrors the card-presence boolean.
    #[test]
    fn cell_data_is_empty_mirrors_card_presence() {
        let empty = CellData { card: None };
        assert!(empty.is_empty());
        let filled = CellData {
            card: Some(mk_card("x", "X", AgentRunStatus::Running)),
        };
        assert!(!filled.is_empty());
    }
}
