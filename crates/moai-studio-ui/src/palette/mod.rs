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

pub mod palette_view;
pub use palette_view::{
    HIGHLIGHT_ALPHA, LIST_MAX_HEIGHT, PALETTE_WIDTH, PaletteItem, PaletteView, ROW_HEIGHT,
};

pub mod scrim;
pub use scrim::Scrim;

/// Palette 이벤트 — Scrim 과 PaletteView 가 공유하는 이벤트 타입.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteEvent {
    /// 목록 항목 선택 확정 — Enter 키 또는 클릭.
    ItemSelected(PaletteItem),
    /// Palette 닫기 요청 — Esc 키 또는 Scrim 클릭.
    DismissRequested,
}
