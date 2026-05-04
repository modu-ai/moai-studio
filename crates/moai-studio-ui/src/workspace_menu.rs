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
    /// SPEC-V0-3-0-WORKSPACE-COLOR-001 (REQ-WC-005): open a color picker modal.
    ChangeColor,
}

impl WorkspaceMenuAction {
    /// Canonical ordering of the actions as they appear in the menu.
    pub fn all() -> [WorkspaceMenuAction; 5] {
        [
            WorkspaceMenuAction::Rename,
            WorkspaceMenuAction::ChangeColor,
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
            WorkspaceMenuAction::ChangeColor => "Change Color",
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
    pub fn items() -> [WorkspaceMenuAction; 5] {
        WorkspaceMenuAction::all()
    }
}

// ============================================================
// ColorPickerModal — SPEC-V0-3-0-WORKSPACE-COLOR-001 REQ-WC-007
// ============================================================

/// SPEC-V0-3-0-WORKSPACE-COLOR-001 (REQ-WC-007): Color picker modal logic.
///
/// Logic-only state machine for the workspace color picker. GPUI render side
/// (swatch grid overlay) is deferred to a follow-up SPEC. Callers drive the
/// modal lifecycle: `open` → `select` → read `selected_color()` → caller
/// invokes `WorkspacesStore::set_color` → caller dismisses the modal.
#[derive(Debug, Clone)]
pub struct ColorPickerModal {
    target_id: String,
    selected: u32,
}

impl ColorPickerModal {
    /// Open the modal for the given workspace, pre-seeding the selection with
    /// the workspace's current color.
    pub fn open(target_id: impl Into<String>, current_color: u32) -> Self {
        Self {
            target_id: target_id.into(),
            selected: current_color,
        }
    }

    /// Update the in-progress selection. Caller is responsible for ensuring
    /// the value comes from the canonical palette.
    pub fn select(&mut self, color: u32) {
        self.selected = color;
    }

    /// Currently-selected color.
    pub fn selected_color(&self) -> u32 {
        self.selected
    }

    /// Workspace id this modal is targeting.
    pub fn target(&self) -> &str {
        &self.target_id
    }
}

// ============================================================
// RenameModal — REQ-D2-MS5-3
// ============================================================

// @MX:NOTE: [AUTO] REQ-D2-MS5-3 — RenameModal encapsulates the rename UI state
//   (target id + text buffer). Logic-level only; render is deferred to a follow-up PR.
/// Rename modal state: target workspace id and current text buffer.
///
/// Default state is closed (target is `None`, buffer is empty).
/// `commit()` returns `None` when the buffer trims to blank, leaving the
/// modal open so the caller can show an error hint.
#[derive(Debug, Default, Clone)]
pub struct RenameModal {
    target_id: Option<String>,
    buffer: String,
}

impl RenameModal {
    /// Open the modal for the given workspace, pre-filling the buffer with the
    /// current workspace name.
    pub fn open(&mut self, ws_id: impl Into<String>, current_name: impl Into<String>) {
        self.target_id = Some(ws_id.into());
        self.buffer = current_name.into();
    }

    /// Update the text buffer (called on every keystroke in the text field).
    pub fn set_buffer(&mut self, s: impl Into<String>) {
        self.buffer = s.into();
    }

    /// Attempt to commit the rename.
    ///
    /// Returns `Some((ws_id, new_name))` when the buffer is non-empty after
    /// trimming, and closes the modal. Returns `None` (modal stays open) when
    /// the buffer is blank.
    pub fn commit(&mut self) -> Option<(String, String)> {
        let trimmed = self.buffer.trim().to_string();
        if trimmed.is_empty() {
            return None;
        }
        let id = self.target_id.take()?;
        self.buffer.clear();
        Some((id, trimmed))
    }

    /// Cancel and reset to default closed state.
    pub fn cancel(&mut self) {
        self.target_id = None;
        self.buffer.clear();
    }

    /// True when the modal is currently open (has a target workspace).
    pub fn is_open(&self) -> bool {
        self.target_id.is_some()
    }

    /// The workspace id the modal is open for, if any.
    pub fn target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    /// Current text buffer contents.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }
}

// ============================================================
// DeleteConfirmation — REQ-D2-MS5-4
// ============================================================

// @MX:NOTE: [AUTO] REQ-D2-MS5-4 — DeleteConfirmation holds the pending delete target.
//   Logic-level only; the confirmation dialog rendering is a follow-up PR.
/// Delete confirmation state: tracks which workspace is pending deletion.
///
/// Default state is closed (no target).
#[derive(Debug, Default, Clone)]
pub struct DeleteConfirmation {
    target_id: Option<String>,
}

impl DeleteConfirmation {
    /// Open the confirmation for `ws_id`.
    pub fn open(&mut self, ws_id: impl Into<String>) {
        self.target_id = Some(ws_id.into());
    }

    /// Confirm the deletion.
    ///
    /// Returns `Some(ws_id)` and closes the dialog, or `None` if already closed.
    pub fn confirm(&mut self) -> Option<String> {
        self.target_id.take()
    }

    /// Cancel and close without deleting.
    pub fn cancel(&mut self) {
        self.target_id = None;
    }

    /// True when a workspace is pending confirmation.
    pub fn is_open(&self) -> bool {
        self.target_id.is_some()
    }

    /// The workspace id pending deletion, if any.
    pub fn target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }
}

// ============================================================
// WorkspaceMenuOutcome + dispatch_workspace_menu_action — REQ-D2-MS5-5
// ============================================================

// @MX:NOTE: [AUTO] REQ-D2-MS5-5 — WorkspaceMenuOutcome is the result type of the
//   dispatch adapter. Allows callers to react without coupling to store internals.
/// Outcome of dispatching a workspace context-menu action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceMenuOutcome {
    /// The Rename action was triggered; caller should open a `RenameModal`
    /// pre-filled with `current_name`.
    OpenRenameModal { ws_id: String, current_name: String },
    /// The Delete action was triggered; caller should open a `DeleteConfirmation`.
    OpenDeleteConfirmation { ws_id: String },
    /// SPEC-V0-3-0-WORKSPACE-COLOR-001 (REQ-WC-005): caller should open a
    /// `ColorPickerModal` pre-seeded with `current_color`.
    OpenColorPicker { ws_id: String, current_color: u32 },
    /// A reorder (MoveUp / MoveDown) completed successfully.
    Reordered,
    /// The action could not be applied (e.g. workspace not found in store).
    Unknown,
}

// @MX:ANCHOR: [AUTO] dispatch-workspace-menu-action
// @MX:REASON: [AUTO] REQ-D2-MS5-5. dispatch_workspace_menu_action is the single
//   adapter between WorkspaceMenuAction enum and WorkspacesStore mutations.
//   fan_in >= 3: RootView::handle_workspace_menu_action (T7), T6 tests,
//   future sidebar right-click wire (next MS).
/// Translate a `WorkspaceMenuAction` into a `WorkspaceMenuOutcome`, applying
/// any required `WorkspacesStore` mutations along the way.
///
/// - `Rename` — looks up the workspace name and returns `OpenRenameModal`.
/// - `Delete` — returns `OpenDeleteConfirmation` (actual removal is deferred
///   until the user confirms).
/// - `MoveUp` / `MoveDown` — mutates the store and returns `Reordered`.
/// - Any store error (e.g. workspace not found) → `Unknown`.
pub fn dispatch_workspace_menu_action(
    action: WorkspaceMenuAction,
    ws_id: &str,
    store: &mut moai_studio_workspace::WorkspacesStore,
) -> WorkspaceMenuOutcome {
    match action {
        WorkspaceMenuAction::Rename => {
            let current_name = match store.list().iter().find(|w| w.id == ws_id) {
                Some(ws) => ws.name.clone(),
                None => return WorkspaceMenuOutcome::Unknown,
            };
            WorkspaceMenuOutcome::OpenRenameModal {
                ws_id: ws_id.to_string(),
                current_name,
            }
        }
        WorkspaceMenuAction::Delete => WorkspaceMenuOutcome::OpenDeleteConfirmation {
            ws_id: ws_id.to_string(),
        },
        WorkspaceMenuAction::MoveUp => match store.move_up(ws_id) {
            Ok(()) => WorkspaceMenuOutcome::Reordered,
            Err(_) => WorkspaceMenuOutcome::Unknown,
        },
        WorkspaceMenuAction::MoveDown => match store.move_down(ws_id) {
            Ok(()) => WorkspaceMenuOutcome::Reordered,
            Err(_) => WorkspaceMenuOutcome::Unknown,
        },
        WorkspaceMenuAction::ChangeColor => match store.list().iter().find(|w| w.id == ws_id) {
            Some(ws) => WorkspaceMenuOutcome::OpenColorPicker {
                ws_id: ws_id.to_string(),
                current_color: ws.color,
            },
            None => WorkspaceMenuOutcome::Unknown,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T4: RenameModal ──────────────────────────────────────────────────────

    #[test]
    fn test_rename_modal_default_closed() {
        let modal = RenameModal::default();
        assert!(!modal.is_open());
        assert_eq!(modal.target_id(), None);
        assert_eq!(modal.buffer(), "");
    }

    #[test]
    fn test_rename_modal_open_sets_target_and_buffer() {
        let mut modal = RenameModal::default();
        modal.open("ws-1", "OldName");
        assert!(modal.is_open());
        assert_eq!(modal.target_id(), Some("ws-1"));
        assert_eq!(modal.buffer(), "OldName");
    }

    #[test]
    fn test_rename_modal_commit_returns_id_name() {
        let mut modal = RenameModal::default();
        modal.open("ws-1", "OldName");
        modal.set_buffer("NewName");
        let result = modal.commit();
        assert_eq!(result, Some(("ws-1".to_string(), "NewName".to_string())));
        assert!(!modal.is_open());
    }

    #[test]
    fn test_rename_modal_commit_empty_buffer_returns_none() {
        let mut modal = RenameModal::default();
        modal.open("ws-1", "OldName");
        modal.set_buffer("   ");
        let result = modal.commit();
        assert_eq!(result, None);
        // Modal remains open when commit fails (empty buffer)
        assert!(modal.is_open());
    }

    #[test]
    fn test_rename_modal_cancel_clears() {
        let mut modal = RenameModal::default();
        modal.open("ws-1", "OldName");
        modal.cancel();
        assert!(!modal.is_open());
        assert_eq!(modal.target_id(), None);
        assert_eq!(modal.buffer(), "");
    }

    // ── T5: DeleteConfirmation ───────────────────────────────────────────────

    #[test]
    fn test_delete_confirmation_default_closed() {
        let conf = DeleteConfirmation::default();
        assert!(!conf.is_open());
        assert_eq!(conf.target_id(), None);
    }

    #[test]
    fn test_delete_confirmation_open_sets_target() {
        let mut conf = DeleteConfirmation::default();
        conf.open("ws-2");
        assert!(conf.is_open());
        assert_eq!(conf.target_id(), Some("ws-2"));
    }

    #[test]
    fn test_delete_confirmation_confirm_returns_id() {
        let mut conf = DeleteConfirmation::default();
        conf.open("ws-2");
        let result = conf.confirm();
        assert_eq!(result, Some("ws-2".to_string()));
        assert!(!conf.is_open());
    }

    #[test]
    fn test_delete_confirmation_cancel_clears() {
        let mut conf = DeleteConfirmation::default();
        conf.open("ws-2");
        conf.cancel();
        assert!(!conf.is_open());
        assert_eq!(conf.target_id(), None);
    }

    // ── T6: WorkspaceMenuOutcome + dispatch_workspace_menu_action ────────────

    #[test]
    fn test_dispatch_rename_returns_open_rename_modal() {
        let tmp_file = std::env::temp_dir().join("moai-dispatch-rename.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = std::env::temp_dir().join("moai-dispatch-rename-project");
        std::fs::create_dir_all(&project).unwrap();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp_file).unwrap();
        let ws = moai_studio_workspace::Workspace::from_path(&project).unwrap();
        let id = ws.id.clone();
        let name = ws.name.clone();
        store.add(ws).unwrap();

        let outcome = dispatch_workspace_menu_action(WorkspaceMenuAction::Rename, &id, &mut store);
        match outcome {
            WorkspaceMenuOutcome::OpenRenameModal {
                ws_id,
                current_name,
            } => {
                assert_eq!(ws_id, id);
                assert_eq!(current_name, name);
            }
            other => panic!("expected OpenRenameModal, got {other:?}"),
        }

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
    }

    #[test]
    fn test_dispatch_delete_returns_open_delete_confirmation() {
        let tmp_file = std::env::temp_dir().join("moai-dispatch-delete.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = std::env::temp_dir().join("moai-dispatch-delete-project");
        std::fs::create_dir_all(&project).unwrap();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp_file).unwrap();
        let ws = moai_studio_workspace::Workspace::from_path(&project).unwrap();
        let id = ws.id.clone();
        store.add(ws).unwrap();

        let outcome = dispatch_workspace_menu_action(WorkspaceMenuAction::Delete, &id, &mut store);
        match outcome {
            WorkspaceMenuOutcome::OpenDeleteConfirmation { ws_id } => {
                assert_eq!(ws_id, id);
            }
            other => panic!("expected OpenDeleteConfirmation, got {other:?}"),
        }

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
    }

    #[test]
    fn test_dispatch_move_up_calls_store_and_returns_reordered() {
        let tmp_file = std::env::temp_dir().join("moai-dispatch-moveup.json");
        std::fs::remove_file(&tmp_file).ok();
        let p1 = std::env::temp_dir().join("moai-dispatch-moveup-a");
        let p2 = std::env::temp_dir().join("moai-dispatch-moveup-b");
        std::fs::create_dir_all(&p1).unwrap();
        std::fs::create_dir_all(&p2).unwrap();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp_file).unwrap();
        let ws1 = moai_studio_workspace::Workspace::from_path(&p1).unwrap();
        let ws2 = moai_studio_workspace::Workspace::from_path(&p2).unwrap();
        let id2 = ws2.id.clone();
        store.add(ws1).unwrap();
        store.add(ws2).unwrap();

        let outcome = dispatch_workspace_menu_action(WorkspaceMenuAction::MoveUp, &id2, &mut store);
        assert!(matches!(outcome, WorkspaceMenuOutcome::Reordered));
        assert_eq!(store.list()[0].id, id2);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&p1).ok();
        std::fs::remove_dir_all(&p2).ok();
    }

    #[test]
    fn test_dispatch_move_down_calls_store_and_returns_reordered() {
        let tmp_file = std::env::temp_dir().join("moai-dispatch-movedown.json");
        std::fs::remove_file(&tmp_file).ok();
        let p1 = std::env::temp_dir().join("moai-dispatch-movedown-a");
        let p2 = std::env::temp_dir().join("moai-dispatch-movedown-b");
        std::fs::create_dir_all(&p1).unwrap();
        std::fs::create_dir_all(&p2).unwrap();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp_file).unwrap();
        let ws1 = moai_studio_workspace::Workspace::from_path(&p1).unwrap();
        let ws2 = moai_studio_workspace::Workspace::from_path(&p2).unwrap();
        let id1 = ws1.id.clone();
        store.add(ws1).unwrap();
        store.add(ws2).unwrap();

        let outcome =
            dispatch_workspace_menu_action(WorkspaceMenuAction::MoveDown, &id1, &mut store);
        assert!(matches!(outcome, WorkspaceMenuOutcome::Reordered));
        assert_eq!(store.list()[1].id, id1);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&p1).ok();
        std::fs::remove_dir_all(&p2).ok();
    }

    #[test]
    fn test_dispatch_unknown_workspace_returns_unknown() {
        let tmp_file = std::env::temp_dir().join("moai-dispatch-unknown.json");
        std::fs::remove_file(&tmp_file).ok();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp_file).unwrap();
        let outcome =
            dispatch_workspace_menu_action(WorkspaceMenuAction::MoveUp, "no-such-id", &mut store);
        assert!(matches!(outcome, WorkspaceMenuOutcome::Unknown));

        std::fs::remove_file(&tmp_file).ok();
    }

    /// AC-D2-1 / AC-WC-5: all five actions are exposed and have non-empty labels.
    #[test]
    fn workspace_menu_action_all_returns_five_distinct_variants() {
        let actions = WorkspaceMenuAction::all();
        assert_eq!(actions.len(), 5);
        let labels: Vec<&'static str> = actions.iter().map(|a| a.label()).collect();
        assert_eq!(
            labels,
            vec!["Rename", "Change Color", "Delete", "Move Up", "Move Down"],
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
        // AC-WC-6: ChangeColor is non-destructive.
        assert!(!WorkspaceMenuAction::ChangeColor.is_destructive());
    }

    // ── T-WC: ColorPickerModal — SPEC-V0-3-0-WORKSPACE-COLOR-001 REQ-WC-007 ──

    /// AC-WC-7: open seeds target + selected_color; select updates the choice.
    #[test]
    fn color_picker_modal_open_seeds_state_and_select_updates() {
        let mut modal = ColorPickerModal::open("ws-a", 0xFF0000);
        assert_eq!(modal.target(), "ws-a");
        assert_eq!(modal.selected_color(), 0xFF0000);
        modal.select(0x00FF00);
        assert_eq!(modal.selected_color(), 0x00FF00);
        assert_eq!(modal.target(), "ws-a", "target is stable across selections");
    }

    /// AC-WC-7 보강: dispatch ChangeColor 가 OpenColorPicker 를 반환한다.
    #[test]
    fn dispatch_change_color_returns_open_color_picker() {
        let tmp_file = std::env::temp_dir().join("moai-dispatch-color.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = std::env::temp_dir().join("moai-dispatch-color-project");
        std::fs::create_dir_all(&project).unwrap();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp_file).unwrap();
        let ws = moai_studio_workspace::Workspace::from_path(&project).unwrap();
        let id = ws.id.clone();
        let color = ws.color;
        store.add(ws).unwrap();

        let outcome =
            dispatch_workspace_menu_action(WorkspaceMenuAction::ChangeColor, &id, &mut store);
        match outcome {
            WorkspaceMenuOutcome::OpenColorPicker {
                ws_id,
                current_color,
            } => {
                assert_eq!(ws_id, id);
                assert_eq!(current_color, color);
            }
            other => panic!("expected OpenColorPicker, got {other:?}"),
        }

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
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
