//! GitBranchSwitcher GPUI Entity — SPEC-V3-008 MS-2.
//!
//! Renders a searchable branch list with fuzzy filtering.
//! REQ-G-030 ~ REQ-G-035: branch listing, fuzzy search, local/remote separation.

use crate::design::tokens;
use gpui::*;
use moai_git::BranchInfo;

/// GPUI Entity that displays a searchable, scrollable branch list.
///
/// Features:
/// - Fuzzy filter via case-insensitive substring matching
/// - Current branch highlighted with accent color
/// - Local branches listed first, then remote branches
/// - Loading spinner overlay when `loading == true`
///
/// # SPEC trace
/// - REQ-G-030: holds branches, query, loading state
/// - REQ-G-031: set_branches() populates branch list
/// - REQ-G-032: set_query() triggers fuzzy filter
/// - REQ-G-033: filtered_branches() returns matching branches
/// - REQ-G-034: current branch (is_head=true) identified
/// - REQ-G-035: render with search input + scrollable list
pub struct GitBranchSwitcher {
    /// All known branches.
    branches: Vec<BranchInfo>,
    /// Current search query string.
    query: String,
    /// Whether the branch list is currently being loaded.
    loading: bool,
}

impl Default for GitBranchSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl GitBranchSwitcher {
    /// Create a new GitBranchSwitcher with empty state.
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            query: String::new(),
            loading: false,
        }
    }

    /// Set the full branch list. Typically called after fetching from GitRepo.
    pub fn set_branches(&mut self, branches: Vec<BranchInfo>) {
        self.branches = branches;
    }

    /// Update the search query for fuzzy filtering.
    pub fn set_query(&mut self, query: String) {
        self.query = query;
    }

    /// Set the loading state. When true, a loading overlay is rendered.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Returns a reference to all branches.
    pub fn branches(&self) -> &[BranchInfo] {
        &self.branches
    }

    /// Returns the current query string.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns whether the switcher is in a loading state.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Return filtered branches matching the current query.
    ///
    /// Filter is case-insensitive substring match on branch name.
    /// Results are ordered: local branches first, then remote.
    /// Within each group, the original insertion order is preserved.
    pub fn filtered_branches(&self) -> Vec<&BranchInfo> {
        let query_lower = self.query.to_lowercase();
        let mut local = Vec::new();
        let mut remote = Vec::new();

        for branch in &self.branches {
            if !query_lower.is_empty() {
                let name_lower = branch.name.to_lowercase();
                if !name_lower.contains(&query_lower) {
                    continue;
                }
            }
            if branch.is_local {
                local.push(branch);
            } else {
                remote.push(branch);
            }
        }

        local.extend(remote);
        local
    }

    /// Find the current (HEAD) branch from the full list.
    pub fn current_branch(&self) -> Option<&BranchInfo> {
        self.branches.iter().find(|b| b.is_head)
    }
}

impl Render for GitBranchSwitcher {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let filtered = self.filtered_branches();

        let search_text = if self.query.is_empty() {
            "Search branches...".to_string()
        } else {
            self.query.clone()
        };
        let search_color = if self.query.is_empty() {
            rgb(tokens::FG_MUTED)
        } else {
            rgb(tokens::FG_PRIMARY)
        };

        let mut list = div()
            .flex()
            .flex_col()
            .flex_grow()
            .overflow_y_hidden()
            .px_2()
            .py_1()
            .gap(px(2.));

        for branch in filtered {
            let is_head = branch.is_head;
            let label = if branch.is_local {
                branch.name.clone()
            } else {
                format!("remote: {}", branch.name)
            };
            let name_color = if is_head {
                rgb(tokens::ACCENT)
            } else {
                rgb(tokens::FG_PRIMARY)
            };

            let mut row = div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .px_3()
                .py(px(4.))
                .rounded_md()
                .hover(|s| s.bg(rgb(tokens::BG_ELEVATED)))
                .cursor_pointer()
                .child(
                    div()
                        .flex_grow()
                        .text_sm()
                        .text_color(name_color)
                        .child(label),
                );

            if is_head {
                row = row.bg(gpui::rgba(0x14_4a_46_33)).child(
                    div()
                        .text_xs()
                        .text_color(rgb(tokens::ACCENT))
                        .child("HEAD"),
                );
            }

            list = list.child(row);
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
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .flex_grow()
                            .text_sm()
                            .text_color(search_color)
                            .child(search_text),
                    ),
            )
            .child(list);

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
                            .child("Loading branches..."),
                    ),
            );
        }

        el.into_any_element()
    }
}
