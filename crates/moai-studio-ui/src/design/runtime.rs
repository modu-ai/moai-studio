//! design::runtime — ActiveTheme 런타임 dispatch wrapper.
//!
//! SPEC-V3-013 MS-3 AC-V13-11: design::tokens 의 const 값을 읽기 전용으로 유지하면서
//! theme/accent/density/font_size 를 런타임에 적용하는 wrapper.
//!
//! ## 책임
//! - ActiveTheme struct: theme/accent/density/font_size_px 런타임 값 보유.
//! - bg_app/bg_panel/fg_primary/accent_color/font_size_px/spacing_multiplier 메서드.
//! - design::tokens const 는 읽기 전용 (FROZEN per R-V13-3, REQ-V13-065).

use crate::settings::settings_state::{AccentColor, Density, ThemeMode};

// ============================================================
// @MX:ANCHOR: [AUTO] active-theme-runtime
// @MX:REASON: [AUTO] SPEC-V3-013 MS-3. ActiveTheme 은 design::tokens const 의 런타임 dispatch wrapper.
//   fan_in >= 3: lib.rs (init + notify), settings/settings_modal.rs (dismiss → save),
//   settings/panes/appearance.rs (change → update).
//   tokens const 변경 금지 invariant — 이 타입만 dispatch.
// ============================================================

/// 런타임 테마 상태 — UserSettings.appearance 에서 derive 되어 cx.global 로 제공된다.
///
/// @MX:NOTE: [AUTO] partial-unthemed-v0.1.0
/// v0.1.0 단계에서 일부 컴포넌트가 design::tokens const 를 직접 참조함 (R-V13-3).
/// 해당 컴포넌트는 dark theme 색상으로 잔존 (수용 — REQ-V13-066).
/// v0.2.0+ 에서 cx.global::<ActiveTheme>() 점진 마이그레이션 예정.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveTheme {
    /// 현재 활성 테마 (Dark/Light/System)
    pub theme: ThemeMode,
    /// 현재 활성 액센트 색상 (4종)
    pub accent: AccentColor,
    /// 현재 밀도 설정 (Compact/Comfortable)
    pub density: Density,
    /// 현재 폰트 크기 (12~18 px)
    pub font_size_px: f32,
}

impl Default for ActiveTheme {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            accent: AccentColor::Teal,
            density: Density::Comfortable,
            font_size_px: 14.0,
        }
    }
}

impl ActiveTheme {
    /// UserSettings.appearance 로부터 ActiveTheme 을 생성한다 (REQ-V13-061).
    pub fn from_settings(s: &crate::settings::user_settings::AppearanceSettings) -> Self {
        Self {
            theme: s.theme,
            accent: s.accent,
            density: s.density,
            font_size_px: s.font_size_px as f32,
        }
    }

    /// 앱 배경 색상을 반환한다 — theme 에 따라 dispatch (REQ-V13-063).
    pub fn bg_app(&self) -> u32 {
        use crate::design::tokens::theme;
        match self.theme {
            ThemeMode::Dark | ThemeMode::System => theme::dark::background::APP,
            ThemeMode::Light => theme::light::background::APP,
        }
    }

    /// 패널 배경 색상을 반환한다 — theme 에 따라 dispatch (REQ-V13-063).
    pub fn bg_panel(&self) -> u32 {
        use crate::design::tokens::theme;
        match self.theme {
            ThemeMode::Dark | ThemeMode::System => theme::dark::background::PANEL,
            ThemeMode::Light => theme::light::background::PANEL,
        }
    }

    /// 기본 텍스트 색상을 반환한다 — theme 에 따라 dispatch (REQ-V13-063).
    pub fn fg_primary(&self) -> u32 {
        use crate::design::tokens::theme;
        match self.theme {
            ThemeMode::Dark | ThemeMode::System => theme::dark::text::PRIMARY,
            ThemeMode::Light => theme::light::text::PRIMARY,
        }
    }

    /// 현재 accent 색상 값을 반환한다 (REQ-V13-063, AC-V13-5).
    pub fn accent_color(&self) -> u32 {
        self.accent.hex_value()
    }

    /// 폰트 크기를 반환한다 (REQ-V13-063).
    pub fn font_size_px(&self) -> f32 {
        self.font_size_px
    }

    /// density 에 따른 spacing multiplier 를 반환한다 (REQ-V13-063).
    pub fn spacing_multiplier(&self) -> f32 {
        match self.density {
            Density::Compact => 0.85,
            Density::Comfortable => 1.0,
        }
    }
}

// ============================================================
// 단위 테스트 — RED phase (SPEC-V3-013 MS-3 ActiveTheme)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::settings_state::{AccentColor, Density, ThemeMode};
    use crate::settings::user_settings::AppearanceSettings;

    /// 기본 ActiveTheme 은 Dark/Teal/Comfortable/14.0 이다.
    #[test]
    fn active_theme_default_values() {
        let t = ActiveTheme::default();
        assert_eq!(t.theme, ThemeMode::Dark);
        assert_eq!(t.accent, AccentColor::Teal);
        assert_eq!(t.density, Density::Comfortable);
        assert!((t.font_size_px - 14.0).abs() < f32::EPSILON);
    }

    /// from_settings 가 AppearanceSettings 를 올바르게 변환한다.
    #[test]
    fn active_theme_from_settings_maps_correctly() {
        let settings = AppearanceSettings {
            theme: ThemeMode::Light,
            density: Density::Compact,
            accent: AccentColor::Violet,
            font_size_px: 16,
        };
        let t = ActiveTheme::from_settings(&settings);
        assert_eq!(t.theme, ThemeMode::Light);
        assert_eq!(t.accent, AccentColor::Violet);
        assert_eq!(t.density, Density::Compact);
        assert!((t.font_size_px - 16.0).abs() < f32::EPSILON);
    }

    /// Dark theme 시 bg_app 이 dark background 값이다.
    #[test]
    fn bg_app_dark_theme_returns_dark_value() {
        let t = ActiveTheme::default();
        let bg = t.bg_app();
        // design::tokens::theme::dark::background::APP 와 같아야 함.
        use crate::design::tokens::theme;
        assert_eq!(bg, theme::dark::background::APP);
    }

    /// Light theme 시 bg_app 이 light background 값이다.
    #[test]
    fn bg_app_light_theme_returns_light_value() {
        let t = ActiveTheme {
            theme: ThemeMode::Light,
            ..ActiveTheme::default()
        };
        let bg = t.bg_app();
        use crate::design::tokens::theme;
        assert_eq!(bg, theme::light::background::APP);
    }

    /// accent_color() 가 AccentColor enum 에 따라 올바른 hex 값을 반환한다 (AC-V13-5).
    #[test]
    fn accent_color_violet_returns_correct_hex() {
        let t = ActiveTheme {
            accent: AccentColor::Violet,
            ..ActiveTheme::default()
        };
        assert_eq!(t.accent_color(), 0x6a4cc7);
    }

    /// accent_color() — Teal 이 0x144a46 이다.
    #[test]
    fn accent_color_teal_returns_correct_hex() {
        let t = ActiveTheme::default();
        assert_eq!(t.accent_color(), 0x144a46);
    }

    /// accent_color() — Blue 이 0x2563eb 이다.
    #[test]
    fn accent_color_blue_returns_correct_hex() {
        let t = ActiveTheme {
            accent: AccentColor::Blue,
            ..ActiveTheme::default()
        };
        assert_eq!(t.accent_color(), 0x2563eb);
    }

    /// accent_color() — Cyan 이 0x06b6d4 이다.
    #[test]
    fn accent_color_cyan_returns_correct_hex() {
        let t = ActiveTheme {
            accent: AccentColor::Cyan,
            ..ActiveTheme::default()
        };
        assert_eq!(t.accent_color(), 0x06b6d4);
    }

    /// spacing_multiplier() — Compact = 0.85.
    #[test]
    fn spacing_multiplier_compact_is_0_85() {
        let t = ActiveTheme {
            density: Density::Compact,
            ..ActiveTheme::default()
        };
        assert!((t.spacing_multiplier() - 0.85).abs() < f32::EPSILON);
    }

    /// spacing_multiplier() — Comfortable = 1.0.
    #[test]
    fn spacing_multiplier_comfortable_is_1_0() {
        let t = ActiveTheme::default();
        assert!((t.spacing_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    /// font_size_px() 가 설정된 값을 반환한다.
    #[test]
    fn font_size_px_returns_set_value() {
        let t = ActiveTheme {
            font_size_px: 16.0,
            ..ActiveTheme::default()
        };
        assert!((t.font_size_px() - 16.0).abs() < f32::EPSILON);
    }

    /// System theme 은 Dark 와 동일한 bg_app 을 반환한다.
    #[test]
    fn system_theme_uses_dark_colors() {
        let dark = ActiveTheme::default();
        let system = ActiveTheme {
            theme: ThemeMode::System,
            ..ActiveTheme::default()
        };
        assert_eq!(dark.bg_app(), system.bg_app());
        assert_eq!(dark.bg_panel(), system.bg_panel());
        assert_eq!(dark.fg_primary(), system.fg_primary());
    }

    /// fg_primary() 가 theme 에 따라 올바른 값을 반환한다.
    #[test]
    fn fg_primary_dark_vs_light() {
        use crate::design::tokens::theme;
        let dark = ActiveTheme::default();
        let light = ActiveTheme {
            theme: ThemeMode::Light,
            ..ActiveTheme::default()
        };
        assert_eq!(dark.fg_primary(), theme::dark::text::PRIMARY);
        assert_eq!(light.fg_primary(), theme::light::text::PRIMARY);
    }
}
