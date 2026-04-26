//! AdvancedPane — 고급 설정 skeleton (1 setting: experimental_flags placeholder).
//!
//! SPEC-V3-013 MS-2: AC-V13-9 (AdvancedPane skeleton) 구현.
//! v0.1.0 단계: section title + description + experimental_flags (read-only, 빈 목록).
//! consumer 배선은 v0.2.0+ 별 SPEC (REQ-V13-045).

use crate::settings::settings_state::AdvancedState;

// ============================================================
// AdvancedPane
// ============================================================

/// AdvancedPane — 고급 설정 skeleton.
///
/// @MX:NOTE: [AUTO] advanced-pane-skeleton
/// v0.1.0: experimental_flags read-only placeholder 만 구현. 활성화 로직은 별 SPEC.
pub struct AdvancedPane {
    /// AdvancedPane 이 소유하는 in-memory 상태.
    pub state: AdvancedState,
}

impl AdvancedPane {
    /// 기본 AdvancedState 로 새 AdvancedPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: AdvancedState::default(),
        }
    }

    /// 지정 상태로 AdvancedPane 을 생성한다 (테스트 편의).
    pub fn with_state(state: AdvancedState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀 (REQ-V13-043).
    pub fn title() -> &'static str {
        "Advanced"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "고급 설정입니다. 실험적 기능 활성화 등의 설정은 향후 버전에서 제공됩니다."
    }

    // ---- experimental_flags (read-only placeholder) ----

    /// 현재 실험적 플래그 목록을 반환한다 (read-only, REQ-V13-043).
    pub fn experimental_flags(&self) -> &[String] {
        &self.state.experimental_flags
    }

    /// 실험적 플래그 개수를 반환한다.
    pub fn flag_count(&self) -> usize {
        self.state.experimental_flags.len()
    }
}

impl Default for AdvancedPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — RED-GREEN phase (SPEC-V3-013 MS-2 AdvancedPane)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// AdvancedPane 타이틀이 "Advanced" 이다 (REQ-V13-043).
    fn advanced_pane_title_is_advanced() {
        assert_eq!(AdvancedPane::title(), "Advanced");
    }

    #[test]
    /// AdvancedPane 설명이 비어 있지 않다.
    fn advanced_pane_description_not_empty() {
        assert!(!AdvancedPane::description().is_empty());
    }

    #[test]
    /// AdvancedPane 기본 experimental_flags 가 빈 목록이다 (REQ-V13-043).
    fn advanced_pane_default_flags_empty() {
        let pane = AdvancedPane::new();
        assert!(pane.experimental_flags().is_empty());
        assert_eq!(pane.flag_count(), 0);
    }

    #[test]
    /// with_state() 로 플래그가 있는 상태를 생성할 수 있다.
    fn advanced_pane_with_state_preserves_flags() {
        let state = AdvancedState {
            experimental_flags: vec!["flag_a".to_string(), "flag_b".to_string()],
        };
        let pane = AdvancedPane::with_state(state);
        assert_eq!(pane.flag_count(), 2);
        assert_eq!(pane.experimental_flags()[0], "flag_a");
        assert_eq!(pane.experimental_flags()[1], "flag_b");
    }

    #[test]
    /// experimental_flags() 가 슬라이스를 반환한다.
    fn advanced_pane_flags_returns_slice() {
        let pane = AdvancedPane::new();
        let flags: &[String] = pane.experimental_flags();
        assert_eq!(flags.len(), 0);
    }
}
