//! Scrim Entity — palette overlay 배경 backdrop.
//!
//! @MX:ANCHOR: [AUTO] Scrim — fan_in 대상 (3 variant + RootView 통합).
//! @MX:REASON: [AUTO] 테마 감지 backdrop + click-to-dismiss 계약. AC-PL-1, AC-PL-2.
//! @MX:SPEC: SPEC-V3-012

// ============================================================
// 팔레트 z-index 상수 (tokens.json round2_component.palette.scrim.z_index = 20)
// ============================================================

/// Palette Scrim z-index — overlay 스택 최상위.
pub const PALETTE_Z: i32 = 20;

// ============================================================
// Scrim 색상 상수 (tokens.json round2_component.palette.scrim)
// ============================================================

/// 다크 테마 Scrim alpha (0.55).
pub const SCRIM_DARK_ALPHA: f32 = 0.55;
/// 라이트 테마 Scrim alpha (0.18).
pub const SCRIM_LIGHT_ALPHA: f32 = 0.18;

// ============================================================
// Scrim 색상 도우미
// ============================================================

/// 다크 테마 Scrim RGBA 컴포넌트 — rgba(8, 12, 11, 0.55).
///
/// tokens.json: `round2_component.palette.scrim.dark` = "rgba(8,12,11,0.55)"
/// 기반 색상: neutral::N950 = 0x09110f (≈ r=9,g=17,b=15), 명세값 r=8,g=12,b=11 사용.
pub const SCRIM_DARK_R: u8 = 8;
pub const SCRIM_DARK_G: u8 = 12;
pub const SCRIM_DARK_B: u8 = 11;

/// 라이트 테마 Scrim RGBA 컴포넌트 — rgba(20, 30, 28, 0.18).
///
/// tokens.json: `round2_component.palette.scrim.light` = "rgba(20,30,28,0.18)"
/// 기반 색상: brand::INK = 0x09110f (근사), 명세값 r=20,g=30,b=28 사용.
pub const SCRIM_LIGHT_R: u8 = 20;
pub const SCRIM_LIGHT_G: u8 = 30;
pub const SCRIM_LIGHT_B: u8 = 28;

// ============================================================
// Theme 구분 타입
// ============================================================

/// 활성 테마 구분 — palette 코드 내부 테마 분기에 사용.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrimTheme {
    Dark,
    Light,
}

// ============================================================
// ScrimColor — RGBA 컬러 표현
// ============================================================

/// Scrim 배경색 — (r, g, b, alpha) 표현.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrimColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: f32,
}

impl ScrimColor {
    /// 다크 테마용 Scrim 색상 반환 — rgba(8,12,11,0.55).
    pub fn dark() -> Self {
        Self {
            r: SCRIM_DARK_R,
            g: SCRIM_DARK_G,
            b: SCRIM_DARK_B,
            alpha: SCRIM_DARK_ALPHA,
        }
    }

    /// 라이트 테마용 Scrim 색상 반환 — rgba(20,30,28,0.18).
    pub fn light() -> Self {
        Self {
            r: SCRIM_LIGHT_R,
            g: SCRIM_LIGHT_G,
            b: SCRIM_LIGHT_B,
            alpha: SCRIM_LIGHT_ALPHA,
        }
    }

    /// 테마에 따른 Scrim 색상 반환.
    pub fn for_theme(theme: ScrimTheme) -> Self {
        match theme {
            ScrimTheme::Dark => Self::dark(),
            ScrimTheme::Light => Self::light(),
        }
    }
}

// ============================================================
// Scrim 상태 — click-to-dismiss 경계 정보
// ============================================================

/// Scrim 상태 — palette container 범위 및 dismiss 이벤트 추적.
#[derive(Debug, Clone)]
pub struct ScrimState {
    /// 현재 테마.
    pub theme: ScrimTheme,
    /// Palette container 의 뷰포트 내 bounds (x, y, width, height).
    /// Scrim 클릭이 이 bounds 밖에 있으면 dismiss_requested 이벤트를 발생시킨다.
    pub palette_bounds: Option<(f32, f32, f32, f32)>,
    /// dismiss 이벤트 발생 횟수 — 테스트에서 정확히 1회 발생 검증.
    pub dismiss_count: u32,
}

impl ScrimState {
    /// 새 ScrimState 를 생성한다.
    pub fn new(theme: ScrimTheme) -> Self {
        Self {
            theme,
            palette_bounds: None,
            dismiss_count: 0,
        }
    }

    /// 테마에 맞는 Scrim 색상을 반환한다 (AC-PL-1).
    pub fn color(&self) -> ScrimColor {
        ScrimColor::for_theme(self.theme)
    }

    /// 클릭 좌표가 palette container 밖인지 검사한다.
    ///
    /// palette_bounds 가 None 이면 (palette container 없음) 항상 외부로 간주한다.
    pub fn is_outside_palette(&self, click_x: f32, click_y: f32) -> bool {
        match self.palette_bounds {
            None => true,
            Some((px, py, pw, ph)) => {
                click_x < px || click_x > px + pw || click_y < py || click_y > py + ph
            }
        }
    }

    /// Scrim 클릭을 처리한다.
    ///
    /// 클릭 좌표가 palette container 밖이면 dismiss_count 를 증가시키고 true 를 반환 (AC-PL-2).
    /// 클릭 좌표가 palette container 안이면 아무 일도 일어나지 않고 false 를 반환.
    pub fn handle_click(&mut self, click_x: f32, click_y: f32) -> bool {
        if self.is_outside_palette(click_x, click_y) {
            self.dismiss_count += 1;
            true
        } else {
            false
        }
    }
}

// ============================================================
// Scrim — palette backdrop entity (GPUI-independent logic layer)
// ============================================================

/// Scrim — 전체 뷰포트 backdrop entity.
///
/// - z-index 20 으로 렌더 (PALETTE_Z).
/// - 테마 감지 색상 (dark: rgba(8,12,11,0.55) / light: rgba(20,30,28,0.18)).
/// - palette container 외부 클릭 시 dismiss_requested 이벤트.
/// - GPUI backdrop-filter blur 미지원으로 solid-alpha fallback (RG-PL-4).
#[derive(Debug)]
pub struct Scrim {
    state: ScrimState,
}

impl Scrim {
    /// 지정 테마로 새 Scrim 을 생성한다.
    pub fn new(theme: ScrimTheme) -> Self {
        Self {
            state: ScrimState::new(theme),
        }
    }

    /// 현재 테마를 반환한다.
    pub fn theme(&self) -> ScrimTheme {
        self.state.theme
    }

    /// 테마에 맞는 Scrim 색상을 반환한다.
    pub fn color(&self) -> ScrimColor {
        self.state.color()
    }

    /// Palette container 범위를 설정한다.
    pub fn set_palette_bounds(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.state.palette_bounds = Some((x, y, w, h));
    }

    /// Scrim 클릭을 처리한다.
    ///
    /// 반환값: dismiss_requested 이벤트를 발생해야 하면 true.
    pub fn handle_click(&mut self, click_x: f32, click_y: f32) -> bool {
        self.state.handle_click(click_x, click_y)
    }

    /// 누적 dismiss 이벤트 수 (테스트 검증용).
    pub fn dismiss_count(&self) -> u32 {
        self.state.dismiss_count
    }

    /// z-index 값을 반환한다.
    pub fn z_index(&self) -> i32 {
        PALETTE_Z
    }
}

// ============================================================
// 단위 테스트 — AC-PL-1, AC-PL-2
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------
    // AC-PL-1: Scrim render — 테마별 색상 검증
    // ----------------------------------------------------------

    /// AC-PL-1: 다크 테마 Scrim 은 alpha 0.55 의 rgba(8,12,11,*) 를 생성한다.
    #[test]
    fn scrim_renders_dark_alpha_055() {
        let scrim = Scrim::new(ScrimTheme::Dark);
        let color = scrim.color();
        assert_eq!(color.r, 8, "다크 Scrim r 값 불일치");
        assert_eq!(color.g, 12, "다크 Scrim g 값 불일치");
        assert_eq!(color.b, 11, "다크 Scrim b 값 불일치");
        assert!(
            (color.alpha - 0.55).abs() < 1e-6,
            "다크 Scrim alpha != 0.55, got {}",
            color.alpha
        );
    }

    /// AC-PL-1: 라이트 테마 Scrim 은 alpha 0.18 의 rgba(20,30,28,*) 를 생성한다.
    #[test]
    fn scrim_renders_light_alpha_018() {
        let scrim = Scrim::new(ScrimTheme::Light);
        let color = scrim.color();
        assert_eq!(color.r, 20, "라이트 Scrim r 값 불일치");
        assert_eq!(color.g, 30, "라이트 Scrim g 값 불일치");
        assert_eq!(color.b, 28, "라이트 Scrim b 값 불일치");
        assert!(
            (color.alpha - 0.18).abs() < 1e-6,
            "라이트 Scrim alpha != 0.18, got {}",
            color.alpha
        );
    }

    /// Scrim z-index 는 20 이어야 한다.
    #[test]
    fn scrim_z_index_is_20() {
        let scrim = Scrim::new(ScrimTheme::Dark);
        assert_eq!(scrim.z_index(), 20, "Scrim z-index 불일치");
    }

    // ----------------------------------------------------------
    // AC-PL-2: click-to-dismiss 이벤트 경계 검증
    // ----------------------------------------------------------

    /// AC-PL-2: palette container 외부 클릭 시 dismiss_requested 이벤트가 정확히 1회 발생.
    #[test]
    fn click_outside_emits_dismiss() {
        let mut scrim = Scrim::new(ScrimTheme::Dark);
        // palette container: (300, 100, 600, 400) — x=300~900, y=100~500
        scrim.set_palette_bounds(300.0, 100.0, 600.0, 400.0);

        // 외부 클릭 (10, 10)
        let dismissed = scrim.handle_click(10.0, 10.0);
        assert!(dismissed, "외부 클릭 시 dismiss 반환값 불일치");
        assert_eq!(
            scrim.dismiss_count(),
            1,
            "dismiss_count 는 정확히 1 이어야 함"
        );
    }

    /// AC-PL-2: palette container 내부 클릭 시 dismiss 이벤트 미발생.
    #[test]
    fn click_inside_does_not_emit() {
        let mut scrim = Scrim::new(ScrimTheme::Dark);
        // palette container: (300, 100, 600, 400)
        scrim.set_palette_bounds(300.0, 100.0, 600.0, 400.0);

        // 내부 클릭 (600, 300)
        let dismissed = scrim.handle_click(600.0, 300.0);
        assert!(!dismissed, "내부 클릭 시 dismiss 가 발생하면 안 됨");
        assert_eq!(scrim.dismiss_count(), 0, "dismiss_count 는 0 이어야 함");
    }

    /// 다중 외부 클릭 시 누적 dismiss_count 가 증가한다.
    #[test]
    fn multiple_outside_clicks_accumulate() {
        let mut scrim = Scrim::new(ScrimTheme::Light);
        scrim.set_palette_bounds(300.0, 100.0, 600.0, 400.0);

        scrim.handle_click(0.0, 0.0);
        scrim.handle_click(1200.0, 800.0);
        assert_eq!(scrim.dismiss_count(), 2);
    }

    /// palette_bounds 미설정 시 모든 클릭이 외부로 처리된다.
    #[test]
    fn no_bounds_means_all_clicks_outside() {
        let mut scrim = Scrim::new(ScrimTheme::Dark);
        let dismissed = scrim.handle_click(500.0, 300.0);
        assert!(dismissed);
    }

    /// ScrimColor::for_theme 이 각 테마의 올바른 색상을 반환한다.
    #[test]
    fn scrim_color_for_theme() {
        let dark = ScrimColor::for_theme(ScrimTheme::Dark);
        let light = ScrimColor::for_theme(ScrimTheme::Light);
        assert!((dark.alpha - 0.55).abs() < 1e-6);
        assert!((light.alpha - 0.18).abs() < 1e-6);
    }
}
