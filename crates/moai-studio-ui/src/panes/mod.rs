//! Pane tree 와 split 관리 (SPEC-V3-003 MS-1).
//!
//! 공개 API:
//! - [`PaneTree`] — 이진 트리 pane 자료구조 (Leaf / Split)
//! - [`PaneId`] — pane 식별자 (Spike 3 결정 따름)
//! - [`SplitDirection`] — Horizontal (좌/우) / Vertical (상/하)
//! - [`PaneConstraints`] — 최소 pane 크기 불변 상수 (40 cols × 10 rows)
//!
pub mod constraints;
pub use constraints::PaneConstraints;

pub mod tree;
pub use tree::{Leaf, PaneId, PaneTree, RatioError, SplitDirection, SplitError, SplitNodeId};

pub mod splitter;
pub use splitter::{CloseError, PaneSplitter};

pub mod divider;
pub use divider::{GpuiDivider, ResizableDivider};

pub mod splitter_gpui_native;
pub use splitter_gpui_native::GpuiNativeSplitter;

pub mod focus;
pub use focus::{
    FocusCommand, FocusRouter, KeyCode, KeyModifiers, PLATFORM_MOD, PlatformMod, dispatch_key,
};

pub mod render;
pub use render::{
    count_leaves, count_splits, divider_horizontal, divider_vertical, render_pane_tree,
};
