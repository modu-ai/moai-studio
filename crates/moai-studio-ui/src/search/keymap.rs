//! Keymap actions for the global search panel.
//!
//! Defines the `ToggleSearchPanel` action consumed by `RootView`'s key
//! dispatch chain. The action is bound to:
//! - macOS: `cmd-shift-f`
//! - Linux / Windows: `ctrl-shift-f`
//!
//! SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2 T7 (REQ-GS-031).

use gpui::actions;

actions!(moai_studio, [ToggleSearchPanel]);
