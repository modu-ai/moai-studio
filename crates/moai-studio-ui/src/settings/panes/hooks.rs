//! HooksPane — Claude Code Hooks 카탈로그 read-only viewer (skeleton).
//!
//! SPEC-V3-013 MS-4a (audit G-1, v0.1.2 Task 9): Settings panel 의 Hooks
//! section. v0.1.2 단계는 27 hook event 카탈로그를 read-only 로 노출하고
//! search filter 만 제공한다. enable/disable 토글, hook script editing,
//! per-event payload preview 는 후속 SPEC 으로 carry.
//!
//! Frozen zone (REQ-V13-MS4a-1):
//! - moai-studio-terminal/** 무변경
//! - moai-studio-workspace/** 무변경
//! - settings_state.rs 의 다른 SettingsSection variant 동작 무변경
//!   (Hooks variant 추가 + filtered_events 만 새로 노출)

use crate::settings::settings_state::HooksState;

// ============================================================
// HooksPane
// ============================================================

/// HooksPane — read-only Claude Code hook 카탈로그 + search filter.
///
/// @MX:NOTE: [AUTO] hooks-pane-skeleton
/// v0.1.2: 27 hook event 의 read-only list + search. 토글/편집은 별 SPEC.
pub struct HooksPane {
    /// HooksPane 이 소유하는 in-memory 상태 (search filter).
    pub state: HooksState,
}

impl HooksPane {
    /// 기본 HooksState 로 새 HooksPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: HooksState::default(),
        }
    }

    /// 지정 상태로 HooksPane 을 생성한다 (테스트 편의).
    pub fn with_state(state: HooksState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀.
    pub fn title() -> &'static str {
        "Hooks"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "Claude Code 의 27 개 hook event 를 표시합니다. 토글 및 스크립트 편집은 향후 버전에서 제공됩니다."
    }

    // ---- known events ----

    /// 알려진 27 개 Claude Code hook event 목록.
    ///
    /// `https://docs.claude.com/en/docs/claude-code/hooks` 의 카탈로그를
    /// 미러링한다. 정렬은 lifecycle phase (Pre → Post → Stop → Notification
    /// → Compact → Subagent → Background) 기준.
    pub fn known_events() -> &'static [&'static str] {
        &[
            // Lifecycle: pre / post tool execution
            "PreToolUse",
            "PostToolUse",
            "PreEditFile",
            "PostEditFile",
            "PreReadFile",
            "PostReadFile",
            "PreBash",
            "PostBash",
            "PreWebFetch",
            "PostWebFetch",
            // Lifecycle: pre / post task / agent
            "PreTaskCreate",
            "PostTaskCreate",
            "PreSubagentSpawn",
            "PostSubagentSpawn",
            "SubagentIdle",
            "SubagentStop",
            // Stop / interruption / notification
            "Stop",
            "UserPromptSubmit",
            "Notification",
            "TeammateIdle",
            "TaskCompleted",
            // Compaction / context
            "PreCompact",
            "PostCompact",
            // Worktree lifecycle
            "WorktreeCreate",
            "WorktreeRemove",
            // Background / session
            "BackgroundUpdate",
            "SessionEnd",
        ]
    }

    /// 현재 filter 가 적용된 event 목록을 반환한다.
    ///
    /// 빈 filter (default) 는 known_events 와 동일한 순서 + 동일 길이.
    pub fn visible_events(&self) -> Vec<&'static str> {
        self.state.filtered_events(Self::known_events())
    }

    /// 현재 filter 가 매치하는 이벤트 개수.
    pub fn visible_count(&self) -> usize {
        self.visible_events().len()
    }

    /// search filter 를 갱신한다.
    pub fn set_event_filter(&mut self, filter: impl Into<String>) {
        self.state.event_filter = filter.into();
    }

    /// 현재 search filter 를 반환한다.
    pub fn event_filter(&self) -> &str {
        &self.state.event_filter
    }

    /// search filter 를 비운다 (전체 노출 상태로 복귀).
    pub fn clear_event_filter(&mut self) {
        self.state.event_filter.clear();
    }
}

impl Default for HooksPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — SPEC-V3-013 MS-4a HooksPane skeleton
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-V13-13: HooksPane 타이틀이 "Hooks" 이다.
    #[test]
    fn hooks_pane_title_is_hooks() {
        assert_eq!(HooksPane::title(), "Hooks");
    }

    /// AC-V13-13: 설명이 비어있지 않고 "27" 카탈로그 사이즈를 언급한다.
    #[test]
    fn hooks_pane_description_mentions_catalog_size() {
        let desc = HooksPane::description();
        assert!(!desc.is_empty(), "description must not be empty");
        assert!(
            desc.contains("27"),
            "description should mention the catalog size: {desc}"
        );
    }

    /// AC-V13-14: known_events() 가 정확히 27 개를 반환한다.
    #[test]
    fn hooks_pane_known_events_has_27_entries() {
        assert_eq!(HooksPane::known_events().len(), 27);
    }

    /// AC-V13-14: known_events 안에 핵심 이벤트들이 포함되어 있다.
    #[test]
    fn hooks_pane_known_events_includes_critical_events() {
        let events = HooksPane::known_events();
        for required in [
            "PreToolUse",
            "PostToolUse",
            "Stop",
            "UserPromptSubmit",
            "WorktreeCreate",
            "WorktreeRemove",
        ] {
            assert!(
                events.contains(&required),
                "known_events missing critical event: {required}"
            );
        }
    }

    /// AC-V13-14: 이벤트 이름은 모두 unique.
    #[test]
    fn hooks_pane_known_events_are_unique() {
        let events = HooksPane::known_events();
        let unique: std::collections::HashSet<&&str> = events.iter().collect();
        assert_eq!(unique.len(), events.len(), "duplicate hook event detected");
    }

    /// AC-V13-15: 빈 filter 는 전체 카탈로그를 노출한다.
    #[test]
    fn hooks_pane_empty_filter_shows_all_events() {
        let pane = HooksPane::new();
        assert_eq!(pane.event_filter(), "");
        assert_eq!(pane.visible_count(), 27);
    }

    /// AC-V13-15: case-insensitive substring 매치.
    #[test]
    fn hooks_pane_filter_case_insensitive_substring_match() {
        let mut pane = HooksPane::new();
        pane.set_event_filter("worktree");
        let visible = pane.visible_events();
        assert!(
            visible
                .iter()
                .all(|name| name.to_lowercase().contains("worktree"))
        );
        // WorktreeCreate + WorktreeRemove
        assert_eq!(visible.len(), 2);
    }

    /// AC-V13-15: 매치 없는 filter 는 빈 결과 (no-op safe).
    #[test]
    fn hooks_pane_filter_no_match_returns_empty() {
        let mut pane = HooksPane::new();
        pane.set_event_filter("ZZZNonexistentZZZ");
        assert_eq!(pane.visible_count(), 0);
    }

    /// AC-V13-16: clear_event_filter 가 전체 노출 상태로 되돌린다.
    #[test]
    fn hooks_pane_clear_filter_restores_full_view() {
        let mut pane = HooksPane::new();
        pane.set_event_filter("worktree");
        assert_eq!(pane.visible_count(), 2);
        pane.clear_event_filter();
        assert_eq!(pane.event_filter(), "");
        assert_eq!(pane.visible_count(), 27);
    }

    /// with_state() 생성자가 지정된 상태를 유지한다.
    #[test]
    fn hooks_pane_with_state_preserves_filter() {
        let state = HooksState {
            event_filter: "post".to_string(),
        };
        let pane = HooksPane::with_state(state);
        assert_eq!(pane.event_filter(), "post");
        // PostToolUse, PostEditFile, PostReadFile, PostBash, PostWebFetch,
        // PostTaskCreate, PostSubagentSpawn, PostCompact = 8 이벤트.
        assert_eq!(pane.visible_count(), 8);
    }
}
