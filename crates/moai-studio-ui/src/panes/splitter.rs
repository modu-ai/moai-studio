//! `PaneSplitter` 추상 trait (REQ-P-061) + 구체 구현 (Spike 1 또는 Spike 2 결과).
//!
//! 스펙 참조:
//! - spec.md §7.2 PaneSplitter
//! - spec.md §11.1 C-1 — gpui-component 도입 여부 plan Spike 후 결정
//!
//! @MX:TODO(T3): `pub trait PaneSplitter` 정의 + `#[cfg(test)] MockPaneSplitter` 구현. Doc test 로 abstract_traits_compile_without_impl 검증.
//! @MX:TODO(T4): Spike 1 PASS 시 `GpuiNativeSplitter` 구현, FAIL + Spike 2 PASS 시 `GpuiComponentSplitter` 구현. 사용자 결정 required.
