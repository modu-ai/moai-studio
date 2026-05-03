//! Global search module — SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2.
//!
//! Provides GPUI-side [`SearchPanel`] entity and supporting types for
//! multi-workspace file-content search. The underlying search engine lives
//! in the `moai-search` crate (zero GPUI dependency).
//!
//! # Module structure
//!
//! - [`panel`]: [`SearchPanel`] GPUI Entity — input, cancel button, result
//!   list, and status line.
//! - [`result_view`]: 2-line result row rendering + match highlight helpers.
//! - [`keymap`]: `ToggleSearchPanel` action definition for ⌘⇧F / Ctrl+Shift+F.

pub mod keymap;
pub mod navigation;
pub mod panel;
pub mod result_view;

pub use keymap::ToggleSearchPanel;
pub use navigation::{NavigationOutcome, hit_to_open_code_viewer, touch_workspace};
pub use panel::{SearchPanel, SearchStatus};
pub use result_view::{
    extract_highlight_span, extract_preview_segments, format_row_label, on_row_click,
};
