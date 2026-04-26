//! SPEC-V3-009 MS-1 — SPEC Management UI 모듈.
//!
//! ## 공개 API
//! - [`list_view`] — SpecListView: `.moai/specs/` 스캔 + SPEC 카드 목록 (AC-SU-1)
//! - [`detail_view`] — SpecDetailView: 선택된 SPEC 의 RG/REQ/AC 표 렌더 (AC-SU-2, AC-SU-3)
//!
//! RootView 통합은 MS-3 에서 수행 (N6 — lib.rs 미수정).

pub mod detail_view;
pub mod list_view;

pub use detail_view::SpecDetailView;
pub use list_view::SpecListView;
