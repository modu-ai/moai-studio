//! Integration tests for SPEC-V3-008 MS-2/MS-3 Git UI components.
//!
//! Pure logic tests for GitDiffViewer, GitBranchSwitcher, GitLogView,
//! GitMergeResolver, and GitStashPanel.
//! Separated from inline #[cfg(test)] modules to avoid gpui_macros
//! proc-macro stack overflow during lib test compilation.

use moai_git::{BranchInfo, CommitInfo, Diff, Hunk, Line, StashInfo};
use moai_studio_ui::git::{
    GitBranchSwitcher, GitDiffViewer, GitLogView, GitMergeResolver, GitStashPanel,
};

// ============================================================
// DiffViewer tests
// ============================================================

fn make_hunk(
    old_start: usize,
    old_lines: usize,
    new_start: usize,
    new_lines: usize,
    header: &str,
    lines: Vec<(char, &str)>,
) -> Hunk {
    Hunk {
        old_start,
        old_lines,
        new_start,
        new_lines,
        header: header.to_string(),
        lines: lines
            .into_iter()
            .map(|(prefix, content)| Line {
                prefix,
                content: content.to_string(),
            })
            .collect(),
    }
}

/// REQ-G-010: GitDiffViewer::new() creates with diff: None.
#[test]
fn diff_viewer_new_creates_with_no_diff() {
    let viewer = GitDiffViewer::new();
    assert!(viewer.diff().is_none(), "new() should have diff = None");
}

/// REQ-G-010: load_diff() updates state.
#[test]
fn diff_viewer_load_diff_updates_state() {
    let mut viewer = GitDiffViewer::new();
    let diff = Diff {
        path: "src/main.rs".to_string(),
        hunks: vec![],
    };
    viewer.load_diff(diff);
    assert!(viewer.diff().is_some());
    assert_eq!(viewer.diff().unwrap().path, "src/main.rs");
}

/// REQ-G-010: diff() returns a reference to the loaded diff.
#[test]
fn diff_viewer_accessor_returns_loaded_diff() {
    let mut viewer = GitDiffViewer::new();
    assert!(viewer.diff().is_none());
    let diff = Diff {
        path: "lib.rs".to_string(),
        hunks: vec![],
    };
    viewer.load_diff(diff);
    assert_eq!(viewer.diff().unwrap().path, "lib.rs");
}

/// REQ-G-011: compute_line_numbers correctly tracks old/new line numbers for mixed hunk.
#[test]
fn diff_viewer_line_numbers_for_mixed_hunk() {
    let hunk = make_hunk(
        10,
        3,
        10,
        4,
        "@@ -10,3 +10,4 @@",
        vec![
            (' ', "context"),
            ('-', "removed"),
            ('+', "added1"),
            ('+', "added2"),
            (' ', "context2"),
        ],
    );
    let nums = GitDiffViewer::compute_line_numbers(&hunk);
    assert_eq!(nums[0], (Some(10), Some(10)));
    assert_eq!(nums[1], (Some(11), None));
    assert_eq!(nums[2], (None, Some(11)));
    assert_eq!(nums[3], (None, Some(12)));
    assert_eq!(nums[4], (Some(12), Some(13)));
}

/// REQ-G-011: line numbers for all-removed hunk.
#[test]
fn diff_viewer_line_numbers_for_removed_only_hunk() {
    let hunk = make_hunk(
        5,
        2,
        5,
        0,
        "@@ -5,2 +5,0 @@",
        vec![('-', "gone1"), ('-', "gone2")],
    );
    let nums = GitDiffViewer::compute_line_numbers(&hunk);
    assert_eq!(nums[0], (Some(5), None));
    assert_eq!(nums[1], (Some(6), None));
}

/// REQ-G-011: line numbers for all-added hunk.
#[test]
fn diff_viewer_line_numbers_for_added_only_hunk() {
    let hunk = make_hunk(
        1,
        0,
        1,
        3,
        "@@ -1,0 +1,3 @@",
        vec![('+', "new1"), ('+', "new2"), ('+', "new3")],
    );
    let nums = GitDiffViewer::compute_line_numbers(&hunk);
    assert_eq!(nums[0], (None, Some(1)));
    assert_eq!(nums[1], (None, Some(2)));
    assert_eq!(nums[2], (None, Some(3)));
}

/// REQ-G-010: load_diff replaces previous diff.
#[test]
fn diff_viewer_load_diff_replaces_previous() {
    let mut viewer = GitDiffViewer::new();
    viewer.load_diff(Diff {
        path: "first.rs".to_string(),
        hunks: vec![],
    });
    assert_eq!(viewer.diff().unwrap().path, "first.rs");
    viewer.load_diff(Diff {
        path: "second.rs".to_string(),
        hunks: vec![],
    });
    assert_eq!(viewer.diff().unwrap().path, "second.rs");
}

// ============================================================
// BranchSwitcher tests
// ============================================================

fn make_branch(name: &str, is_head: bool, is_local: bool) -> BranchInfo {
    BranchInfo {
        name: name.to_string(),
        is_head,
        is_local,
    }
}

/// REQ-G-030: GitBranchSwitcher::new() creates empty state.
#[test]
fn branch_switcher_new_creates_empty_state() {
    let switcher = GitBranchSwitcher::new();
    assert!(switcher.branches().is_empty());
    assert_eq!(switcher.query(), "");
    assert!(!switcher.is_loading());
}

/// REQ-G-031: set_branches() populates the branch list.
#[test]
fn branch_switcher_set_branches_populates_list() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_branches(vec![
        make_branch("main", true, true),
        make_branch("develop", false, true),
    ]);
    assert_eq!(switcher.branches().len(), 2);
    assert_eq!(switcher.branches()[0].name, "main");
    assert_eq!(switcher.branches()[1].name, "develop");
}

/// REQ-G-032: set_query() updates the query string.
#[test]
fn branch_switcher_set_query_updates_query() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_query("feature".to_string());
    assert_eq!(switcher.query(), "feature");
}

/// REQ-G-033: filtered_branches() with empty query returns all branches (local first).
#[test]
fn branch_switcher_filtered_branches_empty_query_returns_all() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_branches(vec![
        make_branch("origin/main", false, false),
        make_branch("main", true, true),
        make_branch("feature/x", false, true),
    ]);
    let filtered = switcher.filtered_branches();
    assert_eq!(filtered.len(), 3);
    assert!(filtered[0].is_local);
    assert!(filtered[1].is_local);
    assert!(!filtered[2].is_local);
}

/// REQ-G-033: filtered_branches() with query filters by substring (case-insensitive).
#[test]
fn branch_switcher_filtered_branches_case_insensitive_filter() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_branches(vec![
        make_branch("main", true, true),
        make_branch("Feature/Auth", false, true),
        make_branch("feature/ui", false, true),
        make_branch("origin/feature/auth", false, false),
    ]);
    switcher.set_query("FEATURE".to_string());
    let filtered = switcher.filtered_branches();
    assert_eq!(filtered.len(), 3);
    assert_eq!(filtered[0].name, "Feature/Auth");
    assert_eq!(filtered[1].name, "feature/ui");
    assert_eq!(filtered[2].name, "origin/feature/auth");
}

/// REQ-G-033: filtered_branches() returns empty when no matches.
#[test]
fn branch_switcher_filtered_branches_no_match_returns_empty() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_branches(vec![
        make_branch("main", true, true),
        make_branch("develop", false, true),
    ]);
    switcher.set_query("nonexistent".to_string());
    assert!(switcher.filtered_branches().is_empty());
}

/// REQ-G-034: current_branch() returns the HEAD branch.
#[test]
fn branch_switcher_current_branch_returns_head() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_branches(vec![
        make_branch("main", true, true),
        make_branch("develop", false, true),
    ]);
    let head = switcher.current_branch();
    assert!(head.is_some());
    assert_eq!(head.unwrap().name, "main");
    assert!(head.unwrap().is_head);
}

/// REQ-G-034: current_branch() returns None when no HEAD branch.
#[test]
fn branch_switcher_current_branch_none_when_no_head() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_branches(vec![make_branch("branch-a", false, true)]);
    assert!(switcher.current_branch().is_none());
}

/// REQ-G-030: set_loading() toggles loading state.
#[test]
fn branch_switcher_set_loading_toggles_state() {
    let mut switcher = GitBranchSwitcher::new();
    assert!(!switcher.is_loading());
    switcher.set_loading(true);
    assert!(switcher.is_loading());
    switcher.set_loading(false);
    assert!(!switcher.is_loading());
}

/// REQ-G-031: set_branches() replaces previous branches.
#[test]
fn branch_switcher_set_branches_replaces_previous() {
    let mut switcher = GitBranchSwitcher::new();
    switcher.set_branches(vec![make_branch("a", false, true)]);
    assert_eq!(switcher.branches().len(), 1);
    switcher.set_branches(vec![
        make_branch("x", false, true),
        make_branch("y", false, true),
    ]);
    assert_eq!(switcher.branches().len(), 2);
    assert_eq!(switcher.branches()[0].name, "x");
}

// ============================================================
// GitLogView tests
// ============================================================

fn make_commit(short_id: &str, message: &str, author: &str) -> CommitInfo {
    CommitInfo {
        short_id: short_id.to_string(),
        oid: format!("{}{}", short_id, "0".repeat(40 - short_id.len())),
        message: message.to_string(),
        author: author.to_string(),
        email: format!("{}@example.com", author.to_lowercase()),
        time: 1700000000,
    }
}

/// REQ-G-040: GitLogView::new() creates with empty commits and loading=false.
#[test]
fn log_view_new_creates_empty_state() {
    let view = GitLogView::new();
    assert!(view.commits().is_empty());
    assert!(view.selected().is_none());
    assert!(!view.is_loading());
    assert!(!view.is_dirty());
}

/// REQ-G-040: set_commits() populates the commit list.
#[test]
fn log_view_set_commits_populates_list() {
    let mut view = GitLogView::new();
    view.set_commits(vec![
        make_commit("abc1234", "first commit", "Alice"),
        make_commit("def5678", "second commit", "Bob"),
    ]);
    assert_eq!(view.commits().len(), 2);
    assert_eq!(view.commits()[0].short_id, "abc1234");
    assert_eq!(view.commits()[1].message, "second commit");
}

/// REQ-G-041: set_loading() toggles loading state.
#[test]
fn log_view_set_loading_toggles_state() {
    let mut view = GitLogView::new();
    assert!(!view.is_loading());
    view.set_loading(true);
    assert!(view.is_loading());
    view.set_loading(false);
    assert!(!view.is_loading());
}

/// REQ-G-042: set_selected() updates the selected index.
#[test]
fn log_view_set_selected_updates_index() {
    let mut view = GitLogView::new();
    view.set_commits(vec![
        make_commit("aaa1111", "commit a", "Alice"),
        make_commit("bbb2222", "commit b", "Bob"),
    ]);
    assert!(view.selected().is_none());
    view.set_selected(Some(0));
    assert_eq!(view.selected(), Some(0));
    view.set_selected(Some(1));
    assert_eq!(view.selected(), Some(1));
    view.set_selected(None);
    assert!(view.selected().is_none());
}

/// REQ-G-043: selected_commit() returns the commit at selected index.
#[test]
fn log_view_selected_commit_returns_correct_commit() {
    let mut view = GitLogView::new();
    view.set_commits(vec![
        make_commit("aaa1111", "commit a", "Alice"),
        make_commit("bbb2222", "commit b", "Bob"),
    ]);
    assert!(view.selected_commit().is_none());
    view.set_selected(Some(0));
    let commit = view.selected_commit().expect("should have a commit");
    assert_eq!(commit.short_id, "aaa1111");
    view.set_selected(Some(1));
    let commit = view.selected_commit().expect("should have a commit");
    assert_eq!(commit.short_id, "bbb2222");
}

/// REQ-G-043: selected_commit() returns None when selected is out of bounds.
#[test]
fn log_view_selected_commit_none_when_out_of_bounds() {
    let mut view = GitLogView::new();
    view.set_commits(vec![make_commit("aaa1111", "commit a", "Alice")]);
    view.set_selected(Some(5));
    assert!(view.selected_commit().is_none());
}

/// REQ-G-044: set_dirty() toggles dirty (uncommitted changes) flag.
#[test]
fn log_view_set_dirty_toggles_flag() {
    let mut view = GitLogView::new();
    assert!(!view.is_dirty());
    view.set_dirty(true);
    assert!(view.is_dirty());
    view.set_dirty(false);
    assert!(!view.is_dirty());
}

/// REQ-G-040: set_commits() replaces previous commits.
#[test]
fn log_view_set_commits_replaces_previous() {
    let mut view = GitLogView::new();
    view.set_commits(vec![make_commit("aaa1111", "old", "Alice")]);
    assert_eq!(view.commits().len(), 1);
    view.set_commits(vec![
        make_commit("bbb2222", "new1", "Bob"),
        make_commit("ccc3333", "new2", "Carol"),
    ]);
    assert_eq!(view.commits().len(), 2);
    assert_eq!(view.commits()[0].short_id, "bbb2222");
}

// ============================================================
// GitMergeResolver tests
// ============================================================

/// REQ-G-050: GitMergeResolver::new() creates empty state.
#[test]
fn merge_resolver_new_creates_empty_state() {
    let resolver = GitMergeResolver::new();
    assert!(resolver.conflict_files().is_empty());
    assert!(resolver.current_file().is_none());
    assert!(resolver.ours().is_empty());
    assert!(resolver.theirs().is_empty());
    assert!(resolver.merged().is_empty());
}

/// REQ-G-051: set_conflict_files() populates the file list.
#[test]
fn merge_resolver_set_conflict_files_populates_list() {
    let mut resolver = GitMergeResolver::new();
    resolver.set_conflict_files(vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]);
    assert_eq!(resolver.conflict_files().len(), 2);
    assert_eq!(resolver.conflict_files()[0], "src/main.rs");
    assert_eq!(resolver.conflict_files()[1], "src/lib.rs");
}

/// REQ-G-052: select_file() sets current_file and initializes content stubs.
#[test]
fn merge_resolver_select_file_sets_current_and_stubs() {
    let mut resolver = GitMergeResolver::new();
    resolver.set_conflict_files(vec!["src/main.rs".to_string()]);
    resolver.select_file("src/main.rs".to_string());
    assert_eq!(resolver.current_file(), Some("src/main.rs"));
    assert_eq!(resolver.ours(), "<<<<<<< ours\nsrc/main.rs");
    assert_eq!(resolver.theirs(), ">>>>>>> theirs\nsrc/main.rs");
    assert_eq!(
        resolver.merged(),
        "<<<<<<< ours\nsrc/main.rs\n=======\n>>>>>>> theirs\nsrc/main.rs"
    );
}

/// REQ-G-053: accept_ours() replaces merged with ours content.
#[test]
fn merge_resolver_accept_ours_replaces_merged() {
    let mut resolver = GitMergeResolver::new();
    resolver.set_conflict_files(vec!["src/main.rs".to_string()]);
    resolver.select_file("src/main.rs".to_string());
    resolver.accept_ours();
    assert_eq!(resolver.merged(), resolver.ours());
}

/// REQ-G-054: accept_theirs() replaces merged with theirs content.
#[test]
fn merge_resolver_accept_theirs_replaces_merged() {
    let mut resolver = GitMergeResolver::new();
    resolver.set_conflict_files(vec!["src/main.rs".to_string()]);
    resolver.select_file("src/main.rs".to_string());
    resolver.accept_theirs();
    assert_eq!(resolver.merged(), resolver.theirs());
}

/// REQ-G-050: set_conflict_files() replaces previous files.
#[test]
fn merge_resolver_set_conflict_files_replaces_previous() {
    let mut resolver = GitMergeResolver::new();
    resolver.set_conflict_files(vec!["old.rs".to_string()]);
    assert_eq!(resolver.conflict_files().len(), 1);
    resolver.set_conflict_files(vec!["a.rs".to_string(), "b.rs".to_string()]);
    assert_eq!(resolver.conflict_files().len(), 2);
}

/// REQ-G-053/054: accept_ours/accept_theirs on no file selected is a no-op.
#[test]
fn merge_resolver_accept_without_file_is_noop() {
    let mut resolver = GitMergeResolver::new();
    resolver.accept_ours();
    assert!(resolver.merged().is_empty());
    resolver.accept_theirs();
    assert!(resolver.merged().is_empty());
}

// ============================================================
// GitStashPanel tests
// ============================================================

fn make_stash(index: usize, message: &str, branch: &str) -> StashInfo {
    StashInfo {
        index,
        message: message.to_string(),
        branch: branch.to_string(),
        oid: format!("{:040}", index),
    }
}

/// REQ-G-060: GitStashPanel::new() creates with empty stashes.
#[test]
fn stash_panel_new_creates_empty_state() {
    let panel = GitStashPanel::new();
    assert!(panel.stashes().is_empty());
    assert_eq!(panel.stash_count(), 0);
}

/// REQ-G-061: set_stashes() populates the stash list.
#[test]
fn stash_panel_set_stashes_populates_list() {
    let mut panel = GitStashPanel::new();
    panel.set_stashes(vec![
        make_stash(0, "WIP on feature", "feature/x"),
        make_stash(1, "experiment", "main"),
    ]);
    assert_eq!(panel.stashes().len(), 2);
    assert_eq!(panel.stashes()[0].message, "WIP on feature");
    assert_eq!(panel.stashes()[1].branch, "main");
}

/// REQ-G-062: stash_count() returns correct count.
#[test]
fn stash_panel_stash_count_returns_correct_count() {
    let mut panel = GitStashPanel::new();
    assert_eq!(panel.stash_count(), 0);
    panel.set_stashes(vec![
        make_stash(0, "stash 0", "main"),
        make_stash(1, "stash 1", "main"),
        make_stash(2, "stash 2", "develop"),
    ]);
    assert_eq!(panel.stash_count(), 3);
}

/// REQ-G-060: set_stashes() replaces previous stashes.
#[test]
fn stash_panel_set_stashes_replaces_previous() {
    let mut panel = GitStashPanel::new();
    panel.set_stashes(vec![make_stash(0, "old", "main")]);
    assert_eq!(panel.stash_count(), 1);
    panel.set_stashes(vec![]);
    assert_eq!(panel.stash_count(), 0);
}

/// REQ-G-063: empty state has no stashes.
#[test]
fn stash_panel_empty_state() {
    let panel = GitStashPanel::new();
    assert_eq!(panel.stash_count(), 0);
    assert!(panel.stashes().is_empty());
}
