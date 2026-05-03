//! Shell Picker logic — SPEC-V0-2-0-MULTI-SHELL-001.
//!
//! `ShellPicker` is a pure-logic struct (no GPUI dependency) that tracks
//! the list of available shells, the current user selection, and the system
//! default.  It is owned by `RootView::shell_picker` and activated by
//! `handle_switch_shell`.
//!
//! GUI overlay (dropdown / dialog) is a v0.2.1 carry-over (N5 non-goal).

// @MX:ANCHOR: [AUTO] ShellPicker — shell selection logic struct.
// @MX:REASON: [AUTO] fan_in >= 3: RootView::handle_switch_shell (create),
//   dispatch_command_shell_switch (activate), unit test suite.
// @MX:SPEC: SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-005

use moai_studio_terminal::shell::Shell;

// ============================================================
// ShellPicker
// ============================================================

/// Logic-level shell picker state.
///
/// Tracks which shells are available on the current system, which one the
/// user has selected, and what the system default is (`$SHELL` resolved to
/// a `Shell` variant, if known).
///
/// This struct deliberately has no GPUI dependency so it can be tested
/// without a running application context (Spike 2 pattern).
pub struct ShellPicker {
    /// Available shells detected via `Shell::detect_available()`.
    available: Vec<Shell>,
    /// System default shell (derived from `$SHELL`, may be `None` if
    /// the env-var is absent or maps to an unknown variant).
    current_default: Option<Shell>,
    /// User's active selection.  `None` means no selection has been made
    /// yet (current session will fall back to `current_default`).
    selected: Option<Shell>,
}

impl ShellPicker {
    /// Create a new `ShellPicker` with the given list and optional default.
    ///
    /// `available` should come from `Shell::detect_available()`.
    /// `current_default` is typically derived from the `$SHELL` env-var
    /// resolved to a `Shell` variant.
    pub fn new(available: Vec<Shell>, current_default: Option<Shell>) -> Self {
        Self {
            available,
            current_default,
            selected: None,
        }
    }

    /// Set the user's active shell selection.
    ///
    /// Returns the selected shell after the update.
    pub fn select(&mut self, shell: Shell) -> Option<Shell> {
        self.selected = Some(shell);
        self.selected
    }

    /// Return the currently active shell.
    ///
    /// Prefers an explicit user `selected` shell; falls back to
    /// `current_default` when no selection has been made.
    pub fn current(&self) -> Option<Shell> {
        self.selected.or(self.current_default)
    }

    /// Return the slice of available shells.
    pub fn available(&self) -> &[Shell] {
        &self.available
    }

    /// Return the system default shell.
    pub fn default_shell(&self) -> Option<Shell> {
        self.current_default
    }
}

// ============================================================
// Unit tests — AC-MS-5
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-MS-5 (REQ-MS-005): new() creates picker with no active selection.
    #[test]
    fn test_shell_picker_new_no_selection() {
        let picker = ShellPicker::new(vec![Shell::Bash, Shell::Zsh], Some(Shell::Bash));
        assert!(
            picker.selected.is_none(),
            "new ShellPicker must have no user selection"
        );
        assert_eq!(
            picker.available(),
            &[Shell::Bash, Shell::Zsh],
            "available() must reflect the constructor argument"
        );
        assert_eq!(
            picker.default_shell(),
            Some(Shell::Bash),
            "default_shell() must reflect the constructor argument"
        );
    }

    /// AC-MS-5: select(Shell::Bash) sets current() to Some(Bash).
    #[test]
    fn test_shell_picker_select_sets_current() {
        let mut picker = ShellPicker::new(vec![Shell::Bash, Shell::Zsh], None);
        let result = picker.select(Shell::Bash);
        assert_eq!(
            result,
            Some(Shell::Bash),
            "select() must return the selected shell"
        );
        assert_eq!(
            picker.current(),
            Some(Shell::Bash),
            "current() must return the selected shell after select()"
        );
    }

    /// AC-MS-5: current() falls back to current_default when no selection made.
    #[test]
    fn test_shell_picker_current_returns_selected_or_default() {
        // No selection → fall back to default.
        let picker = ShellPicker::new(vec![Shell::Zsh], Some(Shell::Zsh));
        assert_eq!(
            picker.current(),
            Some(Shell::Zsh),
            "current() must fall back to current_default when no selection"
        );

        // After selection → prefer selected over default.
        let mut picker2 = ShellPicker::new(vec![Shell::Zsh, Shell::Bash], Some(Shell::Zsh));
        picker2.select(Shell::Bash);
        assert_eq!(
            picker2.current(),
            Some(Shell::Bash),
            "current() must prefer selected over current_default"
        );
    }

    /// current() returns None when both selected and default are absent.
    #[test]
    fn test_shell_picker_current_none_when_no_default_or_selection() {
        let picker = ShellPicker::new(vec![Shell::Fish], None);
        assert!(
            picker.current().is_none(),
            "current() must be None when neither selected nor default is set"
        );
    }
}
