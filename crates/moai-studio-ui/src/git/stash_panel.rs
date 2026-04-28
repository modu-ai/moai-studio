//! GitStashPanel GPUI Entity — SPEC-V3-008 MS-3.
//!
//! Renders a stash list with action buttons for push/pop/drop.
//! REQ-G-060 ~ REQ-G-064: stash management UI.

use crate::design::tokens;
use gpui::*;
use moai_git::StashInfo;

/// GPUI Entity that displays a git stash list with action buttons.
///
/// Features:
/// - Stash rows: index, message, branch name
/// - "Push Stash" button at top
/// - Per-row "Pop" and "Drop" buttons (render-only, parent wires actual git calls)
/// - Empty state: "No stashes" message
///
/// # SPEC trace
/// - REQ-G-060: holds stash list
/// - REQ-G-061: set_stashes() populates stash list
/// - REQ-G-062: stash_count() returns current count
/// - REQ-G-063: empty state display
/// - REQ-G-064: action buttons (render-only)
pub struct GitStashPanel {
    /// Stash entries loaded from git stash list.
    stashes: Vec<StashInfo>,
}

impl Default for GitStashPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl GitStashPanel {
    /// Create a new GitStashPanel with empty state.
    pub fn new() -> Self {
        Self {
            stashes: Vec::new(),
        }
    }

    /// Set the stash list. Typically called after fetching from GitRepo.
    pub fn set_stashes(&mut self, stashes: Vec<StashInfo>) {
        self.stashes = stashes;
    }

    /// Returns a reference to all stashes.
    pub fn stashes(&self) -> &[StashInfo] {
        &self.stashes
    }

    /// Returns the number of stashes.
    pub fn stash_count(&self) -> usize {
        self.stashes.len()
    }
}

impl Render for GitStashPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut el = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(tokens::BG_PANEL));

        // Header with "Push Stash" button.
        el = el.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(rgb(tokens::BORDER_SUBTLE))
                .bg(rgb(tokens::BG_SURFACE))
                .child(
                    div()
                        .flex_grow()
                        .text_sm()
                        .text_color(rgb(tokens::FG_SECONDARY))
                        .child(format!("Stashes ({})", self.stash_count())),
                )
                .child(
                    div()
                        .px_3()
                        .py(px(4.))
                        .rounded_md()
                        .bg(rgb(tokens::ACCENT))
                        .text_xs()
                        .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                        .cursor_pointer()
                        .child("Push Stash"),
                ),
        );

        // Stash list or empty state.
        if self.stashes.is_empty() {
            el = el.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .flex_grow()
                    .text_sm()
                    .text_color(rgb(tokens::FG_MUTED))
                    .child("No stashes"),
            );
        } else {
            let mut list = div()
                .flex()
                .flex_col()
                .flex_grow()
                .px_2()
                .py_1()
                .gap(px(2.));

            for stash in &self.stashes {
                list = list.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .w_full()
                        .px_2()
                        .py(px(2.))
                        .rounded_md()
                        .hover(|s| s.bg(rgb(tokens::BG_ELEVATED)))
                        .cursor_pointer()
                        .text_xs()
                        .child(
                            div()
                                .w(px(24.))
                                .flex_shrink_0()
                                .text_color(rgb(tokens::FG_MUTED))
                                .child(format!("{}", stash.index)),
                        )
                        .child(
                            div()
                                .flex_grow()
                                .text_color(rgb(tokens::FG_PRIMARY))
                                .child(stash.message.clone()),
                        )
                        .child(
                            div()
                                .mx_2()
                                .text_color(rgb(tokens::FG_MUTED))
                                .child(stash.branch.clone()),
                        )
                        // Pop button (render-only).
                        .child(
                            div()
                                .px_2()
                                .py(px(1.))
                                .rounded_md()
                                .bg(rgb(tokens::semantic::INFO))
                                .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                                .cursor_pointer()
                                .child("Pop"),
                        )
                        // Drop button (render-only).
                        .child(
                            div()
                                .px_2()
                                .py(px(1.))
                                .rounded_md()
                                .bg(rgb(tokens::semantic::DANGER))
                                .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                                .cursor_pointer()
                                .child("Drop"),
                        ),
                );
            }
            el = el.child(list);
        }

        el.into_any_element()
    }
}
