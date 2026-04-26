//! AppearancePane — 외모 설정 (theme/density/accent/font_size).
//!
//! SPEC-V3-013 MS-1: AC-V13-4 ~ AC-V13-6 구현.
//! MS-1 단계: in-memory 상태만 관리. ActiveTheme global + 영속화는 MS-3.

use crate::settings::settings_state::{AccentColor, AppearanceState, Density, ThemeMode};

// ============================================================
// AppearancePane
// ============================================================

/// AppearancePane — 외모 설정 4 controls 를 보유하는 pane.
///
/// @MX:NOTE: [AUTO] appearance-pane-ms1-scope
/// MS-1 단계: in-memory AppearanceState 변경만. ActiveTheme global 연동은 MS-3.
pub struct AppearancePane {
    /// AppearancePane 이 소유하는 in-memory 상태.
    pub state: AppearanceState,
}

impl AppearancePane {
    /// 기본 AppearanceState 로 새 AppearancePane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: AppearanceState::default(),
        }
    }

    /// 지정 상태로 AppearancePane 을 생성한다 (테스트 편의).
    pub fn with_state(state: AppearanceState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀.
    pub fn title() -> &'static str {
        "Appearance"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "테마, 밀도, 액센트 색상, 폰트 크기를 설정합니다."
    }

    // ---- theme control ----

    /// theme RadioGroup 선택을 반영한다 (AC-V13-4 in-memory).
    pub fn set_theme(&mut self, theme: ThemeMode) {
        self.state.theme = theme;
    }

    // ---- density control ----

    /// density ToggleGroup 선택을 반영한다 (REQ-V13-022).
    pub fn set_density(&mut self, density: Density) {
        self.state.density = density;
    }

    // ---- accent control ----

    /// accent ColorSwatch 클릭을 반영한다 (AC-V13-5 in-memory).
    pub fn set_accent(&mut self, accent: AccentColor) {
        self.state.accent = accent;
    }

    // ---- font_size control ----

    /// font_size Slider 변경을 반영한다. 12~18 범위 외는 무시 (AC-V13-6).
    /// 범위 내면 true, 범위 외면 false 반환.
    pub fn set_font_size(&mut self, px: u8) -> bool {
        self.state.set_font_size(px)
    }

    // ---- 조회 편의 메서드 ----

    /// 현재 accent 색상의 hex 값을 반환한다.
    pub fn accent_hex(&self) -> u32 {
        self.state.accent.hex_value()
    }

    /// 현재 spacing multiplier 를 반환한다.
    pub fn spacing_multiplier(&self) -> f32 {
        self.state.spacing_multiplier()
    }
}

impl Default for AppearancePane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — RED phase (SPEC-V3-013 MS-1 AppearancePane)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// AppearancePane 초기 상태가 SPEC 기본값과 일치한다.
    fn appearance_pane_default_state() {
        let pane = AppearancePane::new();
        assert_eq!(pane.state.theme, ThemeMode::Dark);
        assert_eq!(pane.state.density, Density::Comfortable);
        assert_eq!(pane.state.accent, AccentColor::Teal);
        assert_eq!(pane.state.font_size_px, 14);
    }

    #[test]
    /// theme 을 Light 로 설정하면 state.theme 이 Light 가 된다 (AC-V13-4 in-memory).
    fn set_theme_light_updates_state() {
        let mut pane = AppearancePane::new();
        pane.set_theme(ThemeMode::Light);
        assert_eq!(pane.state.theme, ThemeMode::Light);
    }

    #[test]
    /// theme 을 System 으로 설정하면 state.theme 이 System 이 된다.
    fn set_theme_system_updates_state() {
        let mut pane = AppearancePane::new();
        pane.set_theme(ThemeMode::System);
        assert_eq!(pane.state.theme, ThemeMode::System);
    }

    #[test]
    /// density 를 Compact 로 설정하면 state.density 가 Compact 가 된다 (REQ-V13-022).
    fn set_density_compact_updates_state() {
        let mut pane = AppearancePane::new();
        pane.set_density(Density::Compact);
        assert_eq!(pane.state.density, Density::Compact);
        assert!((pane.spacing_multiplier() - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    /// density 를 Comfortable 로 설정하면 multiplier 가 1.0 이다.
    fn set_density_comfortable_multiplier_is_1() {
        let mut pane = AppearancePane::new();
        pane.set_density(Density::Compact);
        pane.set_density(Density::Comfortable);
        assert!((pane.spacing_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    /// accent 를 Violet 으로 설정하면 hex 가 0x6a4cc7 이다 (AC-V13-5 in-memory).
    fn set_accent_violet_hex_value() {
        let mut pane = AppearancePane::new();
        pane.set_accent(AccentColor::Violet);
        assert_eq!(pane.state.accent, AccentColor::Violet);
        assert_eq!(pane.accent_hex(), 0x6a4cc7);
    }

    #[test]
    /// accent 를 Blue 로 설정하면 hex 가 0x2563eb 이다.
    fn set_accent_blue_hex_value() {
        let mut pane = AppearancePane::new();
        pane.set_accent(AccentColor::Blue);
        assert_eq!(pane.accent_hex(), 0x2563eb);
    }

    #[test]
    /// accent 를 Cyan 으로 설정하면 hex 가 0x06b6d4 이다.
    fn set_accent_cyan_hex_value() {
        let mut pane = AppearancePane::new();
        pane.set_accent(AccentColor::Cyan);
        assert_eq!(pane.accent_hex(), 0x06b6d4);
    }

    #[test]
    /// font_size 12 설정 성공 (경계 하한).
    fn set_font_size_min_boundary_accepted() {
        let mut pane = AppearancePane::new();
        assert!(pane.set_font_size(12));
        assert_eq!(pane.state.font_size_px, 12);
    }

    #[test]
    /// font_size 18 설정 성공 (경계 상한).
    fn set_font_size_max_boundary_accepted() {
        let mut pane = AppearancePane::new();
        assert!(pane.set_font_size(18));
        assert_eq!(pane.state.font_size_px, 18);
    }

    #[test]
    /// font_size 11 설정 실패 (AC-V13-6 — 범위 하한 미만).
    fn set_font_size_below_min_rejected() {
        let mut pane = AppearancePane::new();
        let original = pane.state.font_size_px;
        assert!(!pane.set_font_size(11));
        assert_eq!(pane.state.font_size_px, original);
    }

    #[test]
    /// font_size 19 설정 실패 (AC-V13-6 — 범위 상한 초과).
    fn set_font_size_above_max_rejected() {
        let mut pane = AppearancePane::new();
        let original = pane.state.font_size_px;
        assert!(!pane.set_font_size(19));
        assert_eq!(pane.state.font_size_px, original);
    }

    #[test]
    /// font_size 0 설정 실패.
    fn set_font_size_zero_rejected() {
        let mut pane = AppearancePane::new();
        assert!(!pane.set_font_size(0));
        assert_eq!(pane.state.font_size_px, 14);
    }

    #[test]
    /// font_size 255 (u8 최대) 설정 실패.
    fn set_font_size_u8_max_rejected() {
        let mut pane = AppearancePane::new();
        assert!(!pane.set_font_size(255));
        assert_eq!(pane.state.font_size_px, 14);
    }

    #[test]
    /// with_state() 로 생성된 pane 이 지정된 상태를 유지한다.
    fn with_state_constructor_preserves_state() {
        let custom = AppearanceState {
            theme: ThemeMode::Light,
            density: Density::Compact,
            accent: AccentColor::Blue,
            font_size_px: 16,
        };
        let pane = AppearancePane::with_state(custom.clone());
        assert_eq!(pane.state, custom);
    }

    #[test]
    /// 연속적인 상태 변경이 올바르게 누적된다.
    fn sequential_state_mutations_accumulate() {
        let mut pane = AppearancePane::new();
        pane.set_theme(ThemeMode::Light);
        pane.set_density(Density::Compact);
        pane.set_accent(AccentColor::Violet);
        assert!(pane.set_font_size(16));

        assert_eq!(pane.state.theme, ThemeMode::Light);
        assert_eq!(pane.state.density, Density::Compact);
        assert_eq!(pane.state.accent, AccentColor::Violet);
        assert_eq!(pane.state.font_size_px, 16);
    }
}
