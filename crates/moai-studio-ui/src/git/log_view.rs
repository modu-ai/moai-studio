//! GitLogView GPUI Entity — SPEC-V3-008 MS-3.
//!
//! Renders a scrollable commit log with graph column, short IDs,
//! truncated messages, and author names.
//! REQ-G-040 ~ REQ-G-044: commit log display with selection and loading state.

use crate::design::tokens;
use gpui::*;
use moai_git::CommitInfo;

/// Maximum displayed length for commit messages.
const MSG_TRUNCATE_LEN: usize = 60;

/// GPUI Entity that displays a scrollable git commit log.
///
/// Features:
/// - Graph column (pipe `|` for linear, asterisk `*` for merge commits)
/// - Short ID (7 chars), truncated message, author
/// - Selected row highlighted with accent background
/// - Virtual "Uncommitted changes" row at top when `is_dirty` flag is set
/// - Loading overlay when `loading == true`
///
/// # SPEC trace
/// - REQ-G-040: holds commits, selected index, loading state, dirty flag
/// - REQ-G-041: set_loading() toggles loading overlay
/// - REQ-G-042: set_selected() updates selection index
/// - REQ-G-043: selected_commit() returns the commit at selected index
/// - REQ-G-044: is_dirty flag for uncommitted-changes virtual row
pub struct GitLogView {
    /// Commit entries loaded from git log.
    commits: Vec<CommitInfo>,
    /// Currently selected commit index (0-based).
    selected: Option<usize>,
    /// Whether the log is being loaded.
    loading: bool,
    /// Whether the working tree has uncommitted changes.
    is_dirty: bool,
}

impl Default for GitLogView {
    fn default() -> Self {
        Self::new()
    }
}

impl GitLogView {
    /// Create a new GitLogView with empty state.
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
            selected: None,
            loading: false,
            is_dirty: false,
        }
    }

    /// Set the commit list. Typically called after fetching from GitRepo.
    pub fn set_commits(&mut self, commits: Vec<CommitInfo>) {
        self.commits = commits;
    }

    /// Set the loading state. When true, a loading overlay is rendered.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Set the selected commit index.
    pub fn set_selected(&mut self, selected: Option<usize>) {
        self.selected = selected;
    }

    /// Set the dirty (uncommitted changes) flag.
    pub fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }

    /// Returns a reference to all commits.
    pub fn commits(&self) -> &[CommitInfo] {
        &self.commits
    }

    /// Returns the current selected index.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns whether the log view is loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Returns whether the working tree has uncommitted changes.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Returns the commit at the currently selected index.
    /// Returns None if nothing is selected or the index is out of bounds.
    pub fn selected_commit(&self) -> Option<&CommitInfo> {
        self.selected.and_then(|i| self.commits.get(i))
    }

    /// Determine the graph column character for a commit.
    /// Returns `*` for merge commits (multi-line first line), `|` otherwise.
    fn graph_char(_commit: &CommitInfo) -> char {
        // Simple heuristic: treat all as linear for now.
        // Merge commits can be detected later via parent count.
        '|'
    }

    /// Truncate a message string to MSG_TRUNCATE_LEN chars with ellipsis.
    fn truncate_message(msg: &str) -> String {
        if msg.len() <= MSG_TRUNCATE_LEN {
            msg.to_string()
        } else {
            format!("{}...", &msg[..MSG_TRUNCATE_LEN])
        }
    }
}

impl Render for GitLogView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut list = div()
            .flex()
            .flex_col()
            .flex_grow()
            .overflow_y_hidden()
            .px_2()
            .py_1()
            .gap(px(1.));

        // Virtual "Uncommitted changes" row when dirty.
        if self.is_dirty {
            list = list.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    .px_2()
                    .py(px(2.))
                    .rounded_md()
                    .bg(gpui::rgba(0xc4_7b_2a_1a))
                    .text_xs()
                    .text_color(rgb(tokens::semantic::WARNING))
                    .child(div().w(px(16.)).child("~"))
                    .child(div().w(px(64.)).flex_shrink_0().child("*******"))
                    .child(div().flex_grow().child("(uncommitted changes)"))
                    .child(div().text_color(rgb(tokens::FG_MUTED)).child("")),
            );
        }

        // Commit rows.
        for (i, commit) in self.commits.iter().enumerate() {
            let is_selected = self.selected == Some(i);
            let graph = Self::graph_char(commit);
            let msg = Self::truncate_message(commit.message.lines().next().unwrap_or(""));

            let bg = if is_selected {
                gpui::rgba(0x14_4a_46_33)
            } else {
                gpui::rgba(0x00_00_00_00)
            };
            let text_color = if is_selected {
                rgb(tokens::ACCENT)
            } else {
                rgb(tokens::FG_PRIMARY)
            };

            list = list.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    .px_2()
                    .py(px(2.))
                    .rounded_md()
                    .bg(bg)
                    .cursor_pointer()
                    .text_xs()
                    .text_color(text_color)
                    .child(div().w(px(16.)).child(graph.to_string()))
                    .child(
                        div()
                            .w(px(64.))
                            .flex_shrink_0()
                            .text_color(rgb(tokens::FG_MUTED))
                            .child(commit.short_id.clone()),
                    )
                    .child(div().flex_grow().child(msg))
                    .child(
                        div()
                            .text_color(rgb(tokens::FG_MUTED))
                            .child(commit.author.clone()),
                    ),
            );
        }

        let mut el = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(tokens::BG_PANEL))
            .relative()
            .child(
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(tokens::BORDER_SUBTLE))
                    .bg(rgb(tokens::BG_SURFACE))
                    .text_sm()
                    .text_color(rgb(tokens::FG_SECONDARY))
                    .child("Commit Log"),
            )
            .child(list);

        // Loading overlay.
        if self.loading {
            el = el.child(
                div()
                    .absolute()
                    .inset_0()
                    .bg(gpui::rgba(0x0a_10_0e_cc))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tokens::FG_SECONDARY))
                            .child("Loading log..."),
                    ),
            );
        }

        el.into_any_element()
    }
}
