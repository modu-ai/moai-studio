//! Focus routing (prev/next pane 단축키 + mouse click → GPUI FocusHandle 이관).
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-3 (REQ-P-020 ~ REQ-P-024)
//! - spec.md §5 RG-P-4 (REQ-P-030 ~ REQ-P-034) — platform_mod 매크로 (macOS: Cmd / Linux: Ctrl)
//!
//! @MX:TODO(T6): `FocusRouter` 구조 + in-order traversal + `platform_mod!` 매크로 + single_focus_invariant 보장 로직. 검증: AC-P-7, AC-P-22, AC-P-23, AC-P-9a/9b MS-1 부분.
