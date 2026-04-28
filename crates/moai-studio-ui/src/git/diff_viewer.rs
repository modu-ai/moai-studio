//! GitDiffViewer GPUI Entity — SPEC-V3-008 MS-2.
//!
//! Renders a unified diff view with colored hunks and line numbers.
//! REQ-G-010 ~ REQ-G-015: diff display with removed (red) / added (green) / context lines.
//! REQ-G-080: plain text fallback (no syntax highlight dependency).

use crate::design::tokens;
use gpui::*;
use moai_git::{Diff, Hunk};

// Diff line background colors (rgba with alpha for subtle highlighting).
/// Removed line background: red with ~10% opacity.
const DIFF_REMOVED_BG: u32 = 0xef_44_44_1a;
/// Added line background: green with ~10% opacity.
const DIFF_ADDED_BG: u32 = 0x22_c5_5e_1a;
/// Hunk header background: subtle accent.
const HUNK_HEADER_BG: u32 = 0x14_4a_46_1a;

/// GPUI Entity that displays a unified diff view for a single file.
///
/// Holds an optional `moai_git::Diff` and renders hunks/lines with
/// colored backgrounds and line numbers on both sides.
///
/// # SPEC trace
/// - REQ-G-010: load_diff() accepts moai_git::Diff
/// - REQ-G-011: renders hunks with line numbers
/// - REQ-G-012: removed lines with red background
/// - REQ-G-013: added lines with green background
/// - REQ-G-014: context lines with default background
/// - REQ-G-015: scroll offset state for large diffs
/// - REQ-G-080: plain text only (no syntax highlight)
pub struct GitDiffViewer {
    /// The loaded diff. None means no diff loaded (empty state).
    diff: Option<Diff>,
    /// Vertical scroll offset for the diff content.
    scroll_offset: f32,
}

impl Default for GitDiffViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl GitDiffViewer {
    /// Create a new GitDiffViewer with no diff loaded.
    pub fn new() -> Self {
        Self {
            diff: None,
            scroll_offset: 0.0,
        }
    }

    /// Load a diff into the viewer, replacing any existing content.
    /// Resets scroll offset to the top.
    pub fn load_diff(&mut self, diff: Diff) {
        self.scroll_offset = 0.0;
        self.diff = Some(diff);
    }

    /// Returns a reference to the currently loaded diff, if any.
    pub fn diff(&self) -> Option<&Diff> {
        self.diff.as_ref()
    }

    /// Compute line numbers for each line in a hunk.
    /// Returns (old_line, new_line) pairs. None when the line doesn't exist on that side.
    pub fn compute_line_numbers(hunk: &Hunk) -> Vec<(Option<usize>, Option<usize>)> {
        let mut old_line = hunk.old_start;
        let mut new_line = hunk.new_start;
        let mut result = Vec::with_capacity(hunk.lines.len());

        for line in &hunk.lines {
            match line.prefix {
                '-' => {
                    result.push((Some(old_line), None));
                    old_line += 1;
                }
                '+' => {
                    result.push((None, Some(new_line)));
                    new_line += 1;
                }
                _ => {
                    result.push((Some(old_line), Some(new_line)));
                    old_line += 1;
                    new_line += 1;
                }
            }
        }

        result
    }

    /// Determine background and foreground color for a diff line prefix.
    pub fn line_colors(prefix: char) -> (u32, u32) {
        match prefix {
            '-' => (DIFF_REMOVED_BG, tokens::semantic::DANGER),
            '+' => (DIFF_ADDED_BG, tokens::semantic::SUCCESS),
            _ => (0x00_00_00_00, tokens::FG_PRIMARY),
        }
    }
}

impl Render for GitDiffViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let base = div()
            .size_full()
            .overflow_y_hidden()
            .bg(rgb(tokens::BG_APP))
            .flex()
            .flex_col();

        match &self.diff {
            None => base
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .size_full()
                        .text_sm()
                        .text_color(rgb(tokens::FG_MUTED))
                        .child("No diff loaded"),
                )
                .into_any_element(),
            Some(diff) => {
                let mut el = base.child(
                    div()
                        .w_full()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(rgb(tokens::BORDER_SUBTLE))
                        .bg(rgb(tokens::BG_SURFACE))
                        .text_xs()
                        .text_color(rgb(tokens::FG_SECONDARY))
                        .child(diff.path.clone()),
                );

                for hunk in &diff.hunks {
                    let line_numbers = Self::compute_line_numbers(hunk);
                    el = el.child(
                        div()
                            .w_full()
                            .px_3()
                            .py(px(2.))
                            .bg(gpui::rgba(HUNK_HEADER_BG))
                            .text_xs()
                            .text_color(rgb(tokens::ACCENT))
                            .child(hunk.header.clone()),
                    );
                    for (i, line) in hunk.lines.iter().enumerate() {
                        let (old_num, new_num) =
                            line_numbers.get(i).copied().unwrap_or((None, None));
                        let (bg, fg) = Self::line_colors(line.prefix);
                        let old_text = old_num.map_or(String::new(), |n| format!("{:>4}", n));
                        let new_text = new_num.map_or(String::new(), |n| format!("{:>4}", n));
                        let mut row = div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .w_full()
                            .px_2()
                            .py(px(1.))
                            .text_xs()
                            .child(
                                div()
                                    .w(px(48.))
                                    .flex_shrink_0()
                                    .text_color(rgb(tokens::FG_MUTED))
                                    .child(old_text),
                            )
                            .child(
                                div()
                                    .w(px(48.))
                                    .flex_shrink_0()
                                    .text_color(rgb(tokens::FG_MUTED))
                                    .child(new_text),
                            )
                            .child(
                                div()
                                    .w(px(16.))
                                    .flex_shrink_0()
                                    .text_color(rgb(fg))
                                    .child(line.prefix.to_string()),
                            )
                            .child(
                                div()
                                    .flex_grow()
                                    .text_color(rgb(fg))
                                    .child(line.content.clone()),
                            );
                        if bg != 0 {
                            row = row.bg(gpui::rgba(bg));
                        }
                        el = el.child(row);
                    }
                }
                el.into_any_element()
            }
        }
    }
}
