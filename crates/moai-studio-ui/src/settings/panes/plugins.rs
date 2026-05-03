//! PluginsPane — Claude Code Plugins 카탈로그 read-only viewer (skeleton).
//!
//! SPEC-V0-2-0-PLUGIN-MGR-001 MS-1 (audit Top 8 #3, v0.2.0 cycle Sprint 7):
//! Settings panel 의 Plugins section. v0.2.0 단계는 6 개 bundled plugin info
//! 를 read-only 로 노출하고 search filter 만 제공한다. install / uninstall /
//! enable 토글, marketplace HTTP fetch 는 후속 SPEC 으로 carry.
//!
//! Frozen zone (REQ-PM-001 ~ REQ-PM-007):
//! - moai-studio-terminal/** 무변경
//! - moai-studio-workspace/** 무변경
//! - SettingsSection 의 기존 10 variant 의 ordinal / discriminant / label 무변경
//!   (Plugins variant 추가 + filtered_plugins 만 새로 노출)
//! - SettingsViewState 의 기존 9 state 필드 무변경
//!
//! Lightweight SPEC fast-track applied — see
//! `.claude/rules/moai/workflow/spec-workflow.md` §Plan Phase variant.

use crate::settings::settings_state::PluginsState;

// ============================================================
// PluginInfo — single bundled plugin metadata
// ============================================================

/// Read-only metadata for a single bundled plugin (REQ-PM-004).
///
/// `name` is the canonical identifier (must be unique across the catalog),
/// `source` is either `"local-bundled"` for in-tree plugins or the
/// marketplace name (e.g. `"claude-code-marketplace"`). Future SPECs may
/// extend this struct with `enabled: bool` once the toggle wire exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    /// Canonical plugin identifier (kebab-case).
    pub name: &'static str,
    /// Origin of the plugin: `"local-bundled"` or marketplace name.
    pub source: &'static str,
    /// Semantic version string (e.g. `"0.1.2"`).
    pub version: &'static str,
    /// One-line human-readable description shown in the catalog row.
    pub description: &'static str,
}

// ============================================================
// PluginsPane
// ============================================================

/// PluginsPane — read-only Claude Code plugin catalog + search filter.
///
/// @MX:NOTE: [AUTO] plugins-pane-skeleton — REQ-PM-001 ~ REQ-PM-007.
/// v0.2.0 MS-1: 6 bundled plugin info 의 read-only list + case-insensitive
/// substring search. install / uninstall / enable / marketplace fetch 는
/// 별 SPEC carry.
pub struct PluginsPane {
    /// In-memory state (search filter buffer).
    pub state: PluginsState,
}

impl PluginsPane {
    /// Construct a `PluginsPane` with the default empty filter.
    pub fn new() -> Self {
        Self {
            state: PluginsState::default(),
        }
    }

    /// Construct a `PluginsPane` with a pre-populated state (test convenience).
    pub fn with_state(state: PluginsState) -> Self {
        Self { state }
    }

    // ---- section metadata ----

    /// Section title (REQ-PM-003).
    pub fn title() -> &'static str {
        "Plugins"
    }

    /// Section description shown above the catalog list.
    pub fn description() -> &'static str {
        "Claude Code 의 6 개 bundled plugin 을 read-only 로 표시합니다. 설치, 활성화 토글, marketplace fetch 는 향후 버전에서 제공됩니다."
    }

    // ---- known plugins ----

    /// Returns the canonical 6-entry bundled plugin list (REQ-PM-004).
    ///
    /// The ordering is the canonical seed order from
    /// `.moai/specs/SPEC-V0-2-0-PLUGIN-MGR-001/spec.md` §8. All names are
    /// unique, all sources / descriptions are non-empty.
    pub fn known_plugins() -> &'static [PluginInfo] {
        &[
            PluginInfo {
                name: "moai-adk",
                source: "local-bundled",
                version: "0.1.2",
                description: "MoAI Agentic Development Kit — full SPEC workflow + DDD/TDD agents.",
            },
            PluginInfo {
                name: "claude-code-skills",
                source: "claude-code-marketplace",
                version: "1.0.0",
                description: "Official Claude Code skills bundle (update-config, simplify, fewer-permission-prompts, ...).",
            },
            PluginInfo {
                name: "mermaid-diagrams",
                source: "local-bundled",
                version: "0.3.0",
                description: "Mermaid diagram rendering for markdown viewer.",
            },
            PluginInfo {
                name: "git-co-author",
                source: "claude-code-marketplace",
                version: "0.2.1",
                description: "Auto-suggest co-author trailer in git commit messages.",
            },
            PluginInfo {
                name: "nextra-docs",
                source: "local-bundled",
                version: "0.4.0",
                description: "Nextra-style documentation site generator.",
            },
            PluginInfo {
                name: "shadcn-ui-helper",
                source: "claude-code-marketplace",
                version: "0.5.0",
                description: "shadcn/ui component scaffolding helper.",
            },
        ]
    }

    /// Returns the filter-applied subset of `known_plugins()` (REQ-PM-005, REQ-PM-006).
    ///
    /// Empty filter (default) returns every plugin in canonical order.
    /// Non-empty filter performs a case-insensitive substring match against
    /// both `name` and `description`.
    pub fn visible_plugins(&self) -> Vec<&'static PluginInfo> {
        self.state
            .filtered_plugins(Self::known_plugins(), |p| (p.name, p.description))
    }

    /// Number of plugins matching the current filter.
    pub fn visible_count(&self) -> usize {
        self.visible_plugins().len()
    }

    /// Update the search filter buffer.
    pub fn set_plugin_filter(&mut self, filter: impl Into<String>) {
        self.state.plugin_filter = filter.into();
    }

    /// Returns the current search filter buffer.
    pub fn plugin_filter(&self) -> &str {
        &self.state.plugin_filter
    }

    /// Clear the filter (return to full-catalog visibility).
    pub fn clear_plugin_filter(&mut self) {
        self.state.plugin_filter.clear();
    }
}

impl Default for PluginsPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Unit tests — SPEC-V0-2-0-PLUGIN-MGR-001 MS-1 (AC-PM-3 ~ AC-PM-7 partial)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-PM-3 (REQ-PM-004): title is "Plugins".
    #[test]
    fn plugins_pane_title_is_plugins() {
        assert_eq!(PluginsPane::title(), "Plugins");
    }

    /// AC-PM-3 (REQ-PM-004): description is non-empty and mentions "6".
    #[test]
    fn plugins_pane_description_mentions_catalog_size() {
        let desc = PluginsPane::description();
        assert!(!desc.is_empty(), "description must not be empty");
        assert!(
            desc.contains('6'),
            "description should mention the catalog size: {desc}"
        );
    }

    /// AC-PM-3 (REQ-PM-004): known_plugins() returns exactly 6 entries.
    #[test]
    fn plugins_pane_known_plugins_has_6_entries() {
        assert_eq!(PluginsPane::known_plugins().len(), 6);
    }

    /// AC-PM-3 (REQ-PM-004): all names are unique.
    #[test]
    fn plugins_pane_known_plugins_have_unique_names() {
        let entries = PluginsPane::known_plugins();
        let unique: std::collections::HashSet<&'static str> =
            entries.iter().map(|p| p.name).collect();
        assert_eq!(
            unique.len(),
            entries.len(),
            "duplicate plugin name detected"
        );
    }

    /// AC-PM-3 (REQ-PM-004): all sources and descriptions are non-empty.
    #[test]
    fn plugins_pane_known_plugins_have_non_empty_metadata() {
        for p in PluginsPane::known_plugins() {
            assert!(!p.name.is_empty(), "plugin name must not be empty");
            assert!(
                !p.source.is_empty(),
                "plugin {}.source must not be empty",
                p.name
            );
            assert!(
                !p.description.is_empty(),
                "plugin {}.description must not be empty",
                p.name
            );
            assert!(
                !p.version.is_empty(),
                "plugin {}.version must not be empty",
                p.name
            );
        }
    }

    /// AC-PM-4 (REQ-PM-005): empty filter (default) shows all 6 entries.
    #[test]
    fn plugins_pane_empty_filter_shows_all_entries() {
        let pane = PluginsPane::new();
        assert_eq!(pane.plugin_filter(), "");
        assert_eq!(pane.visible_count(), 6);
    }

    /// AC-PM-5 (REQ-PM-006): case-insensitive name/description substring match.
    #[test]
    fn plugins_pane_filter_case_insensitive_substring_match() {
        let mut pane = PluginsPane::new();
        pane.set_plugin_filter("git");
        let visible = pane.visible_plugins();
        // git-co-author matches via name; commit-message description also contains "git".
        assert!(
            !visible.is_empty(),
            "filter 'git' must match at least one plugin"
        );
        for p in &visible {
            let n = p.name.to_ascii_lowercase();
            let d = p.description.to_ascii_lowercase();
            assert!(
                n.contains("git") || d.contains("git"),
                "plugin '{}' matched neither name nor description on 'git'",
                p.name
            );
        }
    }

    /// AC-PM-5 (REQ-PM-006): UPPERCASE filter still matches lowercase names.
    #[test]
    fn plugins_pane_filter_uppercase_matches_lowercase() {
        let mut pane = PluginsPane::new();
        pane.set_plugin_filter("MOAI");
        let visible = pane.visible_plugins();
        // moai-adk should match.
        assert!(
            visible.iter().any(|p| p.name == "moai-adk"),
            "uppercase 'MOAI' filter must match lowercase 'moai-adk'"
        );
    }

    /// AC-PM-6 (REQ-PM-006): no-match filter returns empty result.
    #[test]
    fn plugins_pane_filter_no_match_returns_empty() {
        let mut pane = PluginsPane::new();
        pane.set_plugin_filter("ZZZNonexistentPluginZZZ");
        assert_eq!(pane.visible_count(), 0);
    }

    /// clear_plugin_filter restores full visibility.
    #[test]
    fn plugins_pane_clear_filter_restores_full_view() {
        let mut pane = PluginsPane::new();
        pane.set_plugin_filter("git");
        assert!(pane.visible_count() > 0);
        pane.clear_plugin_filter();
        assert_eq!(pane.plugin_filter(), "");
        assert_eq!(pane.visible_count(), 6);
    }

    /// with_state preserves the injected filter.
    #[test]
    fn plugins_pane_with_state_preserves_filter() {
        let state = PluginsState {
            plugin_filter: "shadcn".to_string(),
        };
        let pane = PluginsPane::with_state(state);
        assert_eq!(pane.plugin_filter(), "shadcn");
        assert_eq!(pane.visible_count(), 1);
        assert_eq!(pane.visible_plugins()[0].name, "shadcn-ui-helper");
    }
}
