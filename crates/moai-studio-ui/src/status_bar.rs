//! StatusBar — bottom 28pt status widget surface.
//!
//! SPEC-V3-006 MS-7 (audit F-4): state-bearing replacement of the prior
//! free function `status_bar()` in `lib.rs`. Skeleton-only — actual git /
//! LSP / agent broadcasting is follow-up work (see REQ-SB-MS7-3 in
//! `.moai/specs/SPEC-V3-006/spec.md`).
//!
//! Layout (left -> right):
//!   `[AgentPill?]  [git_label]  ·  moai-studio v{version}  [spacer]  [LspChip?]  ⌘K to search`
//!
//! When `StatusBarState::default()` is rendered, the layout matches the
//! pre-MS-7 static rendering (no agent pill, no LSP chip, "no git" label,
//! version text, ⌘K hint) — preserving REQ-SB-MS7-1 / REQ-SB-MS7-3.

use gpui::{IntoElement, ParentElement, Styled, div, px, rgb};

use crate::design::tokens as tok;

/// LSP server status as exposed in the status bar widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspState {
    /// Server is initialized and serving requests.
    Ready,
    /// Server is initializing or indexing the workspace.
    Indexing,
    /// Server reported a fatal error.
    Error,
    /// Binary not found in `$PATH` (graceful degradation, see SPEC-V3-006 MS-3).
    NotAvailable,
}

impl LspState {
    /// Compact human label rendered in the status bar.
    pub fn label(self) -> &'static str {
        match self {
            LspState::Ready => "ready",
            LspState::Indexing => "indexing",
            LspState::Error => "error",
            LspState::NotAvailable => "n/a",
        }
    }
}

/// Snapshot of widget data injected by external callers.
///
/// All fields default to `None` (no widget rendered) so the status bar
/// reverts to its pre-MS-7 static appearance when no caller has injected
/// state (REQ-SB-MS7-1 / REQ-SB-MS7-3).
#[derive(Debug, Default, Clone)]
pub struct StatusBarState {
    agent_mode: Option<String>,
    git_branch: Option<GitBranchState>,
    lsp_status: Option<LspStatusState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitBranchState {
    branch: String,
    dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LspStatusState {
    server: String,
    state: LspState,
}

impl StatusBarState {
    /// Set or replace the displayed agent mode label (e.g. "Plan", "Run").
    pub fn set_agent_mode(&mut self, mode: impl Into<String>) {
        self.agent_mode = Some(mode.into());
    }

    /// Hide the agent pill (revert to pre-MS-7 default).
    pub fn clear_agent_mode(&mut self) {
        self.agent_mode = None;
    }

    /// Set the displayed git branch with an optional dirty marker.
    pub fn set_git_branch(&mut self, branch: impl Into<String>, dirty: bool) {
        self.git_branch = Some(GitBranchState {
            branch: branch.into(),
            dirty,
        });
    }

    /// Clear the git branch (label returns to "no git").
    pub fn clear_git_branch(&mut self) {
        self.git_branch = None;
    }

    /// Set or replace the LSP status chip.
    pub fn set_lsp_status(&mut self, server: impl Into<String>, state: LspState) {
        self.lsp_status = Some(LspStatusState {
            server: server.into(),
            state,
        });
    }

    /// Hide the LSP status chip (REQ-SB-MS7-2).
    pub fn clear_lsp_status(&mut self) {
        self.lsp_status = None;
    }

    /// Returns the agent mode pill text, when injected.
    pub fn visible_agent_mode(&self) -> Option<&str> {
        self.agent_mode.as_deref()
    }

    /// Returns the git widget label ("branch" or "branch*" when dirty).
    pub fn visible_git_label(&self) -> Option<String> {
        self.git_branch.as_ref().map(|b| {
            if b.dirty {
                format!("{}*", b.branch)
            } else {
                b.branch.clone()
            }
        })
    }

    /// Returns the LSP chip label in "server · state" form, when injected.
    pub fn visible_lsp_label(&self) -> Option<String> {
        self.lsp_status
            .as_ref()
            .map(|s| format!("{} · {}", s.server, s.state.label()))
    }
}

/// Render the status bar with the supplied state.
///
/// See module docs for the layout. Visual properties (colors, spacing) match
/// the pre-MS-7 free function so default rendering is byte-equivalent in
/// element structure (REQ-SB-MS7-1).
pub fn render_status_bar(state: &StatusBarState) -> impl IntoElement {
    let git_label = state
        .visible_git_label()
        .unwrap_or_else(|| "no git".to_string());

    let mut root = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(28.))
        .px_3()
        .gap_3()
        .bg(rgb(tok::BG_ELEVATED))
        .border_t_1()
        .border_color(rgb(tok::BORDER_SUBTLE));

    if let Some(mode) = state.visible_agent_mode() {
        root = root.child(render_agent_pill(mode));
    }

    root = root
        .child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child(git_label),
        )
        .child(div().text_xs().text_color(rgb(tok::FG_DISABLED)).child("·"))
        .child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child(format!("moai-studio v{}", env!("CARGO_PKG_VERSION"))),
        )
        .child(div().flex_grow());

    if let Some(lsp_label) = state.visible_lsp_label() {
        root = root.child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child(lsp_label),
        );
    }

    root.child(
        div()
            .text_xs()
            .text_color(rgb(tok::FG_MUTED))
            .child("⌘K to search"),
    )
}

fn render_agent_pill(mode: &str) -> impl IntoElement {
    div()
        .px_2()
        .text_xs()
        .text_color(rgb(tok::FG_PRIMARY))
        .bg(rgb(tok::BG_PANEL))
        .child(mode.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-SB-1: default state preserves pre-MS-7 visible labels.
    #[test]
    fn default_state_has_no_widgets() {
        let state = StatusBarState::default();
        assert_eq!(state.visible_agent_mode(), None);
        assert_eq!(state.visible_git_label(), None);
        assert_eq!(state.visible_lsp_label(), None);
    }

    /// AC-SB-2: agent mode injection becomes visible.
    #[test]
    fn set_agent_mode_makes_pill_visible() {
        let mut state = StatusBarState::default();
        state.set_agent_mode("Plan");
        assert_eq!(state.visible_agent_mode(), Some("Plan"));
    }

    #[test]
    fn clear_agent_mode_hides_pill() {
        let mut state = StatusBarState::default();
        state.set_agent_mode("Run");
        state.clear_agent_mode();
        assert_eq!(state.visible_agent_mode(), None);
    }

    /// AC-SB-3: clean git branch shows raw branch name.
    #[test]
    fn set_git_branch_clean_shows_branch_only() {
        let mut state = StatusBarState::default();
        state.set_git_branch("main", false);
        assert_eq!(state.visible_git_label().as_deref(), Some("main"));
    }

    /// AC-SB-4: dirty git branch appends asterisk marker.
    #[test]
    fn set_git_branch_dirty_appends_asterisk() {
        let mut state = StatusBarState::default();
        state.set_git_branch("feature/x", true);
        assert_eq!(state.visible_git_label().as_deref(), Some("feature/x*"));
    }

    #[test]
    fn clear_git_branch_returns_to_no_git_default() {
        let mut state = StatusBarState::default();
        state.set_git_branch("dev", true);
        state.clear_git_branch();
        assert_eq!(state.visible_git_label(), None);
    }

    /// AC-SB-5: LSP status injection becomes visible with server name + state.
    #[test]
    fn set_lsp_status_ready_shows_server_and_label() {
        let mut state = StatusBarState::default();
        state.set_lsp_status("rust-analyzer", LspState::Ready);
        assert_eq!(
            state.visible_lsp_label().as_deref(),
            Some("rust-analyzer · ready")
        );
    }

    #[test]
    fn set_lsp_status_indexing_uses_indexing_label() {
        let mut state = StatusBarState::default();
        state.set_lsp_status("pyright", LspState::Indexing);
        assert_eq!(
            state.visible_lsp_label().as_deref(),
            Some("pyright · indexing")
        );
    }

    #[test]
    fn set_lsp_status_not_available_uses_na_label() {
        let mut state = StatusBarState::default();
        state.set_lsp_status("gopls", LspState::NotAvailable);
        assert_eq!(state.visible_lsp_label().as_deref(), Some("gopls · n/a"));
    }

    /// AC-SB-6: clear_lsp_status removes the chip.
    #[test]
    fn clear_lsp_status_hides_chip() {
        let mut state = StatusBarState::default();
        state.set_lsp_status("rust-analyzer", LspState::Ready);
        state.clear_lsp_status();
        assert_eq!(state.visible_lsp_label(), None);
    }

    /// Independent widget state — mutations on one widget do not affect others.
    #[test]
    fn mutations_are_independent_per_widget() {
        let mut state = StatusBarState::default();
        state.set_agent_mode("Sync");
        state.set_git_branch("main", true);
        state.set_lsp_status("rust-analyzer", LspState::Error);

        assert_eq!(state.visible_agent_mode(), Some("Sync"));
        assert_eq!(state.visible_git_label().as_deref(), Some("main*"));
        assert_eq!(
            state.visible_lsp_label().as_deref(),
            Some("rust-analyzer · error")
        );

        state.clear_lsp_status();
        assert_eq!(state.visible_agent_mode(), Some("Sync"));
        assert_eq!(state.visible_git_label().as_deref(), Some("main*"));
        assert_eq!(state.visible_lsp_label(), None);
    }

    /// Render returns an element without panicking for default state.
    #[test]
    fn render_default_state_does_not_panic() {
        let state = StatusBarState::default();
        let _ = render_status_bar(&state);
    }

    /// Render returns an element without panicking for fully populated state.
    #[test]
    fn render_populated_state_does_not_panic() {
        let mut state = StatusBarState::default();
        state.set_agent_mode("Plan");
        state.set_git_branch("feature/SPEC-V3-006-ms7-status-bar", true);
        state.set_lsp_status("rust-analyzer", LspState::Ready);
        let _ = render_status_bar(&state);
    }
}
