//! TerminalPane — 터미널 설정 skeleton (1 setting: scrollback_lines).
//!
//! SPEC-V3-013 MS-2: AC-V13-9 (TerminalPane skeleton) 구현.
//! v0.1.0 단계: section title + description + scrollback_lines NumericInput 1개.
//! consumer 모듈 (terminal) 배선은 v0.2.0+ 별 SPEC (REQ-V13-045).

use crate::settings::settings_state::TerminalState;

// ============================================================
// TerminalPane
// ============================================================

/// TerminalPane — 터미널 설정 skeleton.
///
/// @MX:NOTE: [AUTO] terminal-pane-skeleton
/// v0.1.0: scrollback_lines NumericInput 1개만 구현. consumer 배선은 별 SPEC.
pub struct TerminalPane {
    /// TerminalPane 이 소유하는 in-memory 상태.
    pub state: TerminalState,
}

impl TerminalPane {
    /// 기본 TerminalState 로 새 TerminalPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: TerminalState::default(),
        }
    }

    /// 지정 상태로 TerminalPane 을 생성한다 (테스트 편의).
    pub fn with_state(state: TerminalState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀 (REQ-V13-041).
    pub fn title() -> &'static str {
        "Terminal"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "터미널 동작을 설정합니다. 추가 설정은 향후 버전에서 제공됩니다."
    }

    // ---- scrollback_lines control ----

    /// scrollback_lines 를 설정한다. 1000~100000 범위 외는 무시 (REQ-V13-041).
    pub fn set_scrollback_lines(&mut self, lines: u32) -> bool {
        self.state.set_scrollback_lines(lines)
    }

    /// 현재 scrollback_lines 를 반환한다.
    pub fn scrollback_lines(&self) -> u32 {
        self.state.scrollback_lines
    }

    // ---- scrollback_lines 범위 상수 ----

    /// scrollback_lines 최솟값.
    pub const SCROLLBACK_MIN: u32 = 1_000;
    /// scrollback_lines 최댓값.
    pub const SCROLLBACK_MAX: u32 = 100_000;
    /// scrollback_lines 기본값.
    pub const SCROLLBACK_DEFAULT: u32 = 10_000;
}

impl Default for TerminalPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — RED-GREEN phase (SPEC-V3-013 MS-2 TerminalPane)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// TerminalPane 타이틀이 "Terminal" 이다 (REQ-V13-041).
    fn terminal_pane_title_is_terminal() {
        assert_eq!(TerminalPane::title(), "Terminal");
    }

    #[test]
    /// TerminalPane 설명이 비어 있지 않다.
    fn terminal_pane_description_not_empty() {
        assert!(!TerminalPane::description().is_empty());
    }

    #[test]
    /// TerminalPane 기본 scrollback_lines 가 10000 이다 (REQ-V13-041).
    fn terminal_pane_default_scrollback_is_10000() {
        let pane = TerminalPane::new();
        assert_eq!(pane.scrollback_lines(), TerminalPane::SCROLLBACK_DEFAULT);
        assert_eq!(pane.scrollback_lines(), 10_000);
    }

    #[test]
    /// scrollback 1000 설정 성공 (범위 하한).
    fn terminal_pane_set_scrollback_min_accepted() {
        let mut pane = TerminalPane::new();
        assert!(pane.set_scrollback_lines(TerminalPane::SCROLLBACK_MIN));
        assert_eq!(pane.scrollback_lines(), 1_000);
    }

    #[test]
    /// scrollback 100000 설정 성공 (범위 상한).
    fn terminal_pane_set_scrollback_max_accepted() {
        let mut pane = TerminalPane::new();
        assert!(pane.set_scrollback_lines(TerminalPane::SCROLLBACK_MAX));
        assert_eq!(pane.scrollback_lines(), 100_000);
    }

    #[test]
    /// scrollback 999 거부 (범위 하한 미만).
    fn terminal_pane_set_scrollback_below_min_rejected() {
        let mut pane = TerminalPane::new();
        assert!(!pane.set_scrollback_lines(999));
        assert_eq!(pane.scrollback_lines(), 10_000, "기본값 유지");
    }

    #[test]
    /// scrollback 100001 거부 (범위 상한 초과).
    fn terminal_pane_set_scrollback_above_max_rejected() {
        let mut pane = TerminalPane::new();
        assert!(!pane.set_scrollback_lines(100_001));
        assert_eq!(pane.scrollback_lines(), 10_000, "기본값 유지");
    }

    #[test]
    /// scrollback 0 거부.
    fn terminal_pane_set_scrollback_zero_rejected() {
        let mut pane = TerminalPane::new();
        assert!(!pane.set_scrollback_lines(0));
        assert_eq!(pane.scrollback_lines(), 10_000);
    }

    #[test]
    /// with_state() 생성자가 지정된 상태를 유지한다.
    fn terminal_pane_with_state_preserves_state() {
        let state = TerminalState {
            scrollback_lines: 5_000,
        };
        let pane = TerminalPane::with_state(state);
        assert_eq!(pane.scrollback_lines(), 5_000);
    }
}
