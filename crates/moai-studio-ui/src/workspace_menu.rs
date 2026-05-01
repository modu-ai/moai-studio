//! Sidebar workspace switcher context menu — skeleton (audit D-2 polish).
//!
//! SPEC-V3-004 MS-4 (audit D-2): adds the structural pieces required to
//! surface a right-click context menu on each `workspace_row` in the
//! sidebar. The menu exposes 4 actions (Rename / Delete / MoveUp /
//! MoveDown). This module is intentionally a skeleton — actual rename
//! modal, delete confirmation, and reorder dispatch are follow-up work
//! (REQ-D2-MS4-3). Real wiring will mutate `RootView.workspaces` /
//! `WorkspacesStore` in a separate PR.
//!
//! Pattern mirrors `crate::explorer::context_menu::ContextMenu` so the
//! same idiom (target + items + visible position) carries through across
//! the codebase.

/// Distinct user actions exposed in the workspace context menu.
///
/// Mirrors the four operations called out by audit D-2:
/// "Missing: drag-to-reorder, context menu (rename/delete)".
/// Reorder is split into `MoveUp` / `MoveDown` keyboard-friendly actions
/// while drag-to-reorder remains deferred to v0.2.0 (audit line 147).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMenuAction {
    /// Open a rename modal for the targeted workspace.
    Rename,
    /// Confirm-then-delete the targeted workspace.
    Delete,
    /// Move the workspace one position upwards in the sidebar list.
    MoveUp,
    /// Move the workspace one position downwards in the sidebar list.
    MoveDown,
}

impl WorkspaceMenuAction {
    /// Canonical ordering of the actions as they appear in the menu.
    pub fn all() -> [WorkspaceMenuAction; 4] {
        [
            WorkspaceMenuAction::Rename,
            WorkspaceMenuAction::Delete,
            WorkspaceMenuAction::MoveUp,
            WorkspaceMenuAction::MoveDown,
        ]
    }

    /// Compact human label rendered as the menu item text.
    pub fn label(self) -> &'static str {
        match self {
            WorkspaceMenuAction::Rename => "Rename",
            WorkspaceMenuAction::Delete => "Delete",
            WorkspaceMenuAction::MoveUp => "Move Up",
            WorkspaceMenuAction::MoveDown => "Move Down",
        }
    }

    /// True for actions that mutate workspace identity / data and therefore
    /// SHOULD be guarded by a confirmation step in the follow-up wiring PR.
    /// Skeleton callers can use this to plan the future modal flow without
    /// hard-coding action identities at the dispatch site.
    pub fn is_destructive(self) -> bool {
        matches!(self, WorkspaceMenuAction::Delete)
    }
}

/// Visible-on-screen position (logical px) — populated when the menu opens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuPosition {
    pub x: f32,
    pub y: f32,
}

/// Workspace context menu state.
///
/// Lives outside `RootView` so the orchestration layer can drive open /
/// close transitions without entangling with sidebar render code. Default
/// state is closed (no target). When opened for a different workspace the
/// previous target is replaced atomically — preserves the
/// "single visible menu at a time" invariant required by REQ-D2-MS4-2.
#[derive(Debug, Default, Clone)]
pub struct WorkspaceMenu {
    target: Option<String>,
    position: Option<MenuPosition>,
}

impl WorkspaceMenu {
    /// Open the menu for `workspace_id` at the supplied screen position.
    ///
    /// Replaces any prior target — there is at most one open menu.
    pub fn open_for(&mut self, workspace_id: impl Into<String>, x: f32, y: f32) {
        self.target = Some(workspace_id.into());
        self.position = Some(MenuPosition { x, y });
    }

    /// Close the menu (no target, no position).
    pub fn close(&mut self) {
        self.target = None;
        self.position = None;
    }

    /// Returns the workspace id the menu is currently open for, if any.
    pub fn visible_target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Returns the visible position, if the menu is open.
    pub fn visible_position(&self) -> Option<MenuPosition> {
        self.position
    }

    /// True when the menu is currently open for `workspace_id`.
    pub fn is_visible_for(&self, workspace_id: &str) -> bool {
        self.target.as_deref() == Some(workspace_id)
    }

    /// True when any workspace currently has the menu open.
    pub fn is_open(&self) -> bool {
        self.target.is_some()
    }

    /// Returns the canonical action list to render inside the menu.
    /// Pure helper — kept as an associated function so the renderer can
    /// reuse this without owning a `WorkspaceMenu` instance.
    pub fn items() -> [WorkspaceMenuAction; 4] {
        WorkspaceMenuAction::all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-D2-1: all four actions are exposed and have non-empty labels.
    #[test]
    fn workspace_menu_action_all_returns_four_distinct_variants() {
        let actions = WorkspaceMenuAction::all();
        assert_eq!(actions.len(), 4);
        let labels: Vec<&'static str> = actions.iter().map(|a| a.label()).collect();
        assert_eq!(
            labels,
            vec!["Rename", "Delete", "Move Up", "Move Down"],
            "label order must match canonical enum order"
        );
        // Each label must be unique and non-empty.
        let mut seen = std::collections::HashSet::new();
        for label in &labels {
            assert!(!label.is_empty(), "label must not be empty");
            assert!(seen.insert(*label), "label '{label}' must be unique");
        }
    }

    #[test]
    fn workspace_menu_action_label_covers_all_variants() {
        for action in WorkspaceMenuAction::all() {
            let label = action.label();
            assert!(!label.is_empty(), "{action:?} label must not be empty");
        }
    }

    #[test]
    fn workspace_menu_action_destructive_only_for_delete() {
        assert!(WorkspaceMenuAction::Delete.is_destructive());
        assert!(!WorkspaceMenuAction::Rename.is_destructive());
        assert!(!WorkspaceMenuAction::MoveUp.is_destructive());
        assert!(!WorkspaceMenuAction::MoveDown.is_destructive());
    }

    /// AC-D2-2: default state is closed.
    #[test]
    fn default_menu_is_closed() {
        let menu = WorkspaceMenu::default();
        assert!(!menu.is_open());
        assert_eq!(menu.visible_target(), None);
        assert_eq!(menu.visible_position(), None);
        assert!(!menu.is_visible_for("ws-1"));
    }

    /// AC-D2-3: open_for stores target + position and exposes is_visible_for.
    #[test]
    fn open_for_makes_menu_visible_for_specific_workspace() {
        let mut menu = WorkspaceMenu::default();
        menu.open_for("ws-1", 100.0, 200.0);

        assert!(menu.is_open());
        assert!(menu.is_visible_for("ws-1"));
        assert!(!menu.is_visible_for("ws-2"));
        assert_eq!(menu.visible_target(), Some("ws-1"));
        assert_eq!(
            menu.visible_position(),
            Some(MenuPosition { x: 100.0, y: 200.0 })
        );
    }

    /// AC-D2-4: opening for a second workspace replaces the prior target.
    #[test]
    fn open_for_second_workspace_replaces_prior_target() {
        let mut menu = WorkspaceMenu::default();
        menu.open_for("ws-1", 10.0, 20.0);
        menu.open_for("ws-2", 30.0, 40.0);

        assert!(menu.is_open());
        assert!(!menu.is_visible_for("ws-1"));
        assert!(menu.is_visible_for("ws-2"));
        assert_eq!(menu.visible_target(), Some("ws-2"));
        assert_eq!(
            menu.visible_position(),
            Some(MenuPosition { x: 30.0, y: 40.0 })
        );
    }

    /// AC-D2-5: close clears target and position.
    #[test]
    fn close_clears_menu_state() {
        let mut menu = WorkspaceMenu::default();
        menu.open_for("ws-1", 100.0, 200.0);
        menu.close();

        assert!(!menu.is_open());
        assert_eq!(menu.visible_target(), None);
        assert_eq!(menu.visible_position(), None);
        assert!(!menu.is_visible_for("ws-1"));
    }

    #[test]
    fn close_on_already_closed_menu_is_idempotent() {
        let mut menu = WorkspaceMenu::default();
        menu.close();
        menu.close();
        assert!(!menu.is_open());
        assert_eq!(menu.visible_target(), None);
    }

    /// `WorkspaceMenu::items()` exposes the same canonical list as
    /// `WorkspaceMenuAction::all()` so the renderer can reuse it without
    /// owning an instance.
    #[test]
    fn items_helper_matches_action_all() {
        assert_eq!(WorkspaceMenu::items(), WorkspaceMenuAction::all());
    }

    /// MenuPosition implements Copy + PartialEq for ergonomic assertions.
    #[test]
    fn menu_position_equality() {
        let a = MenuPosition { x: 1.0, y: 2.0 };
        let b = MenuPosition { x: 1.0, y: 2.0 };
        let c = MenuPosition { x: 1.5, y: 2.0 };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    /// Reopening the menu for the same workspace updates the position.
    #[test]
    fn reopening_same_workspace_updates_position() {
        let mut menu = WorkspaceMenu::default();
        menu.open_for("ws-1", 10.0, 20.0);
        menu.open_for("ws-1", 30.0, 40.0);

        assert!(menu.is_visible_for("ws-1"));
        assert_eq!(
            menu.visible_position(),
            Some(MenuPosition { x: 30.0, y: 40.0 })
        );
    }
}
