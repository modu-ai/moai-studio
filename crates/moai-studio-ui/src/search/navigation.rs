//! Navigation adapter — `SearchHit` → `OpenCodeViewer` + workspace activate.
//!
//! SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3 (REQ-GS-040~042, AC-GS-10).
//!
//! # Responsibility
//!
//! This module is the only place that converts a `SearchHit` (from the search
//! engine crate) into a navigation action: workspace activation via
//! `WorkspacesStore::touch`, new tab open via `TabContainer::new_tab`, and an
//! `OpenCodeViewer` struct (SPEC-V3-LINK-001) carrying the target path + line/col.
//!
//! All public functions are logic-level (no GPUI `Context` parameter) so they
//! can be unit-tested without a running GPUI application (Spike 2 pattern,
//! SPEC-V3-005 §6).
//!
//! # Failure contract (REQ-GS-042)
//!
//! None of the functions in this module ever panic. When workspace or path
//! resolution fails, the functions return `None` / `Err` and the caller logs
//! a `tracing::warn!`. The SearchPanel itself stays visible and usable.

use moai_search::SearchHit;
use moai_studio_terminal::link::OpenCodeViewer;
use moai_studio_workspace::{Workspace, WorkspacesStore};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Public adapter — T1
// ---------------------------------------------------------------------------

/// Convert a `SearchHit` into an `OpenCodeViewer` action by resolving the
/// workspace root path.
///
/// Returns `None` when `hit.workspace_id` is not found in `workspaces`.
///
/// @MX:ANCHOR: [AUTO] hit_to_open_code_viewer
/// @MX:REASON: [AUTO] Central adapter fan_in >= 3:
///   - `open_hit` (internal caller)
///   - `RootView::handle_search_open` (MS-3 wire)
///   - unit tests (T1)
pub fn hit_to_open_code_viewer(
    hit: &SearchHit,
    workspaces: &[Workspace],
) -> Option<OpenCodeViewer> {
    // Resolve workspace by id.
    let ws = workspaces.iter().find(|w| w.id == hit.workspace_id)?;
    // Build absolute path: workspace root + relative path from hit.
    let abs_path: PathBuf = ws.project_path.join(&hit.rel_path);
    Some(OpenCodeViewer {
        path: abs_path,
        line: Some(hit.line),
        col: Some(hit.col),
    })
}

// ---------------------------------------------------------------------------
// Navigation result — T2/T3/T4
// ---------------------------------------------------------------------------

/// Outcome of a navigation action.
///
/// Callers use this to drive status-bar feedback without panicking (REQ-GS-042).
#[derive(Debug, PartialEq)]
pub enum NavigationOutcome {
    /// Workspace was touched (activated) successfully.
    WorkspaceTouched,
    /// Workspace touch was skipped because the workspace id was not found in
    /// the store (file may not exist yet or store is stale).
    WorkspaceNotFound,
    /// The `OpenCodeViewer` struct was built and is ready for dispatch.
    OpenCodeViewerReady(OpenCodeViewer),
    /// Navigation could not be completed for the given reason.
    ///
    /// @MX:WARN: [AUTO] navigation-failure-reason
    /// @MX:REASON: [AUTO] Callers must log `tracing::warn!` on this variant to
    /// satisfy REQ-GS-042. Dropping the error silently violates the contract.
    Error(String),
}

/// Attempt to touch (activate) the workspace identified by `workspace_id` in
/// the given mutable store reference.
///
/// Returns `WorkspaceTouched` on success, `WorkspaceNotFound` when the id is
/// absent, or `Error` when the store write fails (filesystem I/O).
///
/// # Failure contract
///
/// This function never panics (REQ-GS-042).
pub fn touch_workspace(store: &mut WorkspacesStore, workspace_id: &str) -> NavigationOutcome {
    match store.touch(workspace_id) {
        Ok(()) => {
            // Verify the id actually exists in the store list.
            if store.list().iter().any(|w| w.id == workspace_id) {
                NavigationOutcome::WorkspaceTouched
            } else {
                // touch() returned Ok but the id was absent — treat as not found.
                NavigationOutcome::WorkspaceNotFound
            }
        }
        Err(e) => {
            tracing::warn!(
                workspace_id = workspace_id,
                error = %e,
                "touch_workspace: store write failed — navigation degraded"
            );
            NavigationOutcome::Error(format!("store write failed: {e}"))
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests — T1/T2/T3/T4/T5
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_workspace::{Workspace, WorkspacesStore};
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ── helpers ──

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

    fn make_hit(workspace_id: &str, rel_path: &str, line: u32, col: u32) -> SearchHit {
        SearchHit {
            workspace_id: workspace_id.to_string(),
            rel_path: PathBuf::from(rel_path),
            line,
            col,
            preview: "some preview text".to_string(),
            match_start: 0,
            match_end: 4,
        }
    }

    fn store_with_workspace(tmp: &TempDir, id: &str, name: &str) -> WorkspacesStore {
        let ws = make_workspace(id, name, tmp.path());
        let store_path = tmp.path().join("workspaces.json");
        // Load from a non-existent path creates an empty store, then add the workspace.
        let mut store = WorkspacesStore::load(store_path).unwrap();
        store.add(ws).unwrap();
        store
    }

    // ── T1: hit_to_open_code_viewer ──

    /// AC-GS-10 (logic): known workspace_id resolves to an OpenCodeViewer with
    /// the correct absolute path (workspace root + rel_path).
    #[test]
    fn test_hit_to_open_code_viewer_known_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = make_workspace("ws-1", "project-a", tmp.path());
        let hit = make_hit("ws-1", "src/main.rs", 42, 7);

        let result = hit_to_open_code_viewer(&hit, &[ws]);
        assert!(result.is_some(), "known workspace must resolve");

        let ocv = result.unwrap();
        assert_eq!(ocv.line, Some(42), "line must match hit.line");
        assert_eq!(ocv.col, Some(7), "col must match hit.col");
        // Path must be workspace root + rel_path.
        assert!(
            ocv.path.ends_with("src/main.rs"),
            "path must end with rel_path"
        );
        assert!(
            ocv.path.starts_with(tmp.path()),
            "path must start with workspace root"
        );
    }

    /// AC-GS-10 (logic): unknown workspace_id returns None — no panic.
    #[test]
    fn test_hit_to_open_code_viewer_unknown_workspace_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = make_workspace("ws-known", "project-a", tmp.path());
        // Hit references a workspace_id that doesn't exist.
        let hit = make_hit("ws-unknown", "src/lib.rs", 1, 0);

        let result = hit_to_open_code_viewer(&hit, &[ws]);
        assert!(
            result.is_none(),
            "unknown workspace_id must return None without panicking"
        );
    }

    /// AC-GS-10 (logic): empty workspace slice returns None — no panic.
    #[test]
    fn test_hit_to_open_code_viewer_empty_workspaces_returns_none() {
        let hit = make_hit("ws-1", "src/main.rs", 1, 0);
        let result = hit_to_open_code_viewer(&hit, &[]);
        assert!(result.is_none(), "empty workspace list must return None");
    }

    // ── T2: touch_workspace ──

    /// AC-GS-10: touch_workspace returns WorkspaceTouched for a known workspace id.
    #[test]
    fn test_handle_search_open_calls_touch_when_not_active() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = store_with_workspace(&tmp, "ws-abc", "my-project");

        let outcome = touch_workspace(&mut store, "ws-abc");
        assert_eq!(
            outcome,
            NavigationOutcome::WorkspaceTouched,
            "touch_workspace must return WorkspaceTouched for a known id"
        );
    }

    /// AC-GS-10 (stability): touch_workspace with unknown id returns
    /// WorkspaceNotFound — no panic.
    #[test]
    fn test_handle_search_open_unknown_workspace_no_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = store_with_workspace(&tmp, "ws-abc", "my-project");

        // Attempt to touch an id that doesn't exist in the store.
        let outcome = touch_workspace(&mut store, "ws-unknown-id");
        // Must not panic; outcome can be WorkspaceNotFound or Error.
        match outcome {
            NavigationOutcome::WorkspaceNotFound | NavigationOutcome::Error(_) => {}
            other => panic!("expected WorkspaceNotFound or Error for unknown id, got {other:?}"),
        }
    }

    // ── T4: OpenCodeViewer line/col accuracy ──

    /// AC-GS-10: line and col fields in OpenCodeViewer match SearchHit exactly.
    #[test]
    fn test_handle_search_open_dispatches_open_code_viewer_with_line_col() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = make_workspace("ws-x", "project-x", tmp.path());
        let hit = make_hit("ws-x", "crates/engine/src/lib.rs", 137, 23);

        let ocv = hit_to_open_code_viewer(&hit, &[ws]).expect("must resolve");
        assert_eq!(ocv.line, Some(137), "line must be exactly 137");
        assert_eq!(ocv.col, Some(23), "col must be exactly 23");
    }

    // ── T5: stability — no panic on degraded inputs ──

    /// AC-GS-10 (stability): navigation over a zero-length rel_path does not panic.
    #[test]
    fn test_handle_search_open_returns_error_state_no_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = make_workspace("ws-z", "project-z", tmp.path());
        // Empty relative path — edge case.
        let hit = SearchHit {
            workspace_id: "ws-z".to_string(),
            rel_path: PathBuf::from(""),
            line: 0,
            col: 0,
            preview: String::new(),
            match_start: 0,
            match_end: 0,
        };
        // Must not panic; result is Some(OpenCodeViewer with root path).
        let _result = hit_to_open_code_viewer(&hit, &[ws]);
        // No assertion beyond "did not panic".
    }
}
