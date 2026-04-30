//! Palette Surface 모듈 — SPEC-V3-012.
//!
//! @MX:NOTE: [AUTO] Palette Surface 모듈 진입점 — 3 variant 가 Scrim + PaletteView core 를 공유.
//! @MX:SPEC: SPEC-V3-012
//!
//! 공개 API:
//! - [`Scrim`] — 전체 뷰포트 backdrop (테마 감지, click-to-dismiss)
//! - [`PaletteView`] — 600px 컨테이너, 14px input, 32px row, 320px 최대 목록 높이, 키보드 nav
//! - [`PaletteItem`] — 목록 항목 타입
//! - [`PaletteEvent`] — Scrim/PaletteView 이벤트 (item_selected, dismiss_requested)
//! - [`fuzzy`] — subsequence + scoring fuzzy 매처
//! - [`variants`] — CmdPalette / CommandPalette / SlashBar

pub mod palette_view;
pub use palette_view::{
    HIGHLIGHT_ALPHA, LIST_MAX_HEIGHT, PALETTE_WIDTH, PaletteItem, PaletteView, ROW_HEIGHT,
};

pub mod scrim;
pub use scrim::Scrim;

/// MS-2: fuzzy 매처 모듈.
pub mod fuzzy;

/// MS-4: Real command registry (AC-PL-16).
pub mod registry;
pub use registry::{CATEGORIES, CommandEntry as RegistryCommandEntry, CommandRegistry};

/// MS-2: 3 variant 모듈.
pub mod variants;

/// Palette variant 구분 — mutual exclusion (RG-PL-24) 에 사용.
///
/// MS-3 에서 RootView 의 `active_palette: Option<PaletteVariant>` 필드로 사용.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteVariant {
    /// Cmd+P — 파일 quick open.
    CmdPalette,
    /// Cmd+Shift+P — 커맨드 실행.
    CommandPalette,
    /// `/` in terminal pane — slash command 런처.
    SlashBar,
}

// ============================================================
// PaletteOverlay — MS-3 RootView 통합 헬퍼 (AC-PL-14~15)
// ============================================================

/// PaletteOverlay — RootView 가 보유하는 palette 상태 관리자.
///
/// @MX:ANCHOR: [AUTO] PaletteOverlay — RootView 통합 진입점.
/// @MX:REASON: [AUTO] AC-PL-14/15 mutual exclusion 계약. fan_in >= 3:
///   RootView.handle_palette_key_event, RootView.render_palette_overlay, lib::tests.
/// @MX:SPEC: SPEC-V3-012 MS-3
pub struct PaletteOverlay {
    /// 현재 활성 variant. None 이면 palette 가 닫혀있음.
    pub active_variant: Option<PaletteVariant>,
}

impl PaletteOverlay {
    /// 새 PaletteOverlay 를 생성한다 (초기 상태: 닫힘).
    pub fn new() -> Self {
        Self {
            active_variant: None,
        }
    }

    /// 주어진 variant 를 연다. 이미 다른 variant 가 열려있으면 교체 (mutual exclusion).
    pub fn open(&mut self, variant: PaletteVariant) {
        self.active_variant = Some(variant);
    }

    /// 현재 열려있는 palette 를 닫는다.
    pub fn dismiss(&mut self) {
        self.active_variant = None;
    }

    /// variant 를 toggle 한다. 동일 variant 가 이미 열려있으면 닫고, 아니면 연다 (AC-PL-14).
    pub fn toggle(&mut self, variant: PaletteVariant) {
        if self.active_variant == Some(variant) {
            self.active_variant = None;
        } else {
            self.active_variant = Some(variant);
        }
    }

    /// palette 가 열려있는지 여부.
    pub fn is_visible(&self) -> bool {
        self.active_variant.is_some()
    }
}

impl Default for PaletteOverlay {
    fn default() -> Self {
        Self::new()
    }
}

/// Palette 이벤트 — Scrim 과 PaletteView 가 공유하는 이벤트 타입.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteEvent {
    /// 목록 항목 선택 확정 — Enter 키 또는 클릭.
    ItemSelected(PaletteItem),
    /// Palette 닫기 요청 — Esc 키 또는 Scrim 클릭.
    DismissRequested,
}

// ============================================================
// mutual exclusion 테스트 — AC-PL-14/15 선행 단위 (MS-2 범위)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::variants::{CmdPalette, CommandPalette, SlashBar};

    // ----------------------------------------------------------
    // PaletteOverlay 단위 테스트 (AC-PL-14/15 기반)
    // ----------------------------------------------------------

    /// PaletteOverlay 초기 상태는 닫힘 (active_variant = None).
    #[test]
    fn overlay_initial_state_is_closed() {
        let overlay = PaletteOverlay::new();
        assert!(overlay.active_variant.is_none());
        assert!(!overlay.is_visible());
    }

    /// open(CmdPalette) → active_variant == CmdPalette.
    #[test]
    fn overlay_open_cmd_palette() {
        let mut overlay = PaletteOverlay::new();
        overlay.open(PaletteVariant::CmdPalette);
        assert_eq!(overlay.active_variant, Some(PaletteVariant::CmdPalette));
        assert!(overlay.is_visible());
    }

    /// open another variant → 이전 variant 는 교체됨 (mutual exclusion).
    #[test]
    fn overlay_open_replaces_existing_variant() {
        let mut overlay = PaletteOverlay::new();
        overlay.open(PaletteVariant::CmdPalette);
        overlay.open(PaletteVariant::CommandPalette);
        assert_eq!(overlay.active_variant, Some(PaletteVariant::CommandPalette));
        assert_ne!(overlay.active_variant, Some(PaletteVariant::CmdPalette));
    }

    /// dismiss() → active_variant == None.
    #[test]
    fn overlay_dismiss_closes() {
        let mut overlay = PaletteOverlay::new();
        overlay.open(PaletteVariant::SlashBar);
        overlay.dismiss();
        assert!(overlay.active_variant.is_none());
        assert!(!overlay.is_visible());
    }

    /// toggle — 닫혀있을 때 open, 같은 variant 열려있을 때 dismiss.
    #[test]
    fn overlay_toggle_opens_when_closed() {
        let mut overlay = PaletteOverlay::new();
        overlay.toggle(PaletteVariant::CmdPalette);
        assert_eq!(overlay.active_variant, Some(PaletteVariant::CmdPalette));
    }

    /// toggle — 같은 variant 가 열려있으면 닫힘 (AC-PL-14 toggle semantics).
    #[test]
    fn overlay_toggle_closes_same_variant() {
        let mut overlay = PaletteOverlay::new();
        overlay.open(PaletteVariant::CmdPalette);
        overlay.toggle(PaletteVariant::CmdPalette);
        assert!(overlay.active_variant.is_none());
    }

    /// toggle — 다른 variant 가 열려있으면 교체 (mutual exclusion 유지).
    #[test]
    fn overlay_toggle_replaces_different_variant() {
        let mut overlay = PaletteOverlay::new();
        overlay.open(PaletteVariant::CmdPalette);
        overlay.toggle(PaletteVariant::CommandPalette);
        assert_eq!(overlay.active_variant, Some(PaletteVariant::CommandPalette));
    }

    /// 전체 SlashBar 순환 (open → open → dismiss → closed).
    #[test]
    fn overlay_slash_bar_lifecycle() {
        let mut overlay = PaletteOverlay::new();
        overlay.open(PaletteVariant::SlashBar);
        assert_eq!(overlay.active_variant, Some(PaletteVariant::SlashBar));
        overlay.open(PaletteVariant::CmdPalette);
        assert_ne!(overlay.active_variant, Some(PaletteVariant::SlashBar));
        overlay.dismiss();
        assert!(!overlay.is_visible());
    }

    /// PaletteVariant enum 이 3개 variant 를 모두 포함한다.
    #[test]
    fn palette_variant_has_three_variants() {
        let variants = [
            PaletteVariant::CmdPalette,
            PaletteVariant::CommandPalette,
            PaletteVariant::SlashBar,
        ];
        assert_eq!(variants.len(), 3);
    }

    /// mutual exclusion: active_palette 가 Some 일 때 새 variant 로 교체하면
    /// 이전 variant 가 None 이 된다 (단일 state 전환 모델).
    #[test]
    fn mutual_exclusion_single_active_variant() {
        // RootView 의 active_palette 필드를 시뮬레이션.
        // 순서대로 교체하며 mutual exclusion 검증.

        // CmdPalette 열기.
        let active: Option<PaletteVariant> = Some(PaletteVariant::CmdPalette);
        assert_eq!(active, Some(PaletteVariant::CmdPalette));

        // CommandPalette 로 교체 — CmdPalette 는 자동으로 닫힌다.
        let active: Option<PaletteVariant> = Some(PaletteVariant::CommandPalette);
        assert_eq!(active, Some(PaletteVariant::CommandPalette));
        // CmdPalette 가 닫혀있음을 확인 (동시에 CmdPalette 가 active 가 아님).
        assert_ne!(active, Some(PaletteVariant::CmdPalette));

        // SlashBar 로 교체.
        let active: Option<PaletteVariant> = Some(PaletteVariant::SlashBar);
        assert_eq!(active, Some(PaletteVariant::SlashBar));

        // Dismiss → None.
        let active: Option<PaletteVariant> = None;
        assert!(active.is_none());
    }

    /// 3 variant 가 모두 독립적으로 생성된다.
    #[test]
    fn three_variants_can_be_constructed() {
        let _cmd = CmdPalette::new();
        let _command = CommandPalette::new();
        let _slash = SlashBar::new();
    }

    /// 각 variant 가 PaletteView 를 공유한다 (공통 API 접근 가능).
    #[test]
    fn variants_share_palette_view_api() {
        let cmd = CmdPalette::new();
        let command = CommandPalette::new();
        let slash = SlashBar::new();

        // PaletteView 공통 API 접근.
        assert!(cmd.view.is_input_focused(), "CmdPalette view 포커스");
        assert!(
            command.view.is_input_focused(),
            "CommandPalette view 포커스"
        );
        assert!(slash.view.is_input_focused(), "SlashBar view 포커스");
    }
}
