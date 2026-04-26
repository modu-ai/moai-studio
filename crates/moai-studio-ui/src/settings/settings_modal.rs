//! SettingsModal — 880×640 컨테이너 + sidebar(200px) + main pane(680px).
//!
//! SPEC-V3-013 MS-1: AC-V13-1 ~ AC-V13-3 구현.
//! MS-1 단계: in-memory 상태 관리 + 레이아웃 상수 정의.
//! RootView 배선 + Cmd+, keybinding 은 lib.rs 와 함께 MS-3 에서.

use crate::settings::settings_state::{SettingsSection, SettingsViewState};

// ============================================================
// 레이아웃 상수 (REQ-V13-010, R-V13-10)
// ============================================================

/// SettingsModal 컨테이너 너비 (880px) — REQ-V13-002.
pub const SETTINGS_MODAL_WIDTH: f32 = 880.0;
/// SettingsModal 컨테이너 높이 (640px) — REQ-V13-002.
pub const SETTINGS_MODAL_HEIGHT: f32 = 640.0;
/// Sidebar 너비 (200px) — REQ-V13-003.
pub const SETTINGS_SIDEBAR_WIDTH: f32 = 200.0;
/// Main pane 너비 (680px = 880 - 200) — REQ-V13-003.
pub const SETTINGS_MAIN_WIDTH: f32 = 680.0;
/// Sidebar section row 높이 (36px) — REQ-V13-011.
pub const SETTINGS_ROW_HEIGHT: f32 = 36.0;

// ============================================================
// SettingsModal
// ============================================================

/// SettingsModal — settings 모달의 주요 컨테이너.
///
/// @MX:ANCHOR: [AUTO] settings-modal-container
/// @MX:REASON: [AUTO] SPEC-V3-013 MS-1. SettingsModal 은 880×640 컨테이너의 진입점.
///   fan_in >= 3: MS-3 lib.rs (mount/dismiss), AC-V13-1 (container layout), AC-V13-2 (sidebar).
pub struct SettingsModal {
    /// 모달의 뷰 상태 (selected_section, appearance state, visibility).
    pub view_state: SettingsViewState,
}

impl SettingsModal {
    /// 기본 SettingsModal 을 생성한다 (Appearance 섹션 선택, 숨김 상태).
    pub fn new() -> Self {
        Self {
            view_state: SettingsViewState::new(),
        }
    }

    // ---- 레이아웃 조회 ----

    /// 컨테이너 너비를 반환한다 (AC-V13-1).
    pub fn width(&self) -> f32 {
        SETTINGS_MODAL_WIDTH
    }

    /// 컨테이너 높이를 반환한다 (AC-V13-1).
    pub fn height(&self) -> f32 {
        SETTINGS_MODAL_HEIGHT
    }

    /// Sidebar 너비를 반환한다 (REQ-V13-003).
    pub fn sidebar_width(&self) -> f32 {
        SETTINGS_SIDEBAR_WIDTH
    }

    /// Main pane 너비를 반환한다 (REQ-V13-003).
    pub fn main_pane_width(&self) -> f32 {
        SETTINGS_MAIN_WIDTH
    }

    // ---- 가시성 제어 ----

    /// SettingsModal 을 표시 상태로 전환한다 (AC-V13-1).
    pub fn mount(&mut self) {
        self.view_state.show();
    }

    /// SettingsModal 을 숨김 상태로 전환한다 (Esc / X 버튼 / scrim click, REQ-V13-004).
    pub fn dismiss(&mut self) {
        self.view_state.hide();
    }

    /// 현재 표시 상태를 반환한다.
    pub fn is_visible(&self) -> bool {
        self.view_state.is_visible
    }

    // ---- Sidebar section 선택 ----

    /// sidebar 의 section row 클릭을 처리한다 (AC-V13-3).
    pub fn select_section(&mut self, section: SettingsSection) {
        self.view_state.select_section(section);
    }

    /// 현재 선택된 section 을 반환한다.
    pub fn selected_section(&self) -> SettingsSection {
        self.view_state.selected_section
    }

    /// sidebar 의 6개 section 을 정해진 순서로 반환한다 (AC-V13-2).
    pub fn sections(&self) -> [SettingsSection; 6] {
        SettingsSection::all()
    }
}

impl Default for SettingsModal {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — RED phase (SPEC-V3-013 MS-1 SettingsModal)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::settings_state::SettingsSection;

    // ---- 컨테이너 레이아웃 tests ----

    #[test]
    /// SettingsModal 컨테이너 너비가 880px 이다 (AC-V13-1).
    fn modal_width_is_880() {
        let modal = SettingsModal::new();
        assert!(
            (modal.width() - 880.0).abs() < f32::EPSILON,
            "width must be 880, got {}",
            modal.width()
        );
    }

    #[test]
    /// SettingsModal 컨테이너 높이가 640px 이다 (AC-V13-1).
    fn modal_height_is_640() {
        let modal = SettingsModal::new();
        assert!(
            (modal.height() - 640.0).abs() < f32::EPSILON,
            "height must be 640, got {}",
            modal.height()
        );
    }

    #[test]
    /// sidebar 너비가 200px 이다 (REQ-V13-003).
    fn sidebar_width_is_200() {
        let modal = SettingsModal::new();
        assert!(
            (modal.sidebar_width() - 200.0).abs() < f32::EPSILON,
            "sidebar width must be 200, got {}",
            modal.sidebar_width()
        );
    }

    #[test]
    /// main pane 너비가 680px 이다 (REQ-V13-003).
    fn main_pane_width_is_680() {
        let modal = SettingsModal::new();
        assert!(
            (modal.main_pane_width() - 680.0).abs() < f32::EPSILON,
            "main pane width must be 680, got {}",
            modal.main_pane_width()
        );
    }

    #[test]
    /// sidebar + main pane 의 합이 880px 이다.
    fn sidebar_plus_main_equals_width() {
        let modal = SettingsModal::new();
        let total = modal.sidebar_width() + modal.main_pane_width();
        assert!(
            (total - modal.width()).abs() < f32::EPSILON,
            "sidebar + main must equal total width, got {total}"
        );
    }

    #[test]
    /// SETTINGS_SIDEBAR_WIDTH 상수가 200.0 이다.
    fn constant_sidebar_width_is_200() {
        assert!((SETTINGS_SIDEBAR_WIDTH - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    /// SETTINGS_MAIN_WIDTH 상수가 680.0 이다.
    fn constant_main_width_is_680() {
        assert!((SETTINGS_MAIN_WIDTH - 680.0).abs() < f32::EPSILON);
    }

    #[test]
    /// SETTINGS_ROW_HEIGHT 상수가 36.0 이다 (REQ-V13-011).
    fn constant_row_height_is_36() {
        assert!((SETTINGS_ROW_HEIGHT - 36.0).abs() < f32::EPSILON);
    }

    // ---- mount/dismiss tests ----

    #[test]
    /// 새 SettingsModal 은 기본적으로 숨김 상태이다.
    fn modal_starts_hidden() {
        let modal = SettingsModal::new();
        assert!(!modal.is_visible());
    }

    #[test]
    /// mount() 후 is_visible() 이 true 가 된다 (AC-V13-1).
    fn mount_makes_modal_visible() {
        let mut modal = SettingsModal::new();
        modal.mount();
        assert!(modal.is_visible());
    }

    #[test]
    /// dismiss() 후 is_visible() 이 false 가 된다 (REQ-V13-004).
    fn dismiss_makes_modal_hidden() {
        let mut modal = SettingsModal::new();
        modal.mount();
        modal.dismiss();
        assert!(!modal.is_visible());
    }

    #[test]
    /// dismiss() 를 mount() 없이 호출해도 panic 이 발생하지 않는다.
    fn dismiss_without_mount_is_safe() {
        let mut modal = SettingsModal::new();
        modal.dismiss(); // must not panic
        assert!(!modal.is_visible());
    }

    #[test]
    /// mount() 를 두 번 호출해도 상태 오염 없이 is_visible() 이 true 이다 (REQ-V13-006).
    fn double_mount_does_not_corrupt_state() {
        let mut modal = SettingsModal::new();
        modal.mount();
        modal.mount();
        assert!(modal.is_visible());
    }

    // ---- sidebar section tests ----

    #[test]
    /// sections() 가 6개 section 을 반환한다 (AC-V13-2).
    fn sections_returns_six() {
        let modal = SettingsModal::new();
        assert_eq!(modal.sections().len(), 6);
    }

    #[test]
    /// 기본 선택 section 이 Appearance 이다 (REQ-V13-012).
    fn default_selected_section_is_appearance() {
        let modal = SettingsModal::new();
        assert_eq!(modal.selected_section(), SettingsSection::Appearance);
    }

    #[test]
    /// Keyboard section 클릭 시 selected_section 이 Keyboard 로 변경된다 (AC-V13-3).
    fn select_keyboard_section_updates_state() {
        let mut modal = SettingsModal::new();
        modal.select_section(SettingsSection::Keyboard);
        assert_eq!(modal.selected_section(), SettingsSection::Keyboard);
    }

    #[test]
    /// 각 section 을 순서대로 선택했을 때 선택이 올바르게 반영된다.
    fn select_all_sections_sequentially() {
        let mut modal = SettingsModal::new();
        for section in SettingsSection::all() {
            modal.select_section(section);
            assert_eq!(modal.selected_section(), section);
        }
    }

    #[test]
    /// sections() 의 첫 번째가 Appearance, 두 번째가 Keyboard 이다 (REQ-V13-010).
    fn sections_order_is_correct() {
        let modal = SettingsModal::new();
        let sections = modal.sections();
        assert_eq!(sections[0], SettingsSection::Appearance);
        assert_eq!(sections[1], SettingsSection::Keyboard);
        assert_eq!(sections[2], SettingsSection::Editor);
        assert_eq!(sections[3], SettingsSection::Terminal);
        assert_eq!(sections[4], SettingsSection::Agent);
        assert_eq!(sections[5], SettingsSection::Advanced);
    }
}
