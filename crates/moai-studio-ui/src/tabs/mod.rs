//! Tabs 모듈 (SPEC-V3-003 MS-2).
//!
//! 공개 API:
//! - [`TabContainer`] — N 개의 Tab 을 소유하고 active_tab_idx 관리
//! - [`Tab`] — 단일 탭 (독립된 `PaneTree` + last_focused_pane: Option<PaneId>)
//! - [`TabId`] — 탭 식별자 (Spike 3 결정 따름)
//! - [`TabCommand`] — MS-2 탭 바인딩 명령 (T9)
//! - [`TabKeyCode`] — MS-2 탭 바인딩 키 코드 (T9)
//! - [`dispatch_tab_key`] — key → TabCommand 매핑 (T9, AC-P-9a/9b/26)

pub mod container;
pub use container::{CloseTabError, SwitchTabError, Tab, TabContainer, TabId};

pub mod bar;

pub mod keys;
pub use keys::{TabCommand, TabKeyCode, dispatch_tab_key};
