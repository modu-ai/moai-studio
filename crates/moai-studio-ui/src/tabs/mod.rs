//! Tabs 모듈 (SPEC-V3-003 MS-2).
//!
//! 공개 API:
//! - [`TabContainer`] — N 개의 Tab 을 소유하고 active_tab_idx 관리
//! - [`Tab`] — 단일 탭 (독립된 `PaneTree` + last_focused_pane: Option<PaneId>)
//! - [`TabId`] — 탭 식별자 (Spike 3 결정 따름)
//!
//! @MX:TODO(T10): TabBar 렌더 + design token 구현.
//! @MX:TODO(T9): 키 바인딩 dispatcher (Cmd/Ctrl+T 등) + integration_tmux_nested.rs.

pub mod container;
pub use container::{CloseTabError, SwitchTabError, Tab, TabContainer, TabId};

pub mod bar;
