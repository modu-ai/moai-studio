//! Tabs 모듈 (SPEC-V3-003 MS-2).
//!
//! 공개 API:
//! - [`TabContainer`] — N 개의 Tab 을 소유하고 active_tab_idx 관리
//! - [`Tab`] — 단일 탭 (독립된 `PaneTree` + last_focused_pane: Option<PaneId>)
//! - [`TabId`] — 탭 식별자 (Spike 3 결정 따름)
//! - [`TabError`] — 탭 조작 에러 (LastTabCloseNoop, IndexOutOfBounds)
//!
//! @MX:TODO(T10): 탭 바 UI + toolbar.tab.active.background design token + bold active indicator.
//! 관련 REQ: REQ-P-040 ~ REQ-P-045 (RG-P-5) + REQ-P-030 ~ REQ-P-034 (RG-P-4 MS-2 부분).
//! 관련 AC: AC-P-8, AC-P-9a/9b, AC-P-10, AC-P-11, AC-P-19, AC-P-24, AC-P-25, AC-P-26, AC-P-27.

pub mod container;
pub use container::{Tab, TabContainer, TabError, TabId};
