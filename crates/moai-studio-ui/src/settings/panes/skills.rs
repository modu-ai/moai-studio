//! SkillsPane — Claude Code Skills read-only viewer (skeleton).
//!
//! SPEC-V3-013 MS-4c (audit G-1, v0.1.2 Task 9c): Settings panel 의 Skills
//! section. v0.1.2 단계는 외부에서 주입된 skill 목록을 read-only 로 노출
//! 하고 search filter 만 제공한다. enable/disable 토글, 신규 skill 등록,
//! frontmatter 편집은 후속 SPEC 으로 carry.
//!
//! Frozen zone (REQ-V13-MS4c-1):
//! - moai-studio-terminal/** 무변경
//! - moai-studio-workspace/** 무변경
//! - settings_state.rs 의 다른 SettingsSection variant 동작 무변경
//!   (Skills variant 추가 + SkillsPaneState 새로 노출)

use crate::settings::settings_state::{Skill, SkillsPaneState};

// ============================================================
// SkillsPane
// ============================================================

/// SkillsPane — read-only Claude Code Skill 목록 + search filter.
///
/// @MX:NOTE: [AUTO] skills-pane-skeleton
/// v0.1.2: 외부 주입된 skill 목록의 read-only list + filter. 토글/편집은 별 SPEC.
pub struct SkillsPane {
    /// SkillsPane 이 소유하는 in-memory 상태 (skill list + filter).
    pub state: SkillsPaneState,
}

impl SkillsPane {
    /// 빈 skill 목록과 빈 filter 로 새 SkillsPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: SkillsPaneState::default(),
        }
    }

    /// 지정 상태로 SkillsPane 을 생성한다 (테스트 / lib.rs 자동 로드 편의).
    pub fn with_state(state: SkillsPaneState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀.
    pub fn title() -> &'static str {
        "Skills"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "Claude Code Skills 목록을 표시합니다. 활성화 토글 및 frontmatter 편집은 향후 버전에서 제공됩니다."
    }

    // ---- skill 관리 ----

    /// 외부 (lib.rs) 가 ~/.claude/skills/ 또는 plugin manifest 를 스캔하여 주입한다.
    pub fn set_skills(&mut self, skills: Vec<Skill>) {
        self.state.skills = skills;
    }

    /// 현재 등록된 skill 의 총 개수 (filter 무시).
    pub fn total_count(&self) -> usize {
        self.state.skills.len()
    }

    /// 현재 filter 가 매치하는 skill 만 반환한다.
    pub fn visible_skills(&self) -> Vec<&Skill> {
        self.state.filtered_skills()
    }

    /// 현재 filter 가 매치하는 skill 의 개수.
    pub fn visible_count(&self) -> usize {
        self.state.filtered_skills().len()
    }

    // ---- filter API ----

    /// search filter 를 갱신한다.
    pub fn set_skill_filter(&mut self, filter: impl Into<String>) {
        self.state.skill_filter = filter.into();
    }

    /// 현재 search filter 를 반환한다.
    pub fn skill_filter(&self) -> &str {
        &self.state.skill_filter
    }

    /// search filter 를 비운다 (전체 노출 상태로 복귀).
    pub fn clear_skill_filter(&mut self) {
        self.state.skill_filter.clear();
    }
}

impl Default for SkillsPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — SPEC-V3-013 MS-4c SkillsPane skeleton
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skills() -> Vec<Skill> {
        vec![
            Skill::new(
                "moai-foundation-cc",
                "Canonical Claude Code authoring kit",
                "user",
                true,
            ),
            Skill::new(
                "moai-workflow-tdd",
                "Test-Driven Development workflow specialist",
                "user",
                true,
            ),
            Skill::new(
                "moai-domain-frontend",
                "Frontend development specialist",
                "user",
                false,
            ),
        ]
    }

    /// AC-V13-22: SkillsPane 타이틀이 "Skills" 이다.
    #[test]
    fn skills_pane_title_is_skills() {
        assert_eq!(SkillsPane::title(), "Skills");
    }

    /// AC-V13-22: 설명이 비어있지 않고 "Skills" 키워드를 언급.
    #[test]
    fn skills_pane_description_mentions_skills() {
        let desc = SkillsPane::description();
        assert!(!desc.is_empty(), "description must not be empty");
        assert!(
            desc.contains("Skills"),
            "description should mention Skills: {desc}"
        );
    }

    /// AC-V13-23: 빈 skill 목록의 total/visible/filter 는 모두 정합.
    #[test]
    fn skills_pane_default_is_empty() {
        let pane = SkillsPane::new();
        assert_eq!(pane.total_count(), 0);
        assert_eq!(pane.visible_count(), 0);
        assert_eq!(pane.skill_filter(), "");
    }

    /// AC-V13-23: set_skills 로 주입한 목록이 그대로 노출된다 (빈 filter).
    #[test]
    fn skills_pane_set_skills_reflects_total() {
        let mut pane = SkillsPane::new();
        pane.set_skills(sample_skills());
        assert_eq!(pane.total_count(), 3);
        assert_eq!(pane.visible_count(), 3);
    }

    /// AC-V13-24: filter 는 case-insensitive substring 으로 name 을 매치한다.
    #[test]
    fn skills_pane_filter_matches_name_case_insensitive() {
        let mut pane = SkillsPane::with_state(SkillsPaneState {
            skill_filter: String::new(),
            skills: sample_skills(),
        });
        pane.set_skill_filter("FOUNDATION");
        let visible = pane.visible_skills();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "moai-foundation-cc");
    }

    /// AC-V13-24: filter 는 description 도 매치한다 ("Test-Driven").
    #[test]
    fn skills_pane_filter_matches_description() {
        let mut pane = SkillsPane::new();
        pane.set_skills(sample_skills());
        pane.set_skill_filter("test-driven");
        let visible = pane.visible_skills();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "moai-workflow-tdd");
    }

    /// AC-V13-24: 매치 없는 filter 는 빈 결과.
    #[test]
    fn skills_pane_filter_no_match_returns_empty() {
        let mut pane = SkillsPane::new();
        pane.set_skills(sample_skills());
        pane.set_skill_filter("nonexistent-skill-zzz");
        assert_eq!(pane.visible_count(), 0);
    }

    /// AC-V13-25: clear_skill_filter 가 전체 목록을 복원한다.
    #[test]
    fn skills_pane_clear_filter_restores_full_list() {
        let mut pane = SkillsPane::new();
        pane.set_skills(sample_skills());
        pane.set_skill_filter("frontend");
        assert_eq!(pane.visible_count(), 1);
        pane.clear_skill_filter();
        assert_eq!(pane.skill_filter(), "");
        assert_eq!(pane.visible_count(), 3);
    }

    /// AC-V13-26: enabled 와 disabled skill 모두 노출된다 (read-only).
    #[test]
    fn skills_pane_includes_disabled_skills() {
        let mut pane = SkillsPane::new();
        pane.set_skills(sample_skills());
        let visible = pane.visible_skills();
        let enabled = visible.iter().filter(|s| s.enabled).count();
        let disabled = visible.iter().filter(|s| !s.enabled).count();
        assert_eq!(enabled, 2);
        assert_eq!(disabled, 1, "moai-domain-frontend is disabled");
    }

    /// with_state 생성자가 skills + filter 를 모두 보존한다.
    #[test]
    fn skills_pane_with_state_preserves_both_fields() {
        let state = SkillsPaneState {
            skill_filter: "workflow".to_string(),
            skills: sample_skills(),
        };
        let pane = SkillsPane::with_state(state);
        assert_eq!(pane.skill_filter(), "workflow");
        assert_eq!(pane.total_count(), 3);
        assert_eq!(pane.visible_count(), 1, "only moai-workflow-tdd matches");
    }
}
