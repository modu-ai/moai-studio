//! `PaneConstraints` 최소 pane 크기 불변 상수 (40 cols × 10 rows).
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-2 REQ-P-010, REQ-P-014
//! - spec.md M-2 (v0.2.0 iter2 해소): associated const 공개 값으로 단일화, runtime 변경 불가
//!
//! @MX:TODO(T2): `pub struct PaneConstraints;` + `impl PaneConstraints { pub const MIN_COLS: u16 = 40; pub const MIN_ROWS: u16 = 10; }`. 검증: AC-P-21 (negative API surface — set_min_cols / set_min_rows / PaneConstraints::new 같은 가변 API 가 공개 API 에 없어야 함).
