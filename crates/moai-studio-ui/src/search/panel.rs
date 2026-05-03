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

    // ── Session management ──

    /// Cancel the active session if one exists (REQ-GS-022).
    pub fn cancel_active_session(&mut self) {
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }
    }

    // ── Batch flush ──

    /// Append a single hit to the pending buffer (REQ-GS-034).
    ///
    /// Call `should_flush` + `flush_pending` in the MS-3 poll loop.
    pub fn add_hit(&mut self, hit: SearchHit) {
        self.pending_buffer.push(hit);
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
