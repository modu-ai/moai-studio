//! 탭 바 UI 구성요소 — 활성/비활성 탭 시각 구분 (design token 기반).
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-5 REQ-P-044 (active 탭: background + bold font-weight 동시 충족)
//! - spec.md §6.3 접근성 (VoiceOver / Orca / tab role / pane title)
//! - acceptance.md AC-P-27 (v1.0.0 Nm-2 해소)
//! - acceptance.md AC-P-24 (탭 바 가시 — T10 library 수준 제공으로 완전 달성)
//!
//! ## 설계 결정 (USER-DECISION: design-token-color-value)
//!
//! 활성 탭 배경 = `BG_SURFACE_3` (0x232327) 확정 (2026-04-24).
//! sidebar active workspace row 와 일관된 색상. 신규 색상 토큰 생성 없음.
//! 토큰 alias: `TOOLBAR_TAB_ACTIVE_BG` → lib.rs tokens 모듈.

use crate::tokens;

// ============================================================
// FontWeight
// ============================================================

/// 탭 라벨 폰트 굵기.
///
/// AC-P-27: 활성 탭은 `Bold`, 비활성 탭은 `Normal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    /// 일반 굵기 (400) — 비활성 탭.
    Normal,
    /// 중간 굵기 (500) — 향후 확장용.
    Medium,
    /// 굵게 (700) — 활성 탭 (AC-P-27 직접 근거).
    // @MX:NOTE: [AUTO] bold-active-indicator
    // AC-P-27: 활성 탭은 bold font-weight 로 시각 구분. style_for().active_font_weight 에서 반환.
    Bold,
}

// ============================================================
// TabBarStyle
// ============================================================

// @MX:ANCHOR: [AUTO] tab-bar-style-contract
// @MX:REASON: [AUTO] 탭 바 스타일 계약의 단일 진입점.
//   fan_in >= 2: T10 style_for 자체 + T13 persistence restore 탭 목록 복원 시 스타일 재계산.
//   AC-P-27 direct: active_bg == BG_SURFACE_3 + active_font_weight == Bold.
/// 탭 바 스타일 토큰 묶음 — active / inactive 두 상태 완전 정의.
///
/// ## TOKEN
///
/// | 필드 | 토큰 | 값 |
/// |------|------|-----|
/// | `active_bg` | `TOOLBAR_TAB_ACTIVE_BG` (= `BG_SURFACE_3`) | 0x232327 |
/// | `inactive_bg` | `BG_SURFACE` | 0x131315 |
/// | `active_fg` | `FG_PRIMARY` | 0xf4f4f5 |
/// | `inactive_fg` | `FG_SECONDARY` | 0xb5b5bb |
/// | `active_font_weight` | — | `Bold` |
/// | `inactive_font_weight` | — | `Normal` |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabBarStyle {
    /// 활성 탭 배경 색 (= `TOOLBAR_TAB_ACTIVE_BG` = `BG_SURFACE_3`).
    pub active_bg: u32,
    /// 비활성 탭 배경 색 (= `BG_SURFACE`).
    pub inactive_bg: u32,
    /// 활성 탭 전경 색 (= `FG_PRIMARY`).
    pub active_fg: u32,
    /// 비활성 탭 전경 색 (= `FG_SECONDARY`).
    pub inactive_fg: u32,
    /// 활성 탭 폰트 굵기 (= `Bold`, AC-P-27 직접).
    pub active_font_weight: FontWeight,
    /// 비활성 탭 폰트 굵기 (= `Normal`).
    pub inactive_font_weight: FontWeight,
}

// ============================================================
// TabBar
// ============================================================

/// 탭 바 렌더링 도우미 — pure-Rust 상태 계산 + 선택적 GPUI 래퍼.
///
/// ## 사용법
///
/// ```rust
/// use moai_studio_ui::tabs::bar::{TabBar, FontWeight};
///
/// // idx=1 이 active 인 상황에서 idx=1 의 스타일
/// let style = TabBar::<String>::style_for(1, 1);
/// assert_eq!(style.active_font_weight, FontWeight::Bold);
///
/// // idx=0 은 비활성
/// let style2 = TabBar::<String>::style_for(0, 1);
/// assert_eq!(style2.inactive_font_weight, FontWeight::Normal);
/// ```
///
/// ## 제네릭 파라미터
///
/// - `L: Clone + 'static`: `TabContainer<L>` 의 leaf payload 타입과 일치해야 한다.
pub struct TabBar<L: Clone + 'static> {
    _phantom: std::marker::PhantomData<L>,
}

impl<L: Clone + 'static> TabBar<L> {
    /// `idx` 탭이 `active_idx` 인지 여부를 반환한다.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use moai_studio_ui::tabs::bar::TabBar;
    ///
    /// assert!(TabBar::<String>::is_active(2, 2));
    /// assert!(!TabBar::<String>::is_active(0, 2));
    /// ```
    pub fn is_active(idx: usize, active_idx: usize) -> bool {
        idx == active_idx
    }

    /// `idx` 탭의 [`TabBarStyle`] 을 반환한다.
    ///
    /// `active_idx` 와 무관하게 전체 스타일 팔레트를 반환한다.
    /// 렌더러는 `is_active(idx, active_idx)` 결과에 따라 `active_*` / `inactive_*` 필드를 선택한다.
    ///
    /// - active 상태 → `active_bg`, `active_fg`, `active_font_weight` 사용
    /// - inactive 상태 → `inactive_bg`, `inactive_fg`, `inactive_font_weight` 사용
    ///
    /// # Examples
    ///
    /// ```rust
    /// use moai_studio_ui::tabs::bar::{TabBar, FontWeight};
    ///
    /// let s = TabBar::<String>::style_for(0, 0);
    /// assert_eq!(s.active_font_weight, FontWeight::Bold);
    ///
    /// let s2 = TabBar::<String>::style_for(1, 0);
    /// assert_eq!(s2.inactive_font_weight, FontWeight::Normal);
    /// ```
    pub fn style_for(_idx: usize, _active_idx: usize) -> TabBarStyle {
        // 전체 팔레트는 항상 동일하다. 렌더러가 is_active 분기를 담당.
        TabBarStyle {
            active_bg: tokens::TOOLBAR_TAB_ACTIVE_BG,
            inactive_bg: tokens::BG_SURFACE,
            active_fg: tokens::FG_PRIMARY,
            inactive_fg: tokens::FG_SECONDARY,
            active_font_weight: FontWeight::Bold,
            inactive_font_weight: FontWeight::Normal,
        }
    }
}

// ============================================================
// 단위 테스트 (RED Phase)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens;

    // -------------------------------------------------------
    // Test 1: active_tab_uses_bg_surface_3 (AC-P-27)
    // -------------------------------------------------------

    /// 활성 탭의 배경은 BG_SURFACE_3 (0x232327) 이어야 한다.
    #[test]
    fn active_tab_uses_bg_surface_3() {
        let style = TabBar::<String>::style_for(2, 2);
        assert_eq!(
            style.active_bg,
            tokens::BG_SURFACE_3,
            "활성 탭 bg == BG_SURFACE_3 (0x{:06x})",
            tokens::BG_SURFACE_3
        );
    }

    // -------------------------------------------------------
    // Test 2: inactive_tab_uses_bg_surface (AC-P-27)
    // -------------------------------------------------------

    /// 비활성 탭의 배경은 BG_SURFACE 이어야 한다.
    #[test]
    fn inactive_tab_uses_bg_surface() {
        let style = TabBar::<String>::style_for(0, 2);
        assert_eq!(
            style.inactive_bg,
            tokens::BG_SURFACE,
            "비활성 탭 bg == BG_SURFACE (0x{:06x})",
            tokens::BG_SURFACE
        );
    }

    // -------------------------------------------------------
    // Test 3: active_tab_is_bold (AC-P-27)
    // -------------------------------------------------------

    /// 활성 탭은 bold font-weight 를 가져야 한다 (AC-P-27 직접).
    #[test]
    fn active_tab_is_bold() {
        let style = TabBar::<String>::style_for(1, 1);
        assert_eq!(
            style.active_font_weight,
            FontWeight::Bold,
            "활성 탭 font_weight == Bold"
        );
    }

    // -------------------------------------------------------
    // Test 4: inactive_tab_is_not_bold (AC-P-27 negative)
    // -------------------------------------------------------

    /// 비활성 탭은 bold 가 아니어야 한다.
    #[test]
    fn inactive_tab_is_not_bold() {
        let style = TabBar::<String>::style_for(0, 1);
        assert_ne!(
            style.inactive_font_weight,
            FontWeight::Bold,
            "비활성 탭 font_weight != Bold"
        );
    }

    // -------------------------------------------------------
    // Test 5: active_tab_fg_is_fg_primary (AC-P-27)
    // -------------------------------------------------------

    /// 활성 탭 전경색은 FG_PRIMARY 이어야 한다.
    #[test]
    fn active_tab_fg_is_fg_primary() {
        let style = TabBar::<String>::style_for(0, 0);
        assert_eq!(
            style.active_fg,
            tokens::FG_PRIMARY,
            "활성 탭 fg == FG_PRIMARY (0x{:06x})",
            tokens::FG_PRIMARY
        );
    }

    // -------------------------------------------------------
    // Test 6: is_active_returns_true_for_active_idx
    // -------------------------------------------------------

    /// is_active(idx, active_idx) 가 idx == active_idx 일 때 true 반환.
    #[test]
    fn is_active_returns_true_for_active_idx() {
        assert!(
            TabBar::<String>::is_active(3, 3),
            "idx == active_idx → true"
        );
        assert!(TabBar::<String>::is_active(0, 0), "idx=0, active=0 → true");
    }

    // -------------------------------------------------------
    // Test 7: is_active_returns_false_for_other_idx
    // -------------------------------------------------------

    /// is_active(idx, active_idx) 가 idx != active_idx 일 때 false 반환.
    #[test]
    fn is_active_returns_false_for_other_idx() {
        assert!(
            !TabBar::<String>::is_active(1, 3),
            "idx=1, active=3 → false"
        );
        assert!(
            !TabBar::<String>::is_active(0, 2),
            "idx=0, active=2 → false"
        );
    }

    // -------------------------------------------------------
    // Test 8: toolbar_tab_active_background_alias_matches_bg_surface_3 (lib.rs const 검증)
    // -------------------------------------------------------

    /// lib.rs 의 TOOLBAR_TAB_ACTIVE_BG 가 BG_SURFACE_3 와 같아야 한다.
    #[test]
    fn toolbar_tab_active_background_alias_matches_bg_surface_3() {
        assert_eq!(
            tokens::TOOLBAR_TAB_ACTIVE_BG,
            tokens::BG_SURFACE_3,
            "TOOLBAR_TAB_ACTIVE_BG == BG_SURFACE_3 (design token alias 일관성)"
        );
    }
}
