//! `TabContainer` 구현 + 탭 생성/전환/닫기 로직 + last_focused_pane 복원.
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-5 (REQ-P-040 ~ REQ-P-045)
//! - spec.md §5 RG-P-3 REQ-P-023 (탭 전환 시 last_focused_pane 복원)
//! - spec.md §5 RG-P-4 REQ-P-034 (tmux 중첩 시 OS/GPUI 레벨 우선 — AC-P-26)
//!
//! @MX:TODO(T8): `TabContainer { tabs: Vec<Tab>, active_tab_idx: usize }` + new_tab / switch_tab / close_tab / get_active_pane_tree 구현.
//! @MX:TODO(T9): 키 바인딩 dispatcher (platform_mod 재사용) + tests/integration_tmux_nested.rs 통합 테스트.
