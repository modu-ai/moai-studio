//! Command Registry — real structured command catalog for CommandPalette.
//!
//! @MX:NOTE: [AUTO] CommandRegistry — 30+ commands in 9 categories replacing mock data.
//! @MX:SPEC: SPEC-V3-012 MS-4 AC-PL-16
//! @MX:ANCHOR: [AUTO] CommandRegistry::default — palette command source, fan_in >= 3 callers.
//! @MX:REASON: [AUTO] CommandPalette, dispatch matcher, tests all depend on this registry.

// ============================================================
// CommandEntry — structured registry entry
// ============================================================

/// A single command entry in the command registry.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandEntry {
    /// Namespaced command identifier, e.g. `pane.split_horizontal`.
    pub id: &'static str,
    /// Human-readable display label.
    pub label: &'static str,
    /// Category grouping for display/filtering.
    pub category: &'static str,
    /// Optional keyboard shortcut hint string (display only).
    pub keybinding: Option<&'static str>,
}

impl CommandEntry {
    /// Create a new CommandEntry.
    pub const fn new(
        id: &'static str,
        label: &'static str,
        category: &'static str,
        keybinding: Option<&'static str>,
    ) -> Self {
        Self {
            id,
            label,
            category,
            keybinding,
        }
    }
}

// ============================================================
// Category constants
// ============================================================

/// All available command categories in display order.
///
/// SPEC-V0-3-0-PALETTE-POLISH-001 (REQ-PP-002): 4 신규 카테고리 추가
/// (Plugin / Layout / Help / Spec). 기존 11 → 15 categories.
pub const CATEGORIES: &[&str] = &[
    "File",
    "View",
    "Pane",
    "Tab",
    "Workspace",
    "Surface",
    "Settings",
    "Theme",
    "Git",
    "Agent",
    "Terminal",
    "Plugin",
    "Layout",
    "Help",
    "Spec",
];

// ============================================================
// CommandRegistry
// ============================================================

/// Structured command registry — 30+ entries across 10 categories.
///
/// @MX:ANCHOR: [AUTO] CommandRegistry — sole source of truth for CommandPalette entries.
/// @MX:REASON: [AUTO] fan_in >= 3: CommandPalette::new(), dispatch_command(), registry tests.
pub struct CommandRegistry {
    /// All registered command entries.
    pub entries: Vec<CommandEntry>,
    /// Available category labels (subset of CATEGORIES that have entries).
    pub categories: Vec<&'static str>,
}

impl CommandRegistry {
    /// Build the default registry with all built-in commands.
    pub fn default_registry() -> Self {
        let entries = default_entries();
        let mut seen = std::collections::HashSet::new();
        let categories: Vec<&'static str> = entries
            .iter()
            .filter_map(|e| {
                if seen.contains(e.category) {
                    None
                } else {
                    seen.insert(e.category);
                    Some(e.category)
                }
            })
            .collect();
        Self {
            entries,
            categories,
        }
    }

    /// Return all entries in the registry.
    pub fn all(&self) -> &[CommandEntry] {
        &self.entries
    }

    /// Lookup a command by its id. Returns None if not found.
    pub fn get(&self, id: &str) -> Option<&CommandEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

// ============================================================
// Built-in commands
// ============================================================

/// Build the canonical list of all built-in commands (30+ entries).
fn default_entries() -> Vec<CommandEntry> {
    vec![
        // ── File ──
        CommandEntry::new("file.new", "New File", "File", Some("Cmd+N")),
        CommandEntry::new("file.open", "Open File...", "File", Some("Cmd+O")),
        CommandEntry::new("file.save", "Save File", "File", Some("Cmd+S")),
        CommandEntry::new(
            "file.save_all",
            "Save All Files",
            "File",
            Some("Cmd+Shift+S"),
        ),
        CommandEntry::new("file.close", "Close File", "File", Some("Cmd+W")),
        // ── View ──
        CommandEntry::new("view.toggle_sidebar", "Toggle Sidebar", "View", None),
        CommandEntry::new("view.toggle_banner", "Toggle Banner", "View", None),
        CommandEntry::new("view.find", "Find in File", "View", Some("Cmd+F")),
        CommandEntry::new("view.reload_workspace", "Reload Workspace", "View", None),
        // ── Pane ──
        CommandEntry::new(
            "pane.split_horizontal",
            "Split Pane Horizontal",
            "Pane",
            None,
        ),
        CommandEntry::new("pane.split_vertical", "Split Pane Vertical", "Pane", None),
        CommandEntry::new("pane.close", "Close Pane", "Pane", None),
        CommandEntry::new("pane.focus_next", "Focus Next Pane", "Pane", None),
        CommandEntry::new("pane.focus_prev", "Focus Previous Pane", "Pane", None),
        // ── Tab ──
        CommandEntry::new("tab.new", "New Tab", "Tab", Some("Cmd+T")),
        CommandEntry::new("tab.close", "Close Tab", "Tab", Some("Cmd+W")),
        CommandEntry::new("tab.next", "Next Tab", "Tab", Some("Ctrl+Tab")),
        CommandEntry::new("tab.prev", "Previous Tab", "Tab", Some("Ctrl+Shift+Tab")),
        CommandEntry::new("tab.move_right", "Move Tab Right", "Tab", None),
        CommandEntry::new("tab.move_left", "Move Tab Left", "Tab", None),
        // ── Workspace ──
        CommandEntry::new("workspace.switch", "Switch Workspace...", "Workspace", None),
        CommandEntry::new("workspace.new", "New Workspace", "Workspace", None),
        CommandEntry::new("workspace.rename", "Rename Workspace", "Workspace", None),
        CommandEntry::new("workspace.delete", "Delete Workspace", "Workspace", None),
        // SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3 (REQ-GS-051/052): updated label and keybinding.
        // id and category are frozen (R5 — must not change).
        CommandEntry::new(
            "workspace.search",
            "Search in all workspaces",
            "Workspace",
            Some("Cmd+Shift+F"),
        ),
        // ── Surface ──
        CommandEntry::new(
            "surface.toggle_terminal",
            "Toggle Terminal Surface",
            "Surface",
            None,
        ),
        CommandEntry::new("surface.toggle_code", "Open Code Viewer", "Surface", None),
        CommandEntry::new(
            "surface.toggle_markdown",
            "Open Markdown Viewer",
            "Surface",
            None,
        ),
        CommandEntry::new("surface.toggle_image", "Open Image Viewer", "Surface", None),
        CommandEntry::new("surface.toggle_web", "Open Web Viewer", "Surface", None),
        CommandEntry::new("surface.toggle_spec", "Open Spec Viewer", "Surface", None),
        // ── Settings ──
        CommandEntry::new("settings.open", "Open Settings", "Settings", Some("Cmd+,")),
        CommandEntry::new("settings.theme", "Change Theme...", "Settings", None),
        CommandEntry::new("settings.font", "Change Font Size...", "Settings", None),
        // ── Theme ──
        CommandEntry::new("theme.toggle", "Toggle Dark/Light Theme", "Theme", None),
        CommandEntry::new("theme.dark", "Switch to Dark Theme", "Theme", None),
        CommandEntry::new("theme.light", "Switch to Light Theme", "Theme", None),
        // ── Git ──
        CommandEntry::new("git.status", "Show Git Status", "Git", None),
        CommandEntry::new("git.branch", "Switch Git Branch...", "Git", None),
        CommandEntry::new("git.commit", "Commit Changes", "Git", None),
        // ── Agent ──
        CommandEntry::new(
            "agent.toggle_dashboard",
            "Toggle Agent Dashboard",
            "Agent",
            None,
        ),
        // ── Terminal ──
        // SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-006: shell.switch Command Palette entry.
        CommandEntry::new("shell.switch", "Switch Shell...", "Terminal", None),
        // ════════════════════════════════════════════════════════════════
        // SPEC-V0-3-0-PALETTE-POLISH-001 — 25 신규 entries (audit Top 16 #12)
        // ════════════════════════════════════════════════════════════════
        // ── File extras (REQ-PP-003) ──
        CommandEntry::new("file.recent_1", "Recent File 1", "File", None),
        CommandEntry::new("file.recent_2", "Recent File 2", "File", None),
        CommandEntry::new("file.recent_3", "Recent File 3", "File", None),
        CommandEntry::new("file.recent_4", "Recent File 4", "File", None),
        CommandEntry::new("file.recent_5", "Recent File 5", "File", None),
        CommandEntry::new("file.duplicate", "Duplicate File", "File", None),
        CommandEntry::new("file.rename", "Rename File", "File", Some("F2")),
        // ── Workspace extras ──
        CommandEntry::new(
            "workspace.recent",
            "Recent Workspaces...",
            "Workspace",
            None,
        ),
        CommandEntry::new(
            "workspace.add_existing",
            "Add Existing Workspace...",
            "Workspace",
            None,
        ),
        CommandEntry::new(
            "workspace.show_in_finder",
            "Show Workspace in Finder",
            "Workspace",
            None,
        ),
        // ── Plugin (REQ-PP-004) ──
        CommandEntry::new("plugin.list", "Show Installed Plugins", "Plugin", None),
        CommandEntry::new("plugin.refresh", "Refresh Plugin List", "Plugin", None),
        CommandEntry::new("plugin.install", "Install Plugin...", "Plugin", None),
        CommandEntry::new("plugin.disable", "Disable Plugin...", "Plugin", None),
        CommandEntry::new("plugin.enable", "Enable Plugin...", "Plugin", None),
        // ── Layout (REQ-PP-005) ──
        CommandEntry::new("layout.center", "Center Layout", "Layout", None),
        CommandEntry::new("layout.zoom_in", "Zoom In", "Layout", Some("Cmd+=")),
        CommandEntry::new("layout.zoom_out", "Zoom Out", "Layout", Some("Cmd+-")),
        CommandEntry::new("layout.reset_zoom", "Reset Zoom", "Layout", Some("Cmd+0")),
        // ── Help (REQ-PP-006) ──
        CommandEntry::new("help.open_docs", "Open Documentation", "Help", None),
        CommandEntry::new("help.report_issue", "Report Issue", "Help", None),
        CommandEntry::new(
            "help.shortcuts",
            "Show Keyboard Shortcuts",
            "Help",
            Some("Cmd+/"),
        ),
        // ── Spec (REQ-PP-007) ──
        CommandEntry::new(
            "spec.open_panel",
            "Open SPEC Panel",
            "Spec",
            Some("Cmd+Shift+P"),
        ),
        CommandEntry::new("spec.new_spec", "Create New SPEC...", "Spec", None),
        CommandEntry::new("spec.refresh", "Refresh SPEC List", "Spec", None),
    ]
}

// ============================================================
// Unit tests — AC-PL-16
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// AC-PL-16 / AC-PP-1 (SPEC-V0-3-0-PALETTE-POLISH-001): registry has >= 60 entries.
    #[test]
    fn default_registry_has_at_least_60_entries() {
        let reg = CommandRegistry::default_registry();
        assert!(
            reg.entries.len() >= 60,
            "registry must have >= 60 entries (audit Top 16 #12), got {}",
            reg.entries.len()
        );
    }

    /// AC-PL-16: no duplicate ids in registry.
    #[test]
    fn no_duplicate_ids() {
        let reg = CommandRegistry::default_registry();
        let mut seen = HashSet::new();
        for entry in &reg.entries {
            assert!(seen.insert(entry.id), "duplicate command id: {}", entry.id);
        }
    }

    /// AC-PL-16: categories list is non-empty.
    #[test]
    fn categories_non_empty() {
        let reg = CommandRegistry::default_registry();
        assert!(!reg.categories.is_empty(), "categories must not be empty");
    }

    /// All expected categories are present in registry.
    /// AC-PP-2 (SPEC-V0-3-0-PALETTE-POLISH-001): 4 신규 카테고리 (Plugin/Layout/Help/Spec) 모두 포함.
    #[test]
    fn all_expected_categories_present() {
        let reg = CommandRegistry::default_registry();
        let expected = [
            "File",
            "View",
            "Pane",
            "Tab",
            "Workspace",
            "Surface",
            "Settings",
            "Theme",
            "Git",
            "Agent",
            "Terminal",
            "Plugin", // SPEC-V0-3-0-PALETTE-POLISH-001 신규
            "Layout", // SPEC-V0-3-0-PALETTE-POLISH-001 신규
            "Help",   // SPEC-V0-3-0-PALETTE-POLISH-001 신규
            "Spec",   // SPEC-V0-3-0-PALETTE-POLISH-001 신규
        ];
        for cat in &expected {
            assert!(
                reg.categories.contains(cat),
                "category '{}' missing from registry",
                cat
            );
        }
    }

    /// All entries have non-empty id, label, and category.
    #[test]
    fn all_entries_have_non_empty_fields() {
        let reg = CommandRegistry::default_registry();
        for entry in &reg.entries {
            assert!(!entry.id.is_empty(), "entry id must not be empty");
            assert!(!entry.label.is_empty(), "entry label must not be empty");
            assert!(
                !entry.category.is_empty(),
                "entry category must not be empty"
            );
        }
    }

    /// All entry ids are namespaced (contain a dot).
    #[test]
    fn all_ids_are_namespaced() {
        let reg = CommandRegistry::default_registry();
        for entry in &reg.entries {
            assert!(
                entry.id.contains('.'),
                "id '{}' must be namespaced (e.g. 'pane.split_horizontal')",
                entry.id
            );
        }
    }

    /// get() returns correct entry for known id.
    #[test]
    fn get_returns_entry_by_id() {
        let reg = CommandRegistry::default_registry();
        let entry = reg.get("pane.split_horizontal");
        assert!(entry.is_some(), "pane.split_horizontal must exist");
        assert_eq!(entry.unwrap().label, "Split Pane Horizontal");
        assert_eq!(entry.unwrap().category, "Pane");
    }

    /// get() returns None for unknown id.
    #[test]
    fn get_returns_none_for_unknown_id() {
        let reg = CommandRegistry::default_registry();
        assert!(reg.get("nonexistent.command").is_none());
    }

    // ── T6: workspace.search entry — AC-GS-11 (label + keybinding) ──

    /// AC-GS-11 (REQ-GS-051): workspace.search entry label must be
    /// "Search in all workspaces".
    #[test]
    fn test_workspace_search_entry_label_updated() {
        let reg = CommandRegistry::default_registry();
        let entry = reg
            .get("workspace.search")
            .expect("workspace.search must exist in registry");
        assert_eq!(
            entry.label, "Search in all workspaces",
            "workspace.search label must be 'Search in all workspaces' (REQ-GS-051)"
        );
    }

    /// AC-GS-11 (REQ-GS-052): workspace.search keybinding must be
    /// Some("Cmd+Shift+F").
    #[test]
    fn test_workspace_search_entry_keybinding_set_to_cmd_shift_f() {
        let reg = CommandRegistry::default_registry();
        let entry = reg
            .get("workspace.search")
            .expect("workspace.search must exist in registry");
        assert_eq!(
            entry.keybinding,
            Some("Cmd+Shift+F"),
            "workspace.search keybinding must be Some(\"Cmd+Shift+F\") (REQ-GS-052)"
        );
    }

    /// AC-GS-11: workspace.search id and category must not change (R5 invariant).
    #[test]
    fn test_workspace_search_id_and_category_unchanged() {
        let reg = CommandRegistry::default_registry();
        let entry = reg
            .get("workspace.search")
            .expect("workspace.search must exist");
        assert_eq!(
            entry.id, "workspace.search",
            "id must remain unchanged (R5)"
        );
        assert_eq!(
            entry.category, "Workspace",
            "category must remain 'Workspace' (R5)"
        );
    }

    // ── T6: shell.switch entry — AC-MS-6 ──

    /// AC-MS-6 (REQ-MS-006): shell.switch entry exists with correct label and category.
    #[test]
    fn test_palette_registry_has_shell_switch_entry() {
        let reg = CommandRegistry::default_registry();
        let entry = reg
            .get("shell.switch")
            .expect("shell.switch must exist in registry (SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-006)");
        assert_eq!(
            entry.label, "Switch Shell...",
            "shell.switch label must be 'Switch Shell...'"
        );
        assert_eq!(
            entry.category, "Terminal",
            "shell.switch category must be 'Terminal'"
        );
        assert!(
            entry.keybinding.is_none(),
            "shell.switch keybinding must be None in v1"
        );
    }

    /// AC-MS-6: workspace.search entry is unchanged (R4 invariant).
    #[test]
    fn test_palette_registry_workspace_search_unchanged() {
        let reg = CommandRegistry::default_registry();
        let entry = reg
            .get("workspace.search")
            .expect("workspace.search must still exist after adding shell.switch");
        assert_eq!(entry.label, "Search in all workspaces");
        assert_eq!(entry.category, "Workspace");
        assert_eq!(entry.keybinding, Some("Cmd+Shift+F"));
    }

    // ════════════════════════════════════════════════════════════════
    // SPEC-V0-3-0-PALETTE-POLISH-001 — T-PP: Registry expansion checks
    // ════════════════════════════════════════════════════════════════

    /// AC-PP-3: file.recent_1 ~ file.recent_5 exist with category "File".
    #[test]
    fn registry_exposes_five_file_recent_slots() {
        let reg = CommandRegistry::default_registry();
        for n in 1..=5 {
            let id = format!("file.recent_{n}");
            let entry = reg
                .get(&id)
                .unwrap_or_else(|| panic!("{id} must exist in registry"));
            assert_eq!(entry.category, "File", "{id} category must be 'File'");
        }
    }

    /// AC-PP-4: 5 Plugin entries (list/refresh/install/disable/enable).
    #[test]
    fn registry_exposes_five_plugin_entries() {
        let reg = CommandRegistry::default_registry();
        let ids = [
            "plugin.list",
            "plugin.refresh",
            "plugin.install",
            "plugin.disable",
            "plugin.enable",
        ];
        for id in &ids {
            let entry = reg.get(id).unwrap_or_else(|| panic!("{id} must exist"));
            assert_eq!(entry.category, "Plugin", "{id} category must be 'Plugin'");
        }
    }

    /// AC-PP-5: 4 Layout entries (center/zoom_in/zoom_out/reset_zoom).
    #[test]
    fn registry_exposes_four_layout_entries() {
        let reg = CommandRegistry::default_registry();
        let ids = [
            "layout.center",
            "layout.zoom_in",
            "layout.zoom_out",
            "layout.reset_zoom",
        ];
        for id in &ids {
            let entry = reg.get(id).unwrap_or_else(|| panic!("{id} must exist"));
            assert_eq!(entry.category, "Layout", "{id} category must be 'Layout'");
        }
    }

    /// AC-PP-6: Help (3) + Spec (3) entries.
    #[test]
    fn registry_exposes_help_and_spec_entries() {
        let reg = CommandRegistry::default_registry();
        let help_ids = ["help.open_docs", "help.report_issue", "help.shortcuts"];
        let spec_ids = ["spec.open_panel", "spec.new_spec", "spec.refresh"];
        for id in &help_ids {
            let entry = reg.get(id).unwrap_or_else(|| panic!("{id} must exist"));
            assert_eq!(entry.category, "Help");
        }
        for id in &spec_ids {
            let entry = reg.get(id).unwrap_or_else(|| panic!("{id} must exist"));
            assert_eq!(entry.category, "Spec");
        }
    }

    /// AC-PP-2 / REQ-PP-002: CATEGORIES const reflects 4 신규 카테고리.
    #[test]
    fn categories_const_includes_four_new_categories() {
        for cat in ["Plugin", "Layout", "Help", "Spec"] {
            assert!(
                CATEGORIES.contains(&cat),
                "CATEGORIES const must include '{cat}'"
            );
        }
        assert_eq!(CATEGORIES.len(), 15, "11 base + 4 new = 15 categories");
    }

    /// REQ-PP-008: 기존 44 entries 의 호환성 — workspace.search / pane.split_horizontal
    /// 등 기존 ids 가 그대로 존재한다.
    #[test]
    fn legacy_entries_remain_unchanged() {
        let reg = CommandRegistry::default_registry();
        // 4 안전 샘플 — id / category / label 모두 보존 검증.
        let ws = reg.get("workspace.search").expect("workspace.search");
        assert_eq!(ws.label, "Search in all workspaces");
        assert_eq!(ws.category, "Workspace");
        assert_eq!(ws.keybinding, Some("Cmd+Shift+F"));

        let split = reg
            .get("pane.split_horizontal")
            .expect("pane.split_horizontal");
        assert_eq!(split.category, "Pane");

        let theme = reg.get("theme.toggle").expect("theme.toggle");
        assert_eq!(theme.category, "Theme");

        let shell = reg.get("shell.switch").expect("shell.switch");
        assert_eq!(shell.category, "Terminal");
    }

    /// Specific required commands exist.
    #[test]
    fn required_commands_exist() {
        let reg = CommandRegistry::default_registry();
        let required_ids = [
            "pane.split_horizontal",
            "pane.split_vertical",
            "pane.close",
            "tab.new",
            "tab.close",
            "tab.next",
            "tab.prev",
            "workspace.switch",
            "workspace.new",
            "surface.toggle_terminal",
            "surface.toggle_web",
            "settings.open",
            "theme.toggle",
            "git.status",
            "git.branch",
            "git.commit",
            "agent.toggle_dashboard",
        ];
        for id in &required_ids {
            assert!(
                reg.get(id).is_some(),
                "required command '{}' must exist in registry",
                id
            );
        }
    }
}
