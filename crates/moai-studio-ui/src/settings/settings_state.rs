//! SettingsViewState — SettingsModal 의 transient UI 상태.
//!
//! 영속화 대상이 아닌 순수 런타임 뷰 상태만 보유한다.
//! UserSettings 영속화 + ActiveTheme 런타임 적용은 MS-3 단계.

// ============================================================
// Section 열거형 (6 sections)
// ============================================================

/// SettingsModal 의 6개 section 식별자.
///
/// @MX:ANCHOR: [AUTO] settings-section-enum
/// @MX:REASON: [AUTO] sidebar row 렌더, main pane swap, selected_section 상태 전이의 공통 타입.
///   fan_in >= 3: settings_state.rs, settings_modal.rs, panes/*.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    /// 외모 설정 (theme/density/accent/font_size)
    Appearance,
    /// 키보드 단축키 설정
    Keyboard,
    /// 에디터 설정 (skeleton)
    Editor,
    /// 터미널 설정 (skeleton)
    Terminal,
    /// 에이전트 설정 (skeleton)
    Agent,
    /// 고급 설정 (skeleton)
    Advanced,
}

impl SettingsSection {
    /// sidebar 에 표시할 레이블 문자열.
    pub fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Keyboard => "Keyboard",
            Self::Editor => "Editor",
            Self::Terminal => "Terminal",
            Self::Agent => "Agent",
            Self::Advanced => "Advanced",
        }
    }

    /// 6개 section 을 정해진 순서대로 반환한다 (REQ-V13-010).
    pub fn all() -> [SettingsSection; 6] {
        [
            Self::Appearance,
            Self::Keyboard,
            Self::Editor,
            Self::Terminal,
            Self::Agent,
            Self::Advanced,
        ]
    }
}

// ============================================================
// AppearanceState — in-memory (MS-3 이전 영속화 없음)
// ============================================================

/// 테마 선택 (dark/light/system).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    /// 다크 테마 (기본값)
    #[default]
    Dark,
    /// 라이트 테마
    Light,
    /// 시스템 설정 따름
    System,
}

/// 밀도 선택 (compact/comfortable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// 컴팩트 — 패딩/행 높이 0.85x 축소
    Compact,
    /// 컴포터블 — 기본 간격 (기본값)
    #[default]
    Comfortable,
}

/// 액센트 색상 선택 (4종).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccentColor {
    /// 틸 청록 (브랜드 기본 — 0x144a46) (기본값)
    #[default]
    Teal,
    /// 블루 (0x2563eb)
    Blue,
    /// 바이올렛 (0x6a4cc7)
    Violet,
    /// 시안 (0x06b6d4)
    Cyan,
}

impl AccentColor {
    /// design::tokens 의 ide_accent 상수를 반환한다.
    pub fn hex_value(self) -> u32 {
        use crate::design::tokens::ide_accent;
        match self {
            Self::Teal => ide_accent::TEAL,
            Self::Blue => ide_accent::BLUE,
            Self::Violet => ide_accent::VIOLET,
            Self::Cyan => ide_accent::CYAN,
        }
    }
}

/// AppearancePane 의 in-memory 상태 (MS-1 범위 — 영속화 없음).
#[derive(Debug, Clone, PartialEq)]
pub struct AppearanceState {
    /// 테마 선택 (default: Dark)
    pub theme: ThemeMode,
    /// 밀도 선택 (default: Comfortable)
    pub density: Density,
    /// 액센트 색상 (default: Teal)
    pub accent: AccentColor,
    /// 폰트 크기 (12~18px, default: 14)
    pub font_size_px: u8,
}

impl Default for AppearanceState {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            density: Density::default(),
            accent: AccentColor::default(),
            font_size_px: 14,
        }
    }
}

impl AppearanceState {
    /// font_size_px 를 설정한다. 12~18 범위 외는 무시한다 (REQ-V13-025).
    pub fn set_font_size(&mut self, px: u8) -> bool {
        if (12..=18).contains(&px) {
            self.font_size_px = px;
            true
        } else {
            false
        }
    }

    /// density 에 따른 spacing multiplier 를 반환한다.
    pub fn spacing_multiplier(&self) -> f32 {
        match self.density {
            Density::Compact => 0.85,
            Density::Comfortable => 1.0,
        }
    }
}

// ============================================================
// SettingsViewState — SettingsModal transient 상태
// ============================================================

/// SettingsModal 의 런타임 전용 뷰 상태.
///
/// @MX:NOTE: [AUTO] settings-view-state-transient
/// 이 상태는 영속화되지 않는다 — UserSettings 영속화는 MS-3.
/// SettingsModal mount 시 항상 default (Appearance section 활성) 로 시작한다.
pub struct SettingsViewState {
    /// 현재 선택된 section (default: Appearance).
    pub selected_section: SettingsSection,
    /// AppearancePane 의 in-memory 상태.
    pub appearance: AppearanceState,
    /// SettingsModal 이 표시 중인지 여부 (mount/dismiss 상태).
    pub is_visible: bool,
}

impl Default for SettingsViewState {
    fn default() -> Self {
        Self {
            selected_section: SettingsSection::Appearance,
            appearance: AppearanceState::default(),
            is_visible: false,
        }
    }
}

impl SettingsViewState {
    /// 새 SettingsViewState 를 생성한다 (default: Appearance 섹션, 숨김 상태).
    pub fn new() -> Self {
        Self::default()
    }

    /// 지정 section 을 선택한다.
    pub fn select_section(&mut self, section: SettingsSection) {
        self.selected_section = section;
    }

    /// SettingsModal 을 mount 상태로 전환한다.
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// SettingsModal 을 dismiss 상태로 전환한다.
    pub fn hide(&mut self) {
        self.is_visible = false;
    }
}

// ============================================================
// 단위 테스트 — RED phase (SPEC-V3-013 MS-1)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- SettingsSection tests ----

    #[test]
    /// SettingsSection::all() 이 6개를 정해진 순서로 반환한다 (REQ-V13-010).
    fn section_all_returns_six_in_order() {
        let all = SettingsSection::all();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], SettingsSection::Appearance);
        assert_eq!(all[1], SettingsSection::Keyboard);
        assert_eq!(all[2], SettingsSection::Editor);
        assert_eq!(all[3], SettingsSection::Terminal);
        assert_eq!(all[4], SettingsSection::Agent);
        assert_eq!(all[5], SettingsSection::Advanced);
    }

    #[test]
    /// 각 section 의 label() 이 올바른 문자열을 반환한다.
    fn section_labels_are_correct() {
        assert_eq!(SettingsSection::Appearance.label(), "Appearance");
        assert_eq!(SettingsSection::Keyboard.label(), "Keyboard");
        assert_eq!(SettingsSection::Editor.label(), "Editor");
        assert_eq!(SettingsSection::Terminal.label(), "Terminal");
        assert_eq!(SettingsSection::Agent.label(), "Agent");
        assert_eq!(SettingsSection::Advanced.label(), "Advanced");
    }

    // ---- AppearanceState tests ----

    #[test]
    /// AppearanceState 기본값이 SPEC 명세와 일치한다.
    fn appearance_state_default_values() {
        let s = AppearanceState::default();
        assert_eq!(s.theme, ThemeMode::Dark);
        assert_eq!(s.density, Density::Comfortable);
        assert_eq!(s.accent, AccentColor::Teal);
        assert_eq!(s.font_size_px, 14);
    }

    #[test]
    /// font_size 12~18 범위 내 설정이 성공한다 (REQ-V13-024).
    fn font_size_valid_range_accepted() {
        let mut s = AppearanceState::default();
        assert!(s.set_font_size(12));
        assert_eq!(s.font_size_px, 12);
        assert!(s.set_font_size(18));
        assert_eq!(s.font_size_px, 18);
        assert!(s.set_font_size(14));
        assert_eq!(s.font_size_px, 14);
    }

    #[test]
    /// font_size 11 (범위 하한 미만) 은 적용되지 않는다 (AC-V13-6).
    fn font_size_below_min_rejected() {
        let mut s = AppearanceState::default();
        let original = s.font_size_px;
        assert!(!s.set_font_size(11));
        assert_eq!(s.font_size_px, original, "범위 외 값 11은 적용되면 안 됨");
    }

    #[test]
    /// font_size 19 (범위 상한 초과) 는 적용되지 않는다 (AC-V13-6).
    fn font_size_above_max_rejected() {
        let mut s = AppearanceState::default();
        let original = s.font_size_px;
        assert!(!s.set_font_size(19));
        assert_eq!(s.font_size_px, original, "범위 외 값 19는 적용되면 안 됨");
    }

    #[test]
    /// font_size 경계값 0 은 적용되지 않는다.
    fn font_size_zero_rejected() {
        let mut s = AppearanceState::default();
        assert!(!s.set_font_size(0));
        assert_eq!(s.font_size_px, 14);
    }

    #[test]
    /// density Compact 시 spacing_multiplier 가 0.85 이다 (REQ-V13-022).
    fn density_compact_multiplier_is_0_85() {
        let s = AppearanceState {
            density: Density::Compact,
            ..Default::default()
        };
        let m = s.spacing_multiplier();
        assert!(
            (m - 0.85).abs() < f32::EPSILON,
            "compact multiplier = 0.85, got {m}"
        );
    }

    #[test]
    /// density Comfortable 시 spacing_multiplier 가 1.0 이다 (REQ-V13-022).
    fn density_comfortable_multiplier_is_1_0() {
        let s = AppearanceState::default();
        assert_eq!(s.density, Density::Comfortable);
        let m = s.spacing_multiplier();
        assert!(
            (m - 1.0).abs() < f32::EPSILON,
            "comfortable multiplier = 1.0, got {m}"
        );
    }

    #[test]
    /// AccentColor::Violet 의 hex_value 가 0x6a4cc7 이다 (AC-V13-5).
    fn accent_violet_hex_is_correct() {
        assert_eq!(AccentColor::Violet.hex_value(), 0x6a4cc7);
    }

    #[test]
    /// AccentColor::Teal 의 hex_value 가 design::tokens::ide_accent::TEAL 과 일치한다.
    fn accent_teal_hex_matches_token() {
        use crate::design::tokens::ide_accent;
        assert_eq!(AccentColor::Teal.hex_value(), ide_accent::TEAL);
    }

    #[test]
    /// AccentColor::Blue hex_value 가 0x2563eb 이다.
    fn accent_blue_hex_is_correct() {
        assert_eq!(AccentColor::Blue.hex_value(), 0x2563eb);
    }

    #[test]
    /// AccentColor::Cyan hex_value 가 0x06b6d4 이다.
    fn accent_cyan_hex_is_correct() {
        assert_eq!(AccentColor::Cyan.hex_value(), 0x06b6d4);
    }

    #[test]
    /// 4개 accent 색상이 모두 다른 값이다.
    fn accent_four_colors_are_distinct() {
        let values = [
            AccentColor::Teal.hex_value(),
            AccentColor::Blue.hex_value(),
            AccentColor::Violet.hex_value(),
            AccentColor::Cyan.hex_value(),
        ];
        // 중복 없이 4개 고유 값
        let mut unique = values.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), 4, "4개 accent 색상이 모두 달라야 함");
    }

    // ---- SettingsViewState tests ----

    #[test]
    /// SettingsViewState 기본값 — Appearance 선택, 숨김 상태.
    fn view_state_default_is_appearance_hidden() {
        let state = SettingsViewState::new();
        assert_eq!(state.selected_section, SettingsSection::Appearance);
        assert!(!state.is_visible);
    }

    #[test]
    /// show() 호출 시 is_visible 이 true 가 된다 (AC-V13-1).
    fn view_state_show_makes_visible() {
        let mut state = SettingsViewState::new();
        state.show();
        assert!(state.is_visible);
    }

    #[test]
    /// hide() 호출 시 is_visible 이 false 가 된다 (REQ-V13-004).
    fn view_state_hide_makes_hidden() {
        let mut state = SettingsViewState::new();
        state.show();
        state.hide();
        assert!(!state.is_visible);
    }

    #[test]
    /// select_section() 이 selected_section 을 업데이트한다 (AC-V13-3).
    fn view_state_select_section_updates_state() {
        let mut state = SettingsViewState::new();
        state.select_section(SettingsSection::Keyboard);
        assert_eq!(state.selected_section, SettingsSection::Keyboard);
        state.select_section(SettingsSection::Advanced);
        assert_eq!(state.selected_section, SettingsSection::Advanced);
    }

    #[test]
    /// AppearanceState theme 변경이 SettingsViewState 에 반영된다 (AC-V13-4 in-memory).
    fn view_state_appearance_theme_mutation() {
        let mut state = SettingsViewState::new();
        assert_eq!(state.appearance.theme, ThemeMode::Dark);
        state.appearance.theme = ThemeMode::Light;
        assert_eq!(state.appearance.theme, ThemeMode::Light);
    }

    #[test]
    /// AppearanceState accent 변경이 반영된다 (AC-V13-5 in-memory).
    fn view_state_appearance_accent_mutation() {
        let mut state = SettingsViewState::new();
        state.appearance.accent = AccentColor::Violet;
        assert_eq!(state.appearance.accent, AccentColor::Violet);
        assert_eq!(state.appearance.accent.hex_value(), 0x6a4cc7);
    }
}
