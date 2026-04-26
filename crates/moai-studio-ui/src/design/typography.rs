//! 타이포그래피 상수 — `tokens.json` v2.0.0 `typography` 섹션.
//!
//! @MX:NOTE: [AUTO] design-typography-canonical
//! tokens.json `typography` 섹션의 값을 GPUI f32/u16 상수로 변환.
//! Source: `.moai/design/tokens.json` (Pretendard 9 weight + JetBrains Mono).

// ============================================================
// 폰트 크기 (px 기준 — 1rem = 16px 가정)
// ============================================================

/// 캡션 (12px, tokens: `fontSize.xs` = "0.75rem")
pub const FONT_SIZE_XS_PX: f32 = 12.0;
/// 본문 tight (14px, tokens: `fontSize.sm` = "0.875rem")
pub const FONT_SIZE_SM_PX: f32 = 14.0;
/// 본문 (16px, tokens: `fontSize.base` = "1rem")
pub const FONT_SIZE_BASE_PX: f32 = 16.0;
/// h4 (18px, tokens: `fontSize.lg` = "1.125rem")
pub const FONT_SIZE_LG_PX: f32 = 18.0;
/// 20px (tokens: `fontSize.xl` = "1.25rem")
pub const FONT_SIZE_XL_PX: f32 = 20.0;
/// h3 (24px, tokens: `fontSize.2xl` = "1.5rem")
pub const FONT_SIZE_2XL_PX: f32 = 24.0;
/// h2 (30px, tokens: `fontSize.3xl` = "1.875rem")
pub const FONT_SIZE_3XL_PX: f32 = 30.0;
/// h1 (36px, tokens: `fontSize.4xl` = "2.25rem")
pub const FONT_SIZE_4XL_PX: f32 = 36.0;
/// 48px (tokens: `fontSize.5xl` = "3rem")
pub const FONT_SIZE_5XL_PX: f32 = 48.0;
/// 60px (tokens: `fontSize.6xl` = "3.75rem")
pub const FONT_SIZE_6XL_PX: f32 = 60.0;

// ============================================================
// 줄 높이
// ============================================================

/// 히어로/디스플레이 (tokens: `lineHeight.tight` = 1.05)
pub const LH_TIGHT: f32 = 1.05;
/// snug (tokens: `lineHeight.snug` = 1.25)
pub const LH_SNUG: f32 = 1.25;
/// 일반 (tokens: `lineHeight.normal` = 1.5)
pub const LH_NORMAL: f32 = 1.5;
/// 마크다운 본문 (tokens: `lineHeight.relaxed` = 1.75)
pub const LH_RELAXED: f32 = 1.75;

// ============================================================
// 자간 (letter-spacing, em 단위)
// ============================================================

/// 히어로 자간 (tokens: `letterSpacing.display.tight` = "-0.075em")
pub const TRACKING_DISPLAY_TIGHT: f32 = -0.075;
/// 메인 타이틀 자간 (tokens: `letterSpacing.display` = "-0.05em")
pub const TRACKING_DISPLAY: f32 = -0.05;
/// heading 자간 (tokens: `letterSpacing.heading` = "-0.05em")
pub const TRACKING_HEADING: f32 = -0.05;
/// 본문 자간 (tokens: `letterSpacing.body` = "-0.025em")
pub const TRACKING_BODY: f32 = -0.025;
/// 본문 tight 자간 (tokens: `letterSpacing.body.tight` = "-0.05em")
pub const TRACKING_BODY_TIGHT: f32 = -0.05;
/// caption 자간 (tokens: `letterSpacing.caption` = "0")
pub const TRACKING_CAPTION: f32 = 0.0;

// ============================================================
// 폰트 웨이트
// ============================================================

/// 폰트 웨이트
pub mod weight {
    /// thin (tokens: `fontWeight.thin` = 100)
    pub const THIN: u16 = 100;
    /// extralight (tokens: `fontWeight.extralight` = 200)
    pub const EXTRALIGHT: u16 = 200;
    /// light (tokens: `fontWeight.light` = 300)
    pub const LIGHT: u16 = 300;
    /// regular (tokens: `fontWeight.regular` = 400)
    pub const REGULAR: u16 = 400;
    /// medium (tokens: `fontWeight.medium` = 500)
    pub const MEDIUM: u16 = 500;
    /// semibold (tokens: `fontWeight.semibold` = 600)
    pub const SEMIBOLD: u16 = 600;
    /// bold (tokens: `fontWeight.bold` = 700)
    pub const BOLD: u16 = 700;
    /// extrabold (tokens: `fontWeight.extrabold` = 800)
    pub const EXTRABOLD: u16 = 800;
    /// black (tokens: `fontWeight.black` = 900)
    pub const BLACK: u16 = 900;
}

// ============================================================
// 단위 테스트 — tokens.json 값 정합 검증
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_size_base_is_16() {
        // tokens.json: `typography.fontSize.base` = "1rem" = 16px
        assert!((FONT_SIZE_BASE_PX - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn font_size_sm_is_14() {
        // tokens.json: `typography.fontSize.sm` = "0.875rem" = 14px
        assert!((FONT_SIZE_SM_PX - 14.0).abs() < f32::EPSILON);
    }

    #[test]
    fn font_size_xs_is_12() {
        // tokens.json: `typography.fontSize.xs` = "0.75rem" = 12px
        assert!((FONT_SIZE_XS_PX - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn line_height_normal() {
        // tokens.json: `typography.lineHeight.normal` = 1.5
        assert!((LH_NORMAL - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn line_height_relaxed() {
        // tokens.json: `typography.lineHeight.relaxed` = 1.75
        assert!((LH_RELAXED - 1.75).abs() < f32::EPSILON);
    }

    #[test]
    fn tracking_body() {
        // tokens.json: `typography.letterSpacing.body` = "-0.025em"
        assert!((TRACKING_BODY - (-0.025)).abs() < f32::EPSILON);
    }

    #[test]
    fn weight_semibold_is_600() {
        // tokens.json: `typography.fontWeight.semibold` = 600
        assert_eq!(weight::SEMIBOLD, 600);
    }

    #[test]
    fn weight_bold_is_700() {
        // tokens.json: `typography.fontWeight.bold` = 700
        assert_eq!(weight::BOLD, 700);
    }
}
