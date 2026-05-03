//! `SearchPanel` GPUI Entity — global search sidebar section.
//!
//! SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2 (REQ-GS-030~035).
//!
//! # Design
//!
//! `SearchPanel` is a logic-bearing GPUI entity. The heavy search work lives
//! in the `moai-search` crate; this module owns the UI state machine:
//!
//! - visibility toggle (`is_visible` / `toggle`)
//! - input field state (`query`, `set_query`)
//! - `SearchStatus` enum + `status_text()` accessor
//! - batch flush buffer (`add_hit`, `should_flush`, `flush_pending`)
//! - workspace count for edge-case rendering (`set_workspace_count`)
//!
//! All logic-level methods have no GPUI Context parameter, making them
//! unit-testable without `TestAppContext` (Spike 2 pattern from SPEC-V3-005).

use moai_search::{CancelToken, SearchHit};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// SearchStatus
// ---------------------------------------------------------------------------

/// State machine for the status line shown below the result list.
///
/// @MX:NOTE: [AUTO] search-status-state-machine
/// Transitions driven by `SearchPanel::set_query`, `add_hit`, `flush_pending`,
/// and the worker lifecycle hooks wired in MS-3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchStatus {
    /// No query entered or results cleared (default state).
    Empty,
    /// Workers are running; results may be arriving.
    Searching,
    /// Workers finished with at least one result in `results`.
    HasResults,
    /// Workers finished with zero results.
    NoMatches,
    /// Total cap (1000 hits) was reached and workers were auto-cancelled.
    CapReached,
}

impl SearchStatus {
    /// Human-readable status string for the status line (REQ-GS-033).
    pub fn status_text(&self) -> &'static str {
        match self {
            SearchStatus::Empty => "",
            SearchStatus::Searching => "Searching...",
            SearchStatus::HasResults => "Results found",
            SearchStatus::NoMatches => "No matches",
            SearchStatus::CapReached => "Too many results — narrow your query",
        }
    }
}

// ---------------------------------------------------------------------------
// SearchPanel
// ---------------------------------------------------------------------------

/// Total cap on search hits across all workspaces (REQ-GS-024).
///
/// @MX:NOTE: [AUTO] total-cap-constant
/// When `pending_buffer.len() + results.len()` reaches this value in `add_hit`,
/// the active session is auto-cancelled and status is set to `CapReached`.
const TOTAL_HIT_CAP: usize = 1000;

/// GPUI Entity for the sidebar global-search section.
///
/// @MX:ANCHOR: [AUTO] search-panel-entity
/// @MX:REASON: [AUTO] Central fan_in target for MS-2~4:
///   - `RootView::new` (init / field mount)
///   - `handle_search_key_event` (toggle + focus_input)
///   - `dispatch_command("workspace.search")` (MS-3 palette integration)
///   - batch flush ticker (MS-3 worker wire)
pub struct SearchPanel {
    /// Whether the panel is currently visible in the sidebar.
    is_visible: bool,
    /// Trimmed query string (empty = no active search).
    query: String,
    /// Current status of the search session.
    pub status: SearchStatus,
    /// Accumulated search results (after flush from `pending_buffer`).
    pub results: Vec<SearchHit>,
    /// Cancellation token for the active search session.
    ///
    /// `None` when no session is running.
    ///
    /// @MX:WARN: [AUTO] cancel-token-ownership
    /// @MX:REASON: [AUTO] `cancel_token` must be cancelled whenever a new
    /// query is set OR the panel is closed. Failure to cancel leaks worker
    /// threads. See REQ-GS-022 for the full cancel contract.
    pub cancel_token: Option<CancelToken>,
    /// Buffered hits waiting for the next batch flush.
    pending_buffer: Vec<SearchHit>,
    /// Wall-clock timestamp of the last flush (used for 1000 ms threshold).
    last_flush_at: Option<Instant>,
    /// Number of active workspaces — drives the 0-workspace edge case.
    workspace_count: usize,
    /// Keyboard navigation: index of the currently selected result row.
    ///
    /// `None` when no row is selected (initial state or after results cleared).
    selected_index: Option<usize>,
    /// Per-workspace progress tracking: true when a worker is still running.
    workspace_progress: HashMap<String, bool>,
}

impl SearchPanel {
    /// Construct a new, hidden `SearchPanel` in the `Empty` state.
    pub fn new() -> Self {
        Self {
            is_visible: false,
            query: String::new(),
            status: SearchStatus::Empty,
            results: Vec::new(),
            cancel_token: None,
            pending_buffer: Vec::new(),
            last_flush_at: None,
            workspace_count: 0,
            selected_index: None,
            workspace_progress: HashMap::new(),
        }
    }

    // ── Visibility ──

    /// Return `true` if the panel is currently visible.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Toggle the panel between visible and hidden.
    ///
    /// When hiding, the active session is cancelled (REQ-GS-022).
    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
        if !self.is_visible {
            self.cancel_active_session();
        }
    }

    /// Bring the panel into view and prepare the input field for focus.
    ///
    /// The actual GPUI focus handle assignment is wired in MS-3.
    /// This method records intent so callers can observe the flag
    /// in logic-level tests (Spike 2 pattern).
    pub fn focus_input(&mut self) {
        self.is_visible = true;
        // GPUI focus dispatch is deferred to MS-3 navigation wire.
    }

    // ── Query ──

    /// Return the current (trimmed) query string.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Update the query string, cancelling any active session.
    ///
    /// Empty (or all-whitespace) queries clear results and set `Empty` status
    /// (REQ-GS-035). Non-empty queries update the stored query; worker spawn
    /// is handled by the navigation wire in MS-3.
    pub fn set_query(&mut self, raw: &str) {
        let trimmed = raw.trim().to_string();
        // Cancel any running session before changing query (REQ-GS-022).
        if self.cancel_token.is_some() {
            self.cancel_active_session();
        }
        if trimmed.is_empty() {
            self.clear_results();
        } else {
            self.query = trimmed;
            // Status stays at Searching when a worker is spawned (MS-3).
            // For now (MS-2 logic only) just mark as Searching intent.
        }
    }

    /// Clear results, pending buffer, and reset status to `Empty`.
    pub fn clear_results(&mut self) {
        self.query = String::new();
        self.results.clear();
        self.pending_buffer.clear();
        self.status = SearchStatus::Empty;
    }

    // ── Workspace count ──

    /// Set the number of active workspaces — used by the 0-workspace edge case.
    pub fn set_workspace_count(&mut self, count: usize) {
        self.workspace_count = count;
    }

    /// Return `true` when there are zero active workspaces (input disabled).
    pub fn input_disabled(&self) -> bool {
        self.workspace_count == 0
    }

    /// Placeholder text shown in the input field.
    pub fn input_placeholder(&self) -> &'static str {
        if self.workspace_count == 0 {
            "Open a workspace to search"
        } else {
            "Search across workspaces..."
        }
    }

    // ── Keyboard navigation (MS-4 T3) ──

    /// Return the currently selected result index, or `None` if nothing is selected.
    ///
    /// @MX:ANCHOR: [AUTO] search-panel-selected-index
    /// @MX:REASON: [AUTO] fan_in >= 3: move_selection_down, move_selection_up,
    ///   enter_selected — all read/write this field via accessor.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Move the keyboard selection down by one row (AC-GS-7).
    ///
    /// When no row is selected, selects the first row (index 0).
    /// When at the last row, stays at the last row (no wrap-around).
    pub fn move_selection_down(&mut self) {
        let len = self.results.len();
        if len == 0 {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            None => 0,
            Some(i) => (i + 1).min(len - 1),
        });
    }

    /// Move the keyboard selection up by one row (AC-GS-7).
    ///
    /// When at the first row (index 0), stays at index 0 (saturating).
    pub fn move_selection_up(&mut self) {
        if self.results.is_empty() {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            None | Some(0) => 0,
            Some(i) => i - 1,
        });
    }

    /// Return a clone of the currently selected `SearchHit`, or `None` when
    /// no row is selected or the index is out of bounds.
    ///
    /// The caller (`RootView::handle_search_open`) performs the actual
    /// workspace activation + tab open + line scroll (REQ-GS-040).
    pub fn enter_selected(&self) -> Option<SearchHit> {
        let idx = self.selected_index?;
        self.results.get(idx).cloned()
    }

    /// Hide the panel (equivalent to toggling off).
    ///
    /// Called when the user presses Escape while the panel is focused.
    pub fn escape_pressed(&mut self) {
        self.is_visible = false;
        self.cancel_active_session();
    }

    // ── Per-workspace progress (MS-4 T2) ──

    /// Record the in-progress state for a given workspace (AC-GS-9).
    ///
    /// `in_progress = true` means a worker is still running for that workspace.
    /// `in_progress = false` means the worker has finished (or was not started).
    pub fn set_workspace_progress(&mut self, workspace_id: &str, in_progress: bool) {
        self.workspace_progress
            .insert(workspace_id.to_string(), in_progress);
    }

    /// Return `true` when a worker for `workspace_id` is still running.
    pub fn is_workspace_in_progress(&self, workspace_id: &str) -> bool {
        self.workspace_progress
            .get(workspace_id)
            .copied()
            .unwrap_or(false)
    }

    // ── Session management ──

    /// Cancel the active session if one exists (REQ-GS-022).
    pub fn cancel_active_session(&mut self) {
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }
    }

    // ── Navigation wire — MS-3 (REQ-GS-040) ──

    /// Record that a result row was clicked, returning the `SearchHit` to
    /// navigate to. The actual navigation side-effects (workspace activate,
    /// new tab, scroll-to-line) are performed by `RootView::handle_search_open`.
    ///
    /// Returns `None` when `index` is out of bounds — no panic (REQ-GS-042).
    pub fn hit_for_row_click(&self, index: usize) -> Option<&SearchHit> {
        self.results.get(index)
    }

    // ── Batch flush ──

    /// User-facing message shown when the total hit cap is reached.
    pub fn cap_message() -> &'static str {
        "Too many results — narrow your query"
    }

    /// Append a single hit to the pending buffer (REQ-GS-034).
    ///
    /// When the combined count (`results.len() + pending_buffer.len()`) reaches
    /// `TOTAL_HIT_CAP` (1000), the active session is auto-cancelled and status
    /// is transitioned to `CapReached` (REQ-GS-024). Subsequent calls after cap
    /// is reached are silently dropped.
    ///
    /// @MX:NOTE: [AUTO] add-hit-cap-guard
    /// The cap guard fires on the hit that causes the total to reach 1000.
    /// The cancel_token.cancel() call stops the background workers. Any hits
    /// arriving after the cancel races through the channel are discarded here.
    pub fn add_hit(&mut self, hit: SearchHit) {
        // Drop hits once cap is reached.
        if self.status == SearchStatus::CapReached {
            return;
        }
        let total = self.results.len() + self.pending_buffer.len();
        if total >= TOTAL_HIT_CAP {
            self.status = SearchStatus::CapReached;
            self.cancel_active_session();
            return;
        }
        self.pending_buffer.push(hit);
        // Check again after push: if we just crossed the cap boundary, cancel.
        if self.results.len() + self.pending_buffer.len() >= TOTAL_HIT_CAP {
            self.status = SearchStatus::CapReached;
            self.cancel_active_session();
        }
    }

    /// Return the number of hits currently waiting in the pending buffer.
    ///
    /// Used by integration tests and the cap-check in `add_hit` to compute
    /// the total hit count without flushing.
    pub fn pending_buffer_len(&self) -> usize {
        self.pending_buffer.len()
    }

    /// Return `true` if the pending buffer should be flushed to `results`.
    ///
    /// Flush conditions (REQ-GS-034):
    /// - `pending_buffer.len() >= 100`, OR
    /// - 1000 ms have elapsed since `last_flush_at`.
    pub fn should_flush(&self, now: Instant) -> bool {
        if self.pending_buffer.len() >= 100 {
            return true;
        }
        if let Some(last) = self.last_flush_at
            && now.duration_since(last) >= Duration::from_millis(1000)
        {
            return true;
        }
        false
    }

    /// Move buffered hits into `results` and update the flush timestamp.
    pub fn flush_pending(&mut self, now: Instant) {
        self.results.append(&mut self.pending_buffer);
        self.last_flush_at = Some(now);
        if !self.results.is_empty() {
            self.status = SearchStatus::HasResults;
        }
    }
}

impl Default for SearchPanel {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hit(workspace_id: &str, preview: &str) -> SearchHit {
        SearchHit {
            workspace_id: workspace_id.to_string(),
            rel_path: std::path::PathBuf::from("src/main.rs"),
            line: 1,
            col: 0,
            preview: preview.to_string(),
            match_start: 0,
            match_end: preview.len() as u32,
        }
    }

    // ── T2: visibility + toggle + focus_input ──

    /// AC-GS-7 (logic): default state is invisible.
    #[test]
    fn test_search_panel_default_invisible() {
        let panel = SearchPanel::new();
        assert!(!panel.is_visible(), "panel must start hidden");
    }

    /// AC-GS-7 (logic): toggle makes panel visible, second toggle hides it.
    #[test]
    fn test_search_panel_toggle_visibility() {
        let mut panel = SearchPanel::new();
        panel.toggle();
        assert!(panel.is_visible(), "first toggle must show panel");
        panel.toggle();
        assert!(!panel.is_visible(), "second toggle must hide panel");
    }

    /// AC-GS-7 (logic): focus_input makes panel visible.
    #[test]
    fn test_search_panel_focus_input() {
        let mut panel = SearchPanel::new();
        assert!(!panel.is_visible());
        panel.focus_input();
        assert!(panel.is_visible(), "focus_input must reveal panel");
    }

    // ── T3: input field state ──

    /// AC-GS-9 (logic): non-empty query is stored.
    #[test]
    fn test_search_panel_set_query_non_empty() {
        let mut panel = SearchPanel::new();
        panel.set_query("TODO");
        assert_eq!(panel.query(), "TODO");
    }

    /// AC-GS-9 (empty query no-spawn): empty string clears results + Empty status.
    #[test]
    fn test_search_panel_empty_trim_clears_results() {
        let mut panel = SearchPanel::new();
        // Seed some state.
        panel.results.push(make_hit("ws-1", "some hit"));
        panel.status = SearchStatus::HasResults;
        // Set empty query.
        panel.set_query("   ");
        assert_eq!(
            panel.query(),
            "",
            "trimmed empty query must clear stored query"
        );
        assert!(panel.results.is_empty(), "results must be cleared");
        assert_eq!(panel.status, SearchStatus::Empty);
    }

    // ── T4: status state machine ──

    /// AC-GS-9 (status): Empty status text is empty string.
    #[test]
    fn test_search_panel_status_empty() {
        let panel = SearchPanel::new();
        assert_eq!(panel.status, SearchStatus::Empty);
        assert_eq!(panel.status.status_text(), "");
    }

    /// AC-GS-9 (status): Searching status text.
    #[test]
    fn test_search_panel_status_searching() {
        let mut panel = SearchPanel::new();
        panel.status = SearchStatus::Searching;
        assert_eq!(panel.status.status_text(), "Searching...");
    }

    /// AC-GS-9 (status): NoMatches status text.
    #[test]
    fn test_search_panel_status_no_matches() {
        let mut panel = SearchPanel::new();
        panel.status = SearchStatus::NoMatches;
        assert_eq!(panel.status.status_text(), "No matches");
    }

    /// AC-GS-9 (status): CapReached status text.
    #[test]
    fn test_search_panel_status_cap_reached() {
        let mut panel = SearchPanel::new();
        panel.status = SearchStatus::CapReached;
        assert_eq!(
            panel.status.status_text(),
            "Too many results — narrow your query"
        );
    }

    // ── MS-4 T3: keyboard navigation ──

    /// AC-GS-7: move_selection_down advances the selected index.
    #[test]
    fn test_search_panel_navigate_down() {
        let mut panel = SearchPanel::new();
        // Add 3 results to results vec directly (simulating flushed hits).
        panel.results.push(make_hit("ws-1", "hit 0"));
        panel.results.push(make_hit("ws-1", "hit 1"));
        panel.results.push(make_hit("ws-1", "hit 2"));

        assert_eq!(panel.selected_index(), None, "initial selection is None");
        panel.move_selection_down();
        assert_eq!(panel.selected_index(), Some(0), "first down → index 0");
        panel.move_selection_down();
        assert_eq!(panel.selected_index(), Some(1), "second down → index 1");
        panel.move_selection_down();
        assert_eq!(panel.selected_index(), Some(2), "third down → index 2");
        // At last item, further down is a no-op.
        panel.move_selection_down();
        assert_eq!(
            panel.selected_index(),
            Some(2),
            "down at last item stays at last"
        );
    }

    /// AC-GS-7: move_selection_up at index 0 is a no-op.
    #[test]
    fn test_search_panel_navigate_up_at_top_no_op() {
        let mut panel = SearchPanel::new();
        panel.results.push(make_hit("ws-1", "hit 0"));
        panel.move_selection_down(); // index = 0
        panel.move_selection_up(); // still 0 (saturating)
        assert_eq!(
            panel.selected_index(),
            Some(0),
            "up at index 0 must stay at 0"
        );
    }

    /// AC-GS-7: enter_selected returns the hit at the current selected index.
    #[test]
    fn test_search_panel_enter_opens_selected() {
        let mut panel = SearchPanel::new();
        panel.results.push(make_hit("ws-1", "hit 0"));
        panel.results.push(make_hit("ws-1", "hit 1"));
        panel.results.push(make_hit("ws-1", "hit 2"));
        panel.move_selection_down(); // 0
        panel.move_selection_down(); // 1
        panel.move_selection_down(); // 2
        let entered = panel.enter_selected();
        assert!(entered.is_some(), "enter must return a hit");
        assert_eq!(
            entered.unwrap().preview,
            "hit 2",
            "enter must return the selected hit"
        );
    }

    /// AC-GS-7: escape_pressed hides the panel.
    #[test]
    fn test_search_panel_escape_closes() {
        let mut panel = SearchPanel::new();
        panel.toggle(); // make visible
        assert!(panel.is_visible(), "panel must be visible before escape");
        panel.escape_pressed();
        assert!(!panel.is_visible(), "escape must hide the panel");
    }

    // ── MS-4 T2: per-workspace progress ──

    /// AC-GS-9: set_workspace_progress records the in-progress state for a workspace.
    #[test]
    fn test_search_panel_workspace_progress_set() {
        let mut panel = SearchPanel::new();
        panel.set_workspace_progress("ws-1", true);
        assert!(
            panel.is_workspace_in_progress("ws-1"),
            "ws-1 must be in progress after set(true)"
        );
        panel.set_workspace_progress("ws-1", false);
        assert!(
            !panel.is_workspace_in_progress("ws-1"),
            "ws-1 must not be in progress after set(false)"
        );
    }

    /// AC-GS-9: unknown workspace id returns false (not in progress).
    #[test]
    fn test_search_panel_workspace_progress_unknown_is_false() {
        let panel = SearchPanel::new();
        assert!(
            !panel.is_workspace_in_progress("ws-unknown"),
            "unknown workspace must return false"
        );
    }

    // ── MS-4 T1: total cap auto-cancel ──

    /// AC-GS-6 (UI): adding 1000 hits auto-cancels the session and sets CapReached.
    #[test]
    fn test_search_panel_total_cap_auto_cancels() {
        let mut panel = SearchPanel::new();
        // Status should transition to CapReached after 1000 hits.
        for i in 0..1000 {
            panel.add_hit(make_hit("ws-1", &format!("hit {i}")));
        }
        assert_eq!(
            panel.status,
            SearchStatus::CapReached,
            "1000 hits must set status to CapReached"
        );
    }

    /// AC-GS-6 (UI): cap_message returns the expected user-facing string.
    #[test]
    fn test_search_panel_cap_message_too_many_results() {
        assert_eq!(
            SearchPanel::cap_message(),
            "Too many results — narrow your query",
            "cap_message must match the CapReached status_text"
        );
    }

    /// AC-GS-6 (UI): hits added after cap is reached are dropped (ignored).
    #[test]
    fn test_search_panel_hits_after_cap_ignored() {
        let mut panel = SearchPanel::new();
        for i in 0..1000 {
            panel.add_hit(make_hit("ws-1", &format!("hit {i}")));
        }
        let count_at_cap = panel.pending_buffer.len() + panel.results.len();
        // Add one more hit after cap.
        panel.add_hit(make_hit("ws-1", "extra hit"));
        let count_after = panel.pending_buffer.len() + panel.results.len();
        assert_eq!(count_at_cap, count_after, "hits after cap must be dropped");
    }

    // ── T6: batch flush ──

    /// AC-GS-8 (batch): flush triggered at 100 hits threshold.
    #[test]
    fn test_search_panel_batch_flush_per_100_hits() {
        let mut panel = SearchPanel::new();
        // 99 hits — should NOT flush yet.
        for i in 0..99 {
            panel.add_hit(make_hit("ws-1", &format!("hit {i}")));
        }
        let t = Instant::now();
        assert!(!panel.should_flush(t), "99 hits must not trigger flush");
        // 100th hit — should flush.
        panel.add_hit(make_hit("ws-1", "hit 100"));
        assert!(panel.should_flush(t), "100 hits must trigger flush");
        panel.flush_pending(t);
        assert_eq!(
            panel.results.len(),
            100,
            "flush moves all 100 hits to results"
        );
        assert!(
            panel.pending_buffer.is_empty(),
            "pending buffer must be empty after flush"
        );
    }

    /// AC-GS-8 (batch): flush triggered after 1000 ms regardless of hit count.
    #[test]
    fn test_search_panel_batch_flush_per_1000ms() {
        let mut panel = SearchPanel::new();
        // Record an initial flush timestamp in the past (> 1s ago).
        let past = Instant::now() - Duration::from_millis(1100);
        panel.last_flush_at = Some(past);
        // Only 1 pending hit — below count threshold.
        panel.add_hit(make_hit("ws-1", "lone hit"));
        let now = Instant::now();
        assert!(
            panel.should_flush(now),
            "1000 ms elapsed must trigger flush regardless of hit count"
        );
        panel.flush_pending(now);
        assert_eq!(panel.results.len(), 1);
    }

    // ── T8: edge cases ──

    /// AC-GS-12 (UI): 0 workspace → input disabled + placeholder.
    #[test]
    fn test_search_panel_zero_workspace_disabled() {
        let mut panel = SearchPanel::new();
        panel.set_workspace_count(0);
        assert!(panel.input_disabled(), "0 workspace → input disabled");
        assert_eq!(panel.input_placeholder(), "Open a workspace to search");
    }

    /// AC-GS-12 (UI): 1 workspace → input enabled (single-group grouping is in result_view).
    #[test]
    fn test_search_panel_one_workspace_single_group() {
        let mut panel = SearchPanel::new();
        panel.set_workspace_count(1);
        assert!(!panel.input_disabled(), "1 workspace → input enabled");
    }
}
