//! GitMergeResolver GPUI Entity — SPEC-V3-008 MS-3.
//!
//! Renders a 3-panel merge conflict resolver (ours | merged | theirs)
//! with file tabs and action buttons.
//! REQ-G-050 ~ REQ-G-056: conflict resolution UI.

use crate::design::tokens;
use gpui::*;

/// GPUI Entity that displays a merge conflict resolver.
///
/// Features:
/// - File tabs at top for multi-file conflicts
/// - 3-panel layout: ours | merged | theirs
/// - "Accept Ours", "Accept Theirs", "Mark Resolved", "Abort Merge" buttons
/// - Plain text display in each panel (read-only in v1.0.0)
///
/// # SPEC trace
/// - REQ-G-050: holds conflict_files, current_file, ours/theirs/merged content
/// - REQ-G-051: set_conflict_files() populates file list
/// - REQ-G-052: select_file() sets current file and loads content stubs
/// - REQ-G-053: accept_ours() replaces merged with ours
/// - REQ-G-054: accept_theirs() replaces merged with theirs
/// - REQ-G-055/056: action buttons (render-only, parent wires actual git calls)
pub struct GitMergeResolver {
    /// List of conflicting file paths.
    conflict_files: Vec<String>,
    /// Currently selected conflict file.
    current_file: Option<String>,
    /// Content from "ours" side.
    ours: String,
    /// Content from "theirs" side.
    theirs: String,
    /// Merged (resolved) content.
    merged: String,
}

impl Default for GitMergeResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl GitMergeResolver {
    /// Create a new GitMergeResolver with empty state.
    pub fn new() -> Self {
        Self {
            conflict_files: Vec::new(),
            current_file: None,
            ours: String::new(),
            theirs: String::new(),
            merged: String::new(),
        }
    }

    /// Set the list of conflicting file paths.
    pub fn set_conflict_files(&mut self, files: Vec<String>) {
        self.conflict_files = files;
    }

    /// Select a conflict file and initialize content stubs for each panel.
    /// In v1.0.0, stubs are generated from the file path for demonstration.
    pub fn select_file(&mut self, path: String) {
        self.ours = format!("<<<<<<< ours\n{}", path);
        self.theirs = format!(">>>>>>> theirs\n{}", path);
        self.merged = format!("<<<<<<< ours\n{}\n=======\n>>>>>>> theirs\n{}", path, path);
        self.current_file = Some(path);
    }

    /// Accept "ours" version: replace merged content with ours.
    pub fn accept_ours(&mut self) {
        self.merged = self.ours.clone();
    }

    /// Accept "theirs" version: replace merged content with theirs.
    pub fn accept_theirs(&mut self) {
        self.merged = self.theirs.clone();
    }

    /// Returns the list of conflict files.
    pub fn conflict_files(&self) -> &[String] {
        &self.conflict_files
    }

    /// Returns the currently selected file path.
    pub fn current_file(&self) -> Option<&str> {
        self.current_file.as_deref()
    }

    /// Returns the "ours" content.
    pub fn ours(&self) -> &str {
        &self.ours
    }

    /// Returns the "theirs" content.
    pub fn theirs(&self) -> &str {
        &self.theirs
    }

    /// Returns the merged content.
    pub fn merged(&self) -> &str {
        &self.merged
    }
}

impl Render for GitMergeResolver {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut el = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(tokens::BG_PANEL));

        // File tabs.
        if !self.conflict_files.is_empty() {
            let mut tabs = div()
                .flex()
                .flex_row()
                .w_full()
                .px_2()
                .py_1()
                .border_b_1()
                .border_color(rgb(tokens::BORDER_SUBTLE))
                .bg(rgb(tokens::BG_SURFACE))
                .gap(px(4.));

            for file in &self.conflict_files {
                let is_active = self.current_file.as_deref() == Some(file.as_str());
                let tab_bg = if is_active {
                    rgb(tokens::ACCENT)
                } else {
                    rgb(tokens::BG_ELEVATED)
                };
                let tab_fg = if is_active {
                    rgb(tokens::theme::dark::text::ON_PRIMARY)
                } else {
                    rgb(tokens::FG_SECONDARY)
                };

                tabs = tabs.child(
                    div()
                        .px_2()
                        .py(px(2.))
                        .rounded_md()
                        .bg(tab_bg)
                        .text_xs()
                        .text_color(tab_fg)
                        .cursor_pointer()
                        .child(file.clone()),
                );
            }
            el = el.child(tabs);
        }

        // 3-panel layout: ours | merged | theirs.
        // Inline panel construction to avoid lifetime issues with closures.
        let make_panel = |label: &str, content: &str| -> Div {
            let mut panel = div()
                .flex()
                .flex_col()
                .flex_grow()
                .border_r_1()
                .border_color(rgb(tokens::BORDER_SUBTLE));

            let label_el = div()
                .w_full()
                .px_2()
                .py(px(2.))
                .border_b_1()
                .border_color(rgb(tokens::BORDER_SUBTLE))
                .bg(rgb(tokens::BG_SURFACE))
                .text_xs()
                .text_color(rgb(tokens::FG_SECONDARY))
                .child(label.to_string());

            let content_el = div()
                .flex_grow()
                .px_2()
                .py_1()
                .text_xs()
                .text_color(rgb(tokens::FG_PRIMARY))
                .overflow_y_hidden()
                .child(content.to_string());

            panel = panel.child(label_el).child(content_el);
            panel
        };

        let panels = div()
            .flex()
            .flex_row()
            .flex_grow()
            .child(make_panel("Ours", &self.ours))
            .child(make_panel("Merged", &self.merged))
            .child(make_panel("Theirs", &self.theirs));

        el = el.child(panels);

        // Action buttons.
        let buttons = div()
            .flex()
            .flex_row()
            .w_full()
            .px_3()
            .py_2()
            .border_t_1()
            .border_color(rgb(tokens::BORDER_SUBTLE))
            .bg(rgb(tokens::BG_SURFACE))
            .gap(px(8.))
            .child(
                div()
                    .px_3()
                    .py(px(4.))
                    .rounded_md()
                    .bg(rgb(tokens::ACCENT))
                    .text_xs()
                    .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                    .cursor_pointer()
                    .child("Accept Ours"),
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
                    .child("Accept Theirs"),
            )
            .child(
                div()
                    .px_3()
                    .py(px(4.))
                    .rounded_md()
                    .bg(rgb(tokens::semantic::SUCCESS))
                    .text_xs()
                    .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                    .cursor_pointer()
                    .child("Mark Resolved"),
            )
            .child(
                div()
                    .px_3()
                    .py(px(4.))
                    .rounded_md()
                    .bg(rgb(tokens::semantic::DANGER))
                    .text_xs()
                    .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                    .cursor_pointer()
                    .child("Abort Merge"),
            );

        el = el.child(buttons);

        el.into_any_element()
    }
}
