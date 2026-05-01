//! RulesPane — Claude Code Rules read-only viewer (skeleton).
//!
//! SPEC-V3-013 MS-4d (audit G-1, v0.1.2 Task 9d): Settings panel 의 Rules
//! section. v0.1.2 단계는 외부에서 주입된 rule 목록을 read-only 로 노출
//! 하고 search filter 만 제공한다. enable/disable 토글, paths frontmatter
//! 편집은 후속 SPEC 으로 carry.
//!
//! Frozen zone (REQ-V13-MS4d-1):
//! - moai-studio-terminal/** 무변경
//! - moai-studio-workspace/** 무변경
//! - settings_state.rs 의 다른 SettingsSection variant 동작 무변경
//!   (Rules variant 추가 + RulesPaneState 새로 노출)

use crate::settings::settings_state::{Rule, RulesPaneState};

// ============================================================
// RulesPane
// ============================================================

/// RulesPane — read-only Claude Code Rule 목록 + search filter.
///
/// @MX:NOTE: [AUTO] rules-pane-skeleton
/// v0.1.2: 외부 주입된 rule 목록의 read-only list + filter. 토글/편집은 별 SPEC.
pub struct RulesPane {
    /// RulesPane 이 소유하는 in-memory 상태 (rule list + filter).
    pub state: RulesPaneState,
}

impl RulesPane {
    /// 빈 rule 목록과 빈 filter 로 새 RulesPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: RulesPaneState::default(),
        }
    }

    /// 지정 상태로 RulesPane 을 생성한다 (테스트 / lib.rs 자동 로드 편의).
    pub fn with_state(state: RulesPaneState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀.
    pub fn title() -> &'static str {
        "Rules"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "Claude Code Rules 목록을 표시합니다. 활성화 토글 및 paths frontmatter 편집은 향후 버전에서 제공됩니다."
    }

    // ---- rule 관리 ----

    /// 외부 (lib.rs) 가 .claude/rules/ 또는 plugin manifest 를 스캔하여 주입한다.
    pub fn set_rules(&mut self, rules: Vec<Rule>) {
        self.state.rules = rules;
    }

    /// 현재 등록된 rule 의 총 개수 (filter 무시).
    pub fn total_count(&self) -> usize {
        self.state.rules.len()
    }

    /// 현재 filter 가 매치하는 rule 만 반환한다.
    pub fn visible_rules(&self) -> Vec<&Rule> {
        self.state.filtered_rules()
    }

    /// 현재 filter 가 매치하는 rule 의 개수.
    pub fn visible_count(&self) -> usize {
        self.state.filtered_rules().len()
    }

    // ---- filter API ----

    /// search filter 를 갱신한다.
    pub fn set_rule_filter(&mut self, filter: impl Into<String>) {
        self.state.rule_filter = filter.into();
    }

    /// 현재 search filter 를 반환한다.
    pub fn rule_filter(&self) -> &str {
        &self.state.rule_filter
    }

    /// search filter 를 비운다 (전체 노출 상태로 복귀).
    pub fn clear_rule_filter(&mut self) {
        self.state.rule_filter.clear();
    }
}

impl Default for RulesPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — SPEC-V3-013 MS-4d RulesPane skeleton
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rules() -> Vec<Rule> {
        vec![
            Rule::new(
                "moai-constitution",
                "Core principles of MoAI orchestration",
                "user",
                true,
            ),
            Rule::new(
                "spec-workflow",
                "SPEC-based development workflow",
                "user",
                true,
            ),
            Rule::new(
                "rust",
                "Rust development guide and patterns",
                "project",
                false,
            ),
        ]
    }

    /// AC-V13-27: RulesPane 타이틀이 "Rules" 이다.
    #[test]
    fn rules_pane_title_is_rules() {
        assert_eq!(RulesPane::title(), "Rules");
    }

    /// AC-V13-27: 설명이 비어있지 않고 "Rules" 키워드를 언급.
    #[test]
    fn rules_pane_description_mentions_rules() {
        let desc = RulesPane::description();
        assert!(!desc.is_empty(), "description must not be empty");
        assert!(
            desc.contains("Rules"),
            "description should mention Rules: {desc}"
        );
    }

    /// AC-V13-28: 빈 rule 목록의 total/visible/filter 는 모두 정합.
    #[test]
    fn rules_pane_default_is_empty() {
        let pane = RulesPane::new();
        assert_eq!(pane.total_count(), 0);
        assert_eq!(pane.visible_count(), 0);
        assert_eq!(pane.rule_filter(), "");
    }

    /// AC-V13-28: set_rules 로 주입한 목록이 그대로 노출된다 (빈 filter).
    #[test]
    fn rules_pane_set_rules_reflects_total() {
        let mut pane = RulesPane::new();
        pane.set_rules(sample_rules());
        assert_eq!(pane.total_count(), 3);
        assert_eq!(pane.visible_count(), 3);
    }

    /// AC-V13-29: filter 는 case-insensitive substring 으로 name 을 매치한다.
    #[test]
    fn rules_pane_filter_matches_name_case_insensitive() {
        let mut pane = RulesPane::with_state(RulesPaneState {
            rule_filter: String::new(),
            rules: sample_rules(),
        });
        pane.set_rule_filter("CONSTITUTION");
        let visible = pane.visible_rules();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "moai-constitution");
    }

    /// AC-V13-29: filter 는 description 도 매치한다 ("workflow").
    #[test]
    fn rules_pane_filter_matches_description() {
        let mut pane = RulesPane::new();
        pane.set_rules(sample_rules());
        pane.set_rule_filter("workflow");
        let visible = pane.visible_rules();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "spec-workflow");
    }

    /// AC-V13-29: 매치 없는 filter 는 빈 결과.
    #[test]
    fn rules_pane_filter_no_match_returns_empty() {
        let mut pane = RulesPane::new();
        pane.set_rules(sample_rules());
        pane.set_rule_filter("nonexistent-rule-zzz");
        assert_eq!(pane.visible_count(), 0);
    }

    /// AC-V13-30: clear_rule_filter 가 전체 목록을 복원한다.
    #[test]
    fn rules_pane_clear_filter_restores_full_list() {
        let mut pane = RulesPane::new();
        pane.set_rules(sample_rules());
        pane.set_rule_filter("rust");
        assert_eq!(pane.visible_count(), 1);
        pane.clear_rule_filter();
        assert_eq!(pane.rule_filter(), "");
        assert_eq!(pane.visible_count(), 3);
    }

    /// AC-V13-31: enabled 와 disabled rule 모두 노출된다 (read-only).
    #[test]
    fn rules_pane_includes_disabled_rules() {
        let mut pane = RulesPane::new();
        pane.set_rules(sample_rules());
        let visible = pane.visible_rules();
        let enabled = visible.iter().filter(|r| r.enabled).count();
        let disabled = visible.iter().filter(|r| !r.enabled).count();
        assert_eq!(enabled, 2);
        assert_eq!(disabled, 1, "rust rule is disabled");
    }

    /// with_state 생성자가 rules + filter 를 모두 보존한다.
    #[test]
    fn rules_pane_with_state_preserves_both_fields() {
        let state = RulesPaneState {
            rule_filter: "spec".to_string(),
            rules: sample_rules(),
        };
        let pane = RulesPane::with_state(state);
        assert_eq!(pane.rule_filter(), "spec");
        assert_eq!(pane.total_count(), 3);
        assert_eq!(pane.visible_count(), 1, "only spec-workflow matches 'spec'");
    }
}
