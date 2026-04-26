//! AgentPane — 에이전트 설정 skeleton (1 setting: auto_approve Toggle).
//!
//! SPEC-V3-013 MS-2: AC-V13-9 (AgentPane skeleton) 구현.
//! v0.1.0 단계: section title + description + auto_approve Toggle 1개.
//! consumer 모듈 (agent) 배선은 v0.2.0+ 별 SPEC (REQ-V13-045).

use crate::settings::settings_state::AgentState;

// ============================================================
// AgentPane
// ============================================================

/// AgentPane — 에이전트 설정 skeleton.
///
/// @MX:NOTE: [AUTO] agent-pane-skeleton
/// v0.1.0: auto_approve Toggle 1개만 구현. consumer 배선은 별 SPEC.
pub struct AgentPane {
    /// AgentPane 이 소유하는 in-memory 상태.
    pub state: AgentState,
}

impl AgentPane {
    /// 기본 AgentState 로 새 AgentPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: AgentState::default(),
        }
    }

    /// 지정 상태로 AgentPane 을 생성한다 (테스트 편의).
    pub fn with_state(state: AgentState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀 (REQ-V13-042).
    pub fn title() -> &'static str {
        "Agent"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "AI 에이전트 동작을 설정합니다. 추가 설정은 향후 버전에서 제공됩니다."
    }

    // ---- auto_approve control ----

    /// auto_approve 를 직접 설정한다 (REQ-V13-042).
    pub fn set_auto_approve(&mut self, value: bool) {
        self.state.auto_approve = value;
    }

    /// auto_approve 를 토글한다.
    pub fn toggle_auto_approve(&mut self) {
        self.state.toggle_auto_approve();
    }

    /// 현재 auto_approve 값을 반환한다.
    pub fn auto_approve(&self) -> bool {
        self.state.auto_approve
    }
}

impl Default for AgentPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — RED-GREEN phase (SPEC-V3-013 MS-2 AgentPane)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// AgentPane 타이틀이 "Agent" 이다 (REQ-V13-042).
    fn agent_pane_title_is_agent() {
        assert_eq!(AgentPane::title(), "Agent");
    }

    #[test]
    /// AgentPane 설명이 비어 있지 않다.
    fn agent_pane_description_not_empty() {
        assert!(!AgentPane::description().is_empty());
    }

    #[test]
    /// AgentPane 기본 auto_approve 가 false 이다 (REQ-V13-042).
    fn agent_pane_default_auto_approve_is_false() {
        let pane = AgentPane::new();
        assert!(!pane.auto_approve());
    }

    #[test]
    /// set_auto_approve(true) 로 활성화된다.
    fn agent_pane_set_auto_approve_true() {
        let mut pane = AgentPane::new();
        pane.set_auto_approve(true);
        assert!(pane.auto_approve());
    }

    #[test]
    /// set_auto_approve(false) 로 비활성화된다.
    fn agent_pane_set_auto_approve_false() {
        let mut pane = AgentPane::new();
        pane.set_auto_approve(true);
        pane.set_auto_approve(false);
        assert!(!pane.auto_approve());
    }

    #[test]
    /// toggle_auto_approve() 가 상태를 반전한다.
    fn agent_pane_toggle_flips_state() {
        let mut pane = AgentPane::new();
        pane.toggle_auto_approve();
        assert!(pane.auto_approve());
        pane.toggle_auto_approve();
        assert!(!pane.auto_approve());
    }

    #[test]
    /// with_state() 생성자가 지정된 상태를 유지한다.
    fn agent_pane_with_state_preserves_state() {
        let state = AgentState { auto_approve: true };
        let pane = AgentPane::with_state(state);
        assert!(pane.auto_approve());
    }
}
