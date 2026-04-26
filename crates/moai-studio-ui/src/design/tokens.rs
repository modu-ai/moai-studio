//! 색상 토큰 상수 — `tokens.json` v2.0.0 의 모든 hex 값.
//!
//! @MX:ANCHOR: [AUTO] design-tokens-canonical
//! @MX:REASON: [AUTO] `.moai/design/tokens.json` v2.0.0 및 `colors_and_type.css` 기준.
//!   모두의AI 공식 브랜드 (#144a46 딥 틸 청록) 적용.
//!   fan_in >= 3: lib.rs, tabs/container.rs, panes/render.rs, viewer/*, terminal/*.
//!   변경 시 design/handoff bundle 과 동기화 필수.

// ============================================================
// 브랜드 원색
// ============================================================

/// 브랜드 색상 — 모두의AI 공식 딥 틸 청록
pub mod brand {
    /// 기본 — 어두운 청록 CTA/타이틀/아이콘 (모두의AI 공식 — 변경 금지)
    pub const PRIMARY: u32 = 0x144a46;
    /// hover 상태
    pub const PRIMARY_HOVER: u32 = 0x0e3835;
    /// active/pressed 상태
    pub const PRIMARY_ACTIVE: u32 = 0x0a2825;
    /// 다크 모드용 라이트 청록 (대비 확보)
    pub const PRIMARY_DARK: u32 = 0x22938a;
    /// 다크 모드 hover
    pub const PRIMARY_DARK_HOVER: u32 = 0x2bafa3;
    /// 본문 텍스트 (#000 대체)
    pub const INK: u32 = 0x09110f;
    /// 페이지 배경 (#fff 대체 금지)
    pub const BG_LIGHT: u32 = 0xf3f3f3;
    /// 카드/모달 surface
    pub const SURFACE_LIGHT: u32 = 0xffffff;
}

// ============================================================
// 뉴트럴 그레이 스케일 (50-950)
// ============================================================

/// 뉴트럴 스케일 — tokens.json `color.neutral`
pub mod neutral {
    pub const N50: u32 = 0xf3f3f3;
    pub const N100: u32 = 0xeaeaea;
    pub const N200: u32 = 0xd4d4d4;
    pub const N300: u32 = 0xbcbcbc;
    pub const N400: u32 = 0x959595;
    pub const N500: u32 = 0x6e6e6e;
    pub const N600: u32 = 0x4c4c4c;
    pub const N700: u32 = 0x2e2e2e;
    pub const N800: u32 = 0x1a1f1d;
    pub const N900: u32 = 0x0e1513;
    pub const N950: u32 = 0x09110f;
}

// ============================================================
// 시맨틱 색상
// ============================================================

/// 시맨틱 — tokens.json `color.semantic`
pub mod semantic {
    /// 테스트 통과/빌드 성공/AC GREEN — moai 청록 계열
    pub const SUCCESS: u32 = 0x1c7c70;
    /// 린트 경고/deprecated API/TODO
    pub const WARNING: u32 = 0xc47b2a;
    /// 컴파일 오류/테스트 실패/critical
    pub const DANGER: u32 = 0xc44a3a;
    /// 힌트/노트/중립 알림
    pub const INFO: u32 = 0x2a8a8c;
}

// ============================================================
// IDE 액센트 (Tweakable per chat decision)
// ============================================================

/// IDE surface 액센트 — tokens.json `color.ide_accent`
pub mod ide_accent {
    /// 기본 — 브랜드 정합 우선
    pub const TEAL: u32 = 0x144a46;
    pub const BLUE: u32 = 0x2563eb;
    pub const VIOLET: u32 = 0x6a4cc7;
    pub const CYAN: u32 = 0x06b6d4;
    pub const AMBER: u32 = 0xc47b2a;
    pub const CRIMSON: u32 = 0xc44a3a;
    pub const MINT: u32 = 0x1c7c70;
}

// ============================================================
// 테마 — 다크 (기본)
// ============================================================

/// 다크 테마 — tokens.json `color.theme.dark`
pub mod theme {
    pub mod dark {
        /// 배경 레이어
        pub mod background {
            /// IDE 시안 d-bg
            pub const APP: u32 = 0x0a100e;
            pub const PANEL: u32 = 0x0e1513;
            pub const SURFACE: u32 = 0x131c19;
            pub const ELEVATED: u32 = 0x182320;
        }

        /// 텍스트 레이어
        pub mod text {
            pub const PRIMARY: u32 = 0xe6ebe9;
            pub const SECONDARY: u32 = 0x98a09d;
            pub const TERTIARY: u32 = 0x6b7370;
            pub const DISABLED: u32 = 0x4c4c4c;
            pub const ON_PRIMARY: u32 = 0xe8f3f2;
        }

        /// border — rgba 값은 별도 함수 사용 (GPUI rgba() 필요)
        /// 근사 hex 값 (불투명 배경 위 blend 결과)
        pub mod border {
            /// rgba(255,255,255,0.06) ≈ 0x0f0f0f blend
            pub const SUBTLE_APPROX: u32 = 0x141c1a;
            /// rgba(255,255,255,0.07)
            pub const DEFAULT_APPROX: u32 = 0x161e1c;
            /// rgba(255,255,255,0.14)
            pub const STRONG_APPROX: u32 = 0x20292b;
            /// focus = brand.primary.dark
            pub const FOCUS: u32 = 0x22938a;
        }

        /// 액센트 레이어
        pub mod accent {
            /// base = brand.primary.dark
            pub const BASE: u32 = 0x22938a;
            /// soft = rgba(20,74,70,0.14) ≈
            pub const SOFT_APPROX: u32 = 0x121f1e;
        }
    }

    /// 라이트 테마 — tokens.json `color.theme.light`
    pub mod light {
        pub mod background {
            pub const APP: u32 = 0xf3f3f3;
            pub const PANEL: u32 = 0xffffff;
            pub const SURFACE: u32 = 0xfafaf9;
            pub const ELEVATED: u32 = 0xffffff;
        }

        pub mod text {
            pub const PRIMARY: u32 = 0x09110f;
            pub const SECONDARY: u32 = 0x4c4c4c;
            pub const TERTIARY: u32 = 0x8a908e;
            pub const DISABLED: u32 = 0xbcbcbc;
            pub const ON_PRIMARY: u32 = 0xffffff;
        }

        pub mod border {
            pub const SUBTLE: u32 = 0xeaeaea;
            pub const DEFAULT: u32 = 0xe6e6e3;
            pub const STRONG: u32 = 0xd4d4d0;
            /// focus = brand.primary
            pub const FOCUS: u32 = 0x144a46;
        }

        pub mod accent {
            /// base = brand.primary
            pub const BASE: u32 = 0x144a46;
        }
    }
}

// ============================================================
// 신택스 하이라이트
// ============================================================

/// 코드 에디터 / Markdown 코드블록 syntax highlight
/// tokens.json `color.syntax`
pub mod syntax {
    /// dark 테마
    pub mod dark {
        pub const KEYWORD: u32 = 0xc792ea;
        pub const STRING: u32 = 0x88b780;
        pub const FUNCTION: u32 = 0x4f9fce;
    }
    /// light 테마
    pub mod light {
        pub const KEYWORD: u32 = 0x5e3bb0;
        pub const STRING: u32 = 0x1c7c70;
        pub const FUNCTION: u32 = 0x155b8a;
    }
    /// 테마 공통
    pub const NUMBER: u32 = 0xc47b2a;
    pub const COMMENT: u32 = 0x6b7370;
    pub const TYPE: u32 = 0xd4a45c;
    pub const OPERATOR: u32 = 0x6fc2c2;
    pub const VARIABLE: u32 = 0xe6ebe9;
    pub const CONSTANT: u32 = 0xc44a3a;
    pub const TAG: u32 = 0xc44a3a;
    pub const ATTRIBUTE: u32 = 0xd4a45c;
}

// ============================================================
// @MX 태그 색상
// ============================================================

/// @MX 태그 팝오버 색상 — tokens.json `round2_component.mx_popover.tag.colors`
pub mod mx_tag {
    /// gold ★ — ANCHOR
    pub const ANCHOR: u32 = 0xd4a017;
    /// amber ⚠ — WARN
    pub const WARN: u32 = 0xc47b2a;
    /// info ● — NOTE
    pub const NOTE: u32 = 0x2a8a8c;
    /// violet ◇ — TODO
    pub const TODO: u32 = 0x6a4cc7;
}

// ============================================================
// macOS 트래픽 라이트
// ============================================================

/// macOS 윈도우 컨트롤 트래픽 라이트
pub mod traffic {
    pub const RED: u32 = 0xff5f57;
    pub const YELLOW: u32 = 0xfebc2e;
    pub const GREEN: u32 = 0x28c840;
}

// ============================================================
// 편의 상수 (다크 테마 기본값 flat-alias)
// ============================================================

/// 다크 테마 기본 배경 (tokens.json `theme.dark.background.app`)
pub const BG_APP: u32 = theme::dark::background::APP;
/// 패널 배경
pub const BG_PANEL: u32 = theme::dark::background::PANEL;
/// surface 배경
pub const BG_SURFACE: u32 = theme::dark::background::SURFACE;
/// elevated surface 배경
pub const BG_ELEVATED: u32 = theme::dark::background::ELEVATED;

/// 1차 텍스트
pub const FG_PRIMARY: u32 = theme::dark::text::PRIMARY;
/// 2차 텍스트
pub const FG_SECONDARY: u32 = theme::dark::text::SECONDARY;
/// 3차 텍스트 (muted)
pub const FG_MUTED: u32 = theme::dark::text::TERTIARY;
/// 비활성 텍스트
pub const FG_DISABLED: u32 = theme::dark::text::DISABLED;

/// border subtle 근사값
pub const BORDER_SUBTLE: u32 = theme::dark::border::SUBTLE_APPROX;
/// border strong 근사값
pub const BORDER_STRONG: u32 = theme::dark::border::STRONG_APPROX;

/// 브랜드 액센트 (다크 모드 — 청록)
pub const ACCENT: u32 = brand::PRIMARY_DARK;

// ============================================================
// 단위 테스트 — tokens.json 값 정합 검증
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 브랜드 원색 검증
    #[test]
    fn brand_primary_is_moai_teal() {
        // tokens.json: `color.brand.primary.$value` = "#144a46"
        assert_eq!(brand::PRIMARY, 0x144a46, "모두의AI 딥 틸 청록 불일치");
    }

    #[test]
    fn brand_primary_dark_is_light_teal() {
        // tokens.json: `color.brand.primary.dark.$value` = "#22938a"
        assert_eq!(
            brand::PRIMARY_DARK,
            0x22938a,
            "다크 모드 라이트 청록 불일치"
        );
    }

    // 시맨틱 색상 검증
    #[test]
    fn semantic_success_is_moai_mint() {
        // tokens.json: `color.semantic.success.$value` = "#1c7c70"
        assert_eq!(semantic::SUCCESS, 0x1c7c70);
    }

    #[test]
    fn semantic_danger_is_crimson() {
        // tokens.json: `color.semantic.danger.$value` = "#c44a3a"
        assert_eq!(semantic::DANGER, 0xc44a3a);
    }

    #[test]
    fn semantic_warning_is_amber() {
        // tokens.json: `color.semantic.warning.$value` = "#c47b2a"
        assert_eq!(semantic::WARNING, 0xc47b2a);
    }

    // 다크 테마 검증
    #[test]
    fn dark_bg_app() {
        // tokens.json: `color.theme.dark.background.app.$value` = "#0a100e"
        assert_eq!(theme::dark::background::APP, 0x0a100e);
    }

    #[test]
    fn dark_bg_panel() {
        // tokens.json: `color.theme.dark.background.panel.$value` = "#0e1513"
        assert_eq!(theme::dark::background::PANEL, 0x0e1513);
    }

    #[test]
    fn dark_bg_surface() {
        // tokens.json: `color.theme.dark.background.surface.$value` = "#131c19"
        assert_eq!(theme::dark::background::SURFACE, 0x131c19);
    }

    #[test]
    fn dark_text_primary() {
        // tokens.json: `color.theme.dark.text.primary.$value` = "#e6ebe9"
        assert_eq!(theme::dark::text::PRIMARY, 0xe6ebe9);
    }

    #[test]
    fn dark_text_tertiary() {
        // tokens.json: `color.theme.dark.text.tertiary.$value` = "#6b7370"
        assert_eq!(theme::dark::text::TERTIARY, 0x6b7370);
    }

    // @MX 태그 색상 검증
    #[test]
    fn mx_anchor_is_gold() {
        // tokens.json: `round2_component.mx_popover.tag.colors.ANCHOR.$value` = "#d4a017"
        assert_eq!(mx_tag::ANCHOR, 0xd4a017);
    }

    #[test]
    fn mx_warn_is_amber() {
        // tokens.json: `round2_component.mx_popover.tag.colors.WARN.$value` = "#c47b2a"
        assert_eq!(mx_tag::WARN, 0xc47b2a);
    }

    #[test]
    fn mx_note_is_info_teal() {
        // tokens.json: `round2_component.mx_popover.tag.colors.NOTE.$value` = "#2a8a8c"
        assert_eq!(mx_tag::NOTE, 0x2a8a8c);
    }

    #[test]
    fn mx_todo_is_violet() {
        // tokens.json: `round2_component.mx_popover.tag.colors.TODO.$value` = "#6a4cc7"
        assert_eq!(mx_tag::TODO, 0x6a4cc7);
    }

    // 신택스 색상 검증
    #[test]
    fn syntax_keyword_dark() {
        // tokens.json: `color.syntax.keyword.dark.$value` = "#c792ea"
        assert_eq!(syntax::dark::KEYWORD, 0xc792ea);
    }

    #[test]
    fn syntax_string_dark() {
        // tokens.json: `color.syntax.string.dark.$value` = "#88b780"
        assert_eq!(syntax::dark::STRING, 0x88b780);
    }

    // 트래픽 라이트 검증
    #[test]
    fn traffic_red() {
        assert_eq!(traffic::RED, 0xff5f57);
    }

    #[test]
    fn traffic_yellow() {
        assert_eq!(traffic::YELLOW, 0xfebc2e);
    }

    #[test]
    fn traffic_green() {
        assert_eq!(traffic::GREEN, 0x28c840);
    }

    // flat-alias 검증
    #[test]
    fn flat_alias_bg_panel_matches_dark_panel() {
        assert_eq!(BG_PANEL, theme::dark::background::PANEL);
    }

    #[test]
    fn flat_alias_accent_matches_brand_primary_dark() {
        assert_eq!(ACCENT, brand::PRIMARY_DARK);
    }
}
