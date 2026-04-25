//! 탭 바 시각 상태 계산 + design token 노출.
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-5 REQ-P-044 (active 탭: background + bold font-weight 동시 충족)
//! - spec.md §6.3 접근성 (VoiceOver / Orca / tab role / pane title)
//! - contract.md §10.2 AC-P-24 (탭 바 가시성: 레이블 포함 전체 탭 열거)
//! - contract.md §10.2 AC-P-27 (Nm-2 해소: bold active indicator + design token)
//!
//! ## 설계 원칙
//!
//! 순수 로직 레이어. [`TabBar::items`] 가 [`TabBarItem`] 목록을 반환하고,
//! 실제 GPUI 렌더는 이 상태를 소비한다.

// ============================================================
// Design Token 상수 (AC-P-27)
// ============================================================

// @MX:NOTE: [AUTO] toolbar-token-constants
// @MX:SPEC: [AUTO] contract.md §10.2 AC-P-27, spec.md §5 REQ-P-044
// toolbar.tab.active/inactive.background 토큰은 GPUI 테마 시스템과 연동되는
// 문자열 키다. 실제 색상 값은 테마 파일에 위임하며 여기서 하드코딩하지 않는다.
/// 활성 탭 배경색 design token 키 (AC-P-27).
pub const TOOLBAR_TAB_ACTIVE_BG: &str = "toolbar.tab.active.background";

/// 비활성 탭 배경색 design token 키 (AC-P-27).
pub const TOOLBAR_TAB_INACTIVE_BG: &str = "toolbar.tab.inactive.background";

// ============================================================
// FontWeight
// ============================================================

/// 탭 레이블 폰트 굵기.
///
/// GPUI 독립 표현 — 렌더 레이어에서 실제 GPUI 타입으로 변환한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    /// 일반 굵기 (비활성 탭).
    Normal,
    /// 굵게 (활성 탭 — REQ-P-044, AC-P-27).
    Bold,
}

// ============================================================
// TabBarItem
// ============================================================

/// 탭 바의 단일 탭 시각 상태.
///
/// [`TabBar::items`] 가 반환하는 구조체이며, 렌더 레이어가 이 상태를 소비한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabBarItem {
    /// 탭 레이블 (Tab.title 기반).
    pub label: String,
    /// 활성 탭 여부.
    pub is_active: bool,
    /// 배경색 design token 키 (AC-P-27).
    pub background_token: &'static str,
    /// 레이블 폰트 굵기 (AC-P-27 bold active indicator).
    pub font_weight: FontWeight,
}

// ============================================================
// TabBar
// ============================================================

/// 탭 바 시각 상태 계산기.
///
/// `TabContainer` 의 공개 API 만 사용하고, 상태 계산 결과를 [`TabBarItem`] 목록으로 반환한다.
pub struct TabBar;

impl TabBar {
    // ----------------------------------------------------------
    // @MX:ANCHOR: [AUTO] bar-active-indicator
    // @MX:REASON: [AUTO] active/inactive 시각 상태 결정 진입점.
    //   AC-P-27 bold indicator + token 전환이 이 함수에 집중됨.
    //   fan_in >= 3: TabBar 렌더(GPUI view), 테스트 suite, MS-3 snapshot.
    // ----------------------------------------------------------

    /// `labels` 와 `active_idx` 를 받아 탭 바 아이템 목록을 반환한다.
    ///
    /// # Arguments
    ///
    /// * `labels` — 탭 레이블 슬라이스 (TabContainer.tabs 순서 그대로).
    /// * `active_idx` — 현재 활성 탭 인덱스.
    ///
    /// # Returns
    ///
    /// 각 탭에 대해 활성/비활성 시각 상태가 결정된 [`TabBarItem`] 목록.
    pub fn items(labels: &[&str], active_idx: usize) -> Vec<TabBarItem> {
        labels
            .iter()
            .enumerate()
            .map(|(i, &label)| {
                let is_active = i == active_idx;
                TabBarItem {
                    label: label.to_string(),
                    is_active,
                    background_token: if is_active {
                        TOOLBAR_TAB_ACTIVE_BG
                    } else {
                        TOOLBAR_TAB_INACTIVE_BG
                    },
                    font_weight: if is_active {
                        FontWeight::Bold
                    } else {
                        FontWeight::Normal
                    },
                }
            })
            .collect()
    }
}

// ============================================================
// 단위 테스트 (RED phase — contract.md §10.4)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tabs::container::TabContainer;

    // ──────────────────────────────────────────────
    // AC-P-24: 탭 바가 레이블과 함께 전체 탭을 열거한다
    // ──────────────────────────────────────────────

    /// AC-P-24: bar_enumerates_tabs_in_order
    ///
    /// 3개 탭을 가진 컨테이너에서 bar items 수와 순서가 일치해야 한다.
    #[test]
    fn bar_enumerates_tabs_in_order() {
        let labels = ["alpha", "beta", "gamma"];
        let items = TabBar::items(&labels, 0);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].label, "alpha");
        assert_eq!(items[1].label, "beta");
        assert_eq!(items[2].label, "gamma");
    }

    /// AC-P-24 snapshot: TabContainer 공개 API 를 통한 레이블 수집 후 바 열거
    ///
    /// TabContainer.tabs 에서 레이블을 수집해 TabBar::items 와 연동하는 패턴 검증.
    #[test]
    fn bar_shows_all_tabs_with_labels_via_container() {
        let mut container = TabContainer::new(); // 탭 0: "untitled"
        container.new_tab(None); // 탭 1: "untitled"
        container.new_tab(None); // 탭 2: "untitled"

        // TabContainer 공개 API 로 레이블 수집
        let labels: Vec<&str> = container.tabs.iter().map(|t| t.title.as_str()).collect();
        let items = TabBar::items(&labels, container.active_tab_idx);

        assert_eq!(items.len(), 3);
        // 모든 탭이 레이블을 가진다
        for item in &items {
            assert!(!item.label.is_empty());
        }
    }

    // ──────────────────────────────────────────────
    // AC-P-27: active indicator is bold
    // ──────────────────────────────────────────────

    /// AC-P-27: active_indicator_is_bold
    ///
    /// 활성 탭은 Bold, 비활성 탭은 Normal 폰트 굵기를 가져야 한다.
    #[test]
    fn active_indicator_is_bold() {
        let labels = ["first", "second", "third"];
        let items = TabBar::items(&labels, 1); // "second" 가 활성

        assert_eq!(
            items[0].font_weight,
            FontWeight::Normal,
            "비활성 탭은 Normal"
        );
        assert_eq!(items[1].font_weight, FontWeight::Bold, "활성 탭은 Bold");
        assert_eq!(
            items[2].font_weight,
            FontWeight::Normal,
            "비활성 탭은 Normal"
        );
    }

    // ──────────────────────────────────────────────
    // AC-P-27: inactive_uses_toolbar_background_token
    // ──────────────────────────────────────────────

    /// AC-P-27: inactive_uses_toolbar_background_token
    ///
    /// 비활성 탭은 TOOLBAR_TAB_INACTIVE_BG 토큰, 활성 탭은 TOOLBAR_TAB_ACTIVE_BG 토큰을 사용해야 한다.
    #[test]
    fn inactive_uses_toolbar_background_token() {
        let labels = ["a", "b"];
        let items = TabBar::items(&labels, 0); // "a" 가 활성

        assert_eq!(
            items[0].background_token, TOOLBAR_TAB_ACTIVE_BG,
            "활성 탭은 active.background 토큰"
        );
        assert_eq!(
            items[1].background_token, TOOLBAR_TAB_INACTIVE_BG,
            "비활성 탭은 inactive.background 토큰"
        );
    }
}
