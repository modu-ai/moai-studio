//! `ResizableDivider` 추상 trait (REQ-P-062) + drag handler + ratio clamp 로직.
//!
//! 스펙 참조:
//! - spec.md §7.3 ResizableDivider
//! - spec.md §5 RG-P-1 REQ-P-005 (ratio 경계 거부) + RG-P-2 REQ-P-012 (drag clamp)
//!
//! @MX:TODO(T3): `pub trait ResizableDivider` 정의 + 기본 drag 이벤트 시그니처 + `#[cfg(test)] MockDivider`.
//! @MX:TODO(T5): Spike 1 결과 기반 구체 구현. Clamp 로직은 PaneConstraints::{MIN_COLS, MIN_ROWS} 준수.
