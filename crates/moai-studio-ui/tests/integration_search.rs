//! Integration tests for SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-4.
//!
//! Final regression sweep of all 12 ACs via logic-level end-to-end scenarios.
//!
//! # Test strategy (HARD rule §8 GPUI fallback)
//!
//! All tests are logic-level (no GPUI context required) because `RootView`
//! exposes context-free APIs (`handle_toggle_search_panel` is GPUI-bound, but
//! `dispatch_command_workspace_search`, `handle_search_open`, and the
//! `SearchPanel` logic methods are context-free — Spike 2 pattern).
//!
//! Where full GPUI context is required (e.g. toggle via `handle_toggle_search_panel`),
//! the test drives `dispatch_command_workspace_search` or manipulates
//! `search_panel` directly as the logic-level equivalent.
//!
//! # AC coverage
//!
//! AC-GS-1  — SearchHit types: verified via `moai_search` crate (MS-1 unit tests).
//! AC-GS-2  — Walk results: verified via `moai_search` walker tests (MS-1).
//! AC-GS-3  — Gitignore: verified via `moai_search` walker tests (MS-1).
//! AC-GS-4  — Binary skip: verified via `moai_search` walker tests (MS-1).
//! AC-GS-5  — Cancel mid-walk: verified via `moai_search` walker tests (MS-1).
//! AC-GS-6  — Cap / auto-cancel: T5d integration test + panel unit tests (MS-4 T1).
//! AC-GS-7  — Keyboard navigation: T5c + panel unit tests (MS-4 T3).
//! AC-GS-8  — Highlight + batch: result_view unit tests (MS-4 T4) + panel tests.
//! AC-GS-9  — Searching status / spinner: T5b + panel unit tests (MS-4 T2).
//! AC-GS-10 — Navigation wire: navigation unit tests (MS-3).
//! AC-GS-11 — Palette entry: T5e + dispatch_command unit tests (MS-3).
//! AC-GS-12 — 0-workspace edge case: T5f + panel unit tests.

use moai_search::SearchHit;
use moai_studio_ui::RootView;
use moai_studio_ui::search::navigation::hit_to_open_code_viewer;
use moai_studio_ui::search::{SearchPanel, SearchStatus};
use moai_studio_ui::search::{extract_preview_segments, format_row_label};
use moai_studio_workspace::Workspace;
use std::path::PathBuf;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn make_hit(workspace_id: &str, rel_path: &str, line: u32, preview: &str) -> SearchHit {
    SearchHit {
        workspace_id: workspace_id.to_string(),
        rel_path: PathBuf::from(rel_path),
        line,
        col: 0,
        preview: preview.to_string(),
        match_start: 0,
        match_end: preview.len().min(4) as u32,
    }
}

fn make_workspace(id: &str, name: &str, path: &std::path::Path) -> Workspace {
    Workspace {
        id: id.to_string(),
        name: name.to_string(),
        project_path: path.to_path_buf(),
        moai_config: PathBuf::from(".moai"),
        color: 0xff6a3d,
        last_active: 0,
    }
}

fn make_root_view(workspaces: Vec<Workspace>, tmp: &TempDir) -> RootView {
    let store_path = tmp.path().join("workspaces.json");
    RootView::new(workspaces, store_path)
}

// ---------------------------------------------------------------------------
// T5a: ⌘⇧F → SearchPanel visible (AC-GS-7 logic-level equivalent)
// ---------------------------------------------------------------------------

/// AC-GS-7: dispatch_command_workspace_search makes the panel visible.
///
/// This is the logic-level equivalent of ⌘⇧F key dispatch. The actual
/// key binding routes through `handle_toggle_search_panel` (GPUI context
/// required), but `dispatch_command_workspace_search` is context-free and
/// exercises the same SearchPanel lazy-init path.
#[test]
fn integration_test_search_panel_becomes_visible_on_toggle() {
    let tmp = tempfile::tempdir().unwrap();
    let ws = make_workspace("ws-1", "project-alpha", tmp.path());
    let mut root = make_root_view(vec![ws], &tmp);

    // Initially no panel.
    assert!(
        root.search_panel.is_none(),
        "search_panel must be None before first toggle"
    );

    // First dispatch — lazy init → visible.
    root.dispatch_command_workspace_search();
    assert!(
        root.search_panel.is_some(),
        "search_panel must be Some after first dispatch"
    );
    assert!(
        root.search_panel.as_ref().unwrap().is_visible(),
        "panel must be visible after first dispatch"
    );

    // Second dispatch — toggle off.
    root.dispatch_command_workspace_search();
    assert!(
        !root.search_panel.as_ref().unwrap().is_visible(),
        "panel must be hidden after second dispatch (toggle off)"
    );
}

// ---------------------------------------------------------------------------
// T5b: set_query → results stream → batch flush (AC-GS-8/9)
// ---------------------------------------------------------------------------

/// AC-GS-9: set_query transitions status; AC-GS-8: add_hit + flush fills results.
#[test]
fn integration_test_query_results_flow() {
    let mut panel = SearchPanel::new();

    // Initially Empty.
    assert_eq!(panel.status, SearchStatus::Empty);

    // Set a query.
    panel.set_query("use");
    assert_eq!(panel.query(), "use");

    // Simulate worker returning hits and flushing at 100-hit threshold.
    for i in 0..100 {
        panel.add_hit(make_hit("ws-1", "src/main.rs", i as u32, "use std::fs;"));
    }
    // 100 hits should trigger should_flush.
    assert!(
        panel.should_flush(std::time::Instant::now()),
        "100 hits must trigger flush"
    );
    panel.flush_pending(std::time::Instant::now());
    assert_eq!(panel.results.len(), 100, "100 results after flush");
    assert_eq!(
        panel.status,
        SearchStatus::HasResults,
        "status must be HasResults after flush"
    );
}

// ---------------------------------------------------------------------------
// T5c: keyboard navigation → enter → navigation (AC-GS-7)
// ---------------------------------------------------------------------------

/// AC-GS-7: ↓×3 then Enter returns the 3rd hit; Escape hides panel.
#[test]
fn integration_test_keyboard_navigation() {
    let mut panel = SearchPanel::new();
    panel.toggle(); // make visible
    // Populate results directly.
    panel.results.push(make_hit("ws-1", "a.rs", 1, "line a"));
    panel.results.push(make_hit("ws-1", "b.rs", 2, "line b"));
    panel.results.push(make_hit("ws-1", "c.rs", 3, "line c"));
    panel.results.push(make_hit("ws-1", "d.rs", 4, "line d"));

    assert_eq!(panel.selected_index(), None, "no selection initially");

    panel.move_selection_down(); // 0
    panel.move_selection_down(); // 1
    panel.move_selection_down(); // 2
    panel.move_selection_down(); // 3
    assert_eq!(panel.selected_index(), Some(3), "4th row selected");

    let hit = panel.enter_selected();
    assert!(hit.is_some(), "enter_selected must return a hit");
    assert_eq!(
        hit.unwrap().preview,
        "line d",
        "enter_selected must return the 4th hit"
    );

    // Escape hides panel.
    panel.escape_pressed();
    assert!(!panel.is_visible(), "escape must close the panel");
}

// ---------------------------------------------------------------------------
// T5d: total cap → auto-cancel + message (AC-GS-6)
// ---------------------------------------------------------------------------

/// AC-GS-6: 1001 hits triggers auto-cancel and CapReached status.
#[test]
fn integration_test_total_cap_auto_cancel() {
    let mut panel = SearchPanel::new();

    for i in 0..1001 {
        panel.add_hit(make_hit("ws-1", "src/main.rs", i as u32, "some line"));
    }

    assert_eq!(
        panel.status,
        SearchStatus::CapReached,
        "status must be CapReached after 1001 hits"
    );
    // Cap message matches CapReached status_text.
    assert_eq!(
        SearchPanel::cap_message(),
        SearchStatus::CapReached.status_text(),
        "cap_message must match CapReached status_text"
    );
    // Total result count must not exceed the cap.
    let total = panel.results.len() + panel.pending_buffer_len();
    assert!(
        total <= 1000,
        "total hits must not exceed cap of 1000, got {total}"
    );
}

// ---------------------------------------------------------------------------
// T5e: Command Palette `workspace.search` → SearchPanel toggle (AC-GS-11)
// ---------------------------------------------------------------------------

/// AC-GS-11: dispatch_command("workspace.search") activates SearchPanel.
#[test]
fn integration_test_palette_workspace_search() {
    let tmp = tempfile::tempdir().unwrap();
    let mut root = make_root_view(vec![], &tmp);

    let handled = root.dispatch_command("workspace.search");
    assert!(
        handled,
        "dispatch_command('workspace.search') must return true"
    );
    assert!(
        root.search_panel.is_some(),
        "search_panel must exist after palette command"
    );
    assert!(
        root.search_panel.as_ref().unwrap().is_visible(),
        "panel must be visible after palette command"
    );
}

// ---------------------------------------------------------------------------
// T5f: 0-workspace edge case (AC-GS-12)
// ---------------------------------------------------------------------------

/// AC-GS-12: 0 workspace → input disabled + placeholder message.
#[test]
fn integration_test_zero_workspace_input_disabled() {
    let mut panel = SearchPanel::new();
    panel.set_workspace_count(0);

    assert!(panel.input_disabled(), "0 workspace → input disabled");
    assert_eq!(
        panel.input_placeholder(),
        "Open a workspace to search",
        "placeholder must hint to open a workspace"
    );
}

// ---------------------------------------------------------------------------
// T5g: handle_search_open → OpenCodeViewer resolved (AC-GS-10)
// ---------------------------------------------------------------------------

/// AC-GS-10: handle_search_open resolves workspace and records OpenCodeViewer.
#[test]
fn integration_test_search_open_resolves_ocv() {
    let tmp = tempfile::tempdir().unwrap();
    let ws = make_workspace("ws-1", "project-alpha", tmp.path());
    let mut root = make_root_view(vec![ws], &tmp);

    let hit = make_hit("ws-1", "src/lib.rs", 42, "pub fn main");
    let success = root.handle_search_open(&hit);

    assert!(
        success,
        "handle_search_open must return true for known workspace"
    );
    let ocv = root.last_open_code_viewer.as_ref();
    assert!(ocv.is_some(), "last_open_code_viewer must be set");
    let ocv = ocv.unwrap();
    assert_eq!(ocv.line, Some(42), "OCV line must be 42");
    assert!(
        ocv.path.ends_with("src/lib.rs"),
        "OCV path must end with rel_path"
    );
}

/// AC-GS-10 stability: unknown workspace returns false, no panic.
#[test]
fn integration_test_search_open_unknown_workspace_no_panic() {
    let tmp = tempfile::tempdir().unwrap();
    let mut root = make_root_view(vec![], &tmp);

    let hit = make_hit("ws-unknown", "src/lib.rs", 1, "line");
    let result = root.handle_search_open(&hit);
    assert!(!result, "unknown workspace must return false");
}

// ---------------------------------------------------------------------------
// T5h: per-workspace progress spinner (AC-GS-9)
// ---------------------------------------------------------------------------

/// AC-GS-9: set_workspace_progress + is_workspace_in_progress round-trip.
#[test]
fn integration_test_workspace_progress_spinner() {
    let mut panel = SearchPanel::new();

    // No workspace known yet.
    assert!(!panel.is_workspace_in_progress("ws-1"), "must start false");

    // Worker starts.
    panel.set_workspace_progress("ws-1", true);
    assert!(
        panel.is_workspace_in_progress("ws-1"),
        "must be true while worker is running"
    );

    // Worker finishes.
    panel.set_workspace_progress("ws-1", false);
    assert!(
        !panel.is_workspace_in_progress("ws-1"),
        "must be false after worker finishes"
    );
}

// ---------------------------------------------------------------------------
// T5i: match highlight segments (AC-GS-8)
// ---------------------------------------------------------------------------

/// AC-GS-8: extract_preview_segments produces correct three-segment split.
#[test]
fn integration_test_match_highlight_segments() {
    let preview = "pub fn main() { }";
    // Match "main" at bytes 7..11.
    let (pre, matched, post) = extract_preview_segments(preview, 7, 11);

    assert_eq!(pre, "pub fn ", "pre-match segment");
    assert_eq!(matched, "main", "match segment");
    assert_eq!(post, "() { }", "post-match segment");
}

// ---------------------------------------------------------------------------
// T5j: format_row_label (AC-GS-8 row layout)
// ---------------------------------------------------------------------------

/// AC-GS-8: format_row_label produces `<workspace> / <path>:<line>` format.
#[test]
fn integration_test_row_label_format() {
    let hit = SearchHit {
        workspace_id: "ws-alpha".to_string(),
        rel_path: PathBuf::from("crates/moai-search/src/lib.rs"),
        line: 99,
        col: 0,
        preview: "pub fn search".to_string(),
        match_start: 0,
        match_end: 3,
    };
    let label = format_row_label("alpha-project", &hit);
    assert_eq!(
        label, "alpha-project / crates/moai-search/src/lib.rs:99",
        "label must follow workspace / path:line format"
    );
}

// ---------------------------------------------------------------------------
// T5k: navigation adapter — hit_to_open_code_viewer (AC-GS-10)
// ---------------------------------------------------------------------------

/// AC-GS-10: hit_to_open_code_viewer resolves absolute path from workspace root.
#[test]
fn integration_test_hit_to_ocv_path_resolution() {
    let tmp = tempfile::tempdir().unwrap();
    let ws = make_workspace("ws-x", "project-x", tmp.path());
    let hit = make_hit("ws-x", "src/engine.rs", 5, "let x = 1;");

    let ocv = hit_to_open_code_viewer(&hit, &[ws]);
    assert!(ocv.is_some(), "must resolve for known workspace");
    let ocv = ocv.unwrap();
    assert!(
        ocv.path.starts_with(tmp.path()),
        "path must start with workspace root"
    );
    assert!(ocv.path.ends_with("src/engine.rs"), "path tail must match");
    assert_eq!(ocv.line, Some(5), "line must be 5");
}
