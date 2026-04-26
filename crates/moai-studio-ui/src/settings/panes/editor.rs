//! EditorPane — 에디터 설정 skeleton (1 setting: tab_size).
//!
//! SPEC-V3-013 MS-2: AC-V13-9 (EditorPane skeleton) 구현.
//! v0.1.0 단계: section title + description + tab_size NumericInput 1개.
//! consumer 모듈 (viewer) 배선은 v0.2.0+ 별 SPEC (REQ-V13-045).

use crate::settings::settings_state::EditorState;

// ============================================================
// EditorPane
// ============================================================

/// EditorPane — 에디터 설정 skeleton.
///
/// @MX:NOTE: [AUTO] editor-pane-skeleton
/// v0.1.0: tab_size NumericInput 1개만 구현. consumer 배선은 MS-3 이후 별 SPEC.
pub struct EditorPane {
    /// EditorPane 이 소유하는 in-memory 상태.
    pub state: EditorState,
}

impl EditorPane {
    /// 기본 EditorState 로 새 EditorPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: EditorState::default(),
        }
    }

    /// 지정 상태로 EditorPane 을 생성한다 (테스트 편의).
    pub fn with_state(state: EditorState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀 (REQ-V13-040).
    pub fn title() -> &'static str {
        "Editor"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "에디터 동작을 설정합니다. 추가 설정은 향후 버전에서 제공됩니다."
    }

    // ---- tab_size control ----

    /// tab_size 를 설정한다. 2~8 범위 외는 무시 (REQ-V13-040).
    pub fn set_tab_size(&mut self, size: u8) -> bool {
        self.state.set_tab_size(size)
    }

    /// 현재 tab_size 를 반환한다.
    pub fn tab_size(&self) -> u8 {
        self.state.tab_size
    }

    // ---- tab_size 범위 상수 ----

    /// tab_size 최솟값.
    pub const TAB_SIZE_MIN: u8 = 2;
    /// tab_size 최댓값.
    pub const TAB_SIZE_MAX: u8 = 8;
    /// tab_size 기본값.
    pub const TAB_SIZE_DEFAULT: u8 = 4;
}

impl Default for EditorPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — RED-GREEN phase (SPEC-V3-013 MS-2 EditorPane)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// EditorPane 타이틀이 "Editor" 이다 (REQ-V13-040).
    fn editor_pane_title_is_editor() {
        assert_eq!(EditorPane::title(), "Editor");
    }

    #[test]
    /// EditorPane 설명이 비어 있지 않다.
    fn editor_pane_description_not_empty() {
        assert!(!EditorPane::description().is_empty());
    }

    #[test]
    /// EditorPane 기본 tab_size 가 4 이다 (REQ-V13-040).
    fn editor_pane_default_tab_size_is_4() {
        let pane = EditorPane::new();
        assert_eq!(pane.tab_size(), EditorPane::TAB_SIZE_DEFAULT);
        assert_eq!(pane.tab_size(), 4);
    }

    #[test]
    /// tab_size 2 설정 성공 (범위 하한).
    fn editor_pane_set_tab_size_min_accepted() {
        let mut pane = EditorPane::new();
        assert!(pane.set_tab_size(EditorPane::TAB_SIZE_MIN));
        assert_eq!(pane.tab_size(), 2);
    }

    #[test]
    /// tab_size 8 설정 성공 (범위 상한).
    fn editor_pane_set_tab_size_max_accepted() {
        let mut pane = EditorPane::new();
        assert!(pane.set_tab_size(EditorPane::TAB_SIZE_MAX));
        assert_eq!(pane.tab_size(), 8);
    }

    #[test]
    /// tab_size 1 설정 거부 (범위 하한 미만).
    fn editor_pane_set_tab_size_below_min_rejected() {
        let mut pane = EditorPane::new();
        assert!(!pane.set_tab_size(1));
        assert_eq!(pane.tab_size(), 4, "기본값 유지");
    }

    #[test]
    /// tab_size 9 설정 거부 (범위 상한 초과).
    fn editor_pane_set_tab_size_above_max_rejected() {
        let mut pane = EditorPane::new();
        assert!(!pane.set_tab_size(9));
        assert_eq!(pane.tab_size(), 4, "기본값 유지");
    }

    #[test]
    /// with_state() 생성자가 지정된 상태를 유지한다.
    fn editor_pane_with_state_preserves_state() {
        let state = EditorState { tab_size: 2 };
        let pane = EditorPane::with_state(state);
        assert_eq!(pane.tab_size(), 2);
    }

    #[test]
    /// 범위 내 여러 값 설정이 올바르게 누적된다.
    fn editor_pane_sequential_tab_size_changes() {
        let mut pane = EditorPane::new();
        for size in 2u8..=8 {
            assert!(pane.set_tab_size(size));
            assert_eq!(pane.tab_size(), size);
        }
    }
}
