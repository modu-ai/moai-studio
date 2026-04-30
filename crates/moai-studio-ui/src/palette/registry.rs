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
        CommandEntry::new("workspace.search", "Search in Workspace", "Workspace", None),
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
    ]
}

// ============================================================
// Unit tests — AC-PL-16
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// AC-PL-16: default registry has >= 30 entries.
    #[test]
    fn default_registry_has_at_least_30_entries() {
        let reg = CommandRegistry::default_registry();
        assert!(
            reg.entries.len() >= 30,
            "registry must have >= 30 entries, got {}",
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
