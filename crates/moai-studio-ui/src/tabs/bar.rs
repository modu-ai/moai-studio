//! 탭 바 GPUI element + active 탭 시각 구분 (design token 기반).
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-5 REQ-P-044 (active 탭: background + bold font-weight 동시 충족)
//! - spec.md §6.3 접근성 (VoiceOver / Orca / tab role / pane title)
//! - acceptance.md AC-P-27 (v1.0.0 Nm-2 해소)
//!
//! @MX:TODO(T10): TabBar 렌더 + `toolbar.tab.active.background` 디자인 토큰 참조 (.moai/design/v3/system.md Toolbar 섹션에 추가 필요). bold active indicator + color 동시 적용. 비활성 탭은 둘 다 미적용.
//! @MX:TODO [USER-DECISION-REQUIRED: design-token-color-value] — 정확한 색상 값은 T10 RED phase 직전 AskUserQuestion.
