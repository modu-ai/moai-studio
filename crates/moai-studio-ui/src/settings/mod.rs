//! Settings 모듈 — SettingsModal + AppearancePane + in-memory 상태.
//!
//! SPEC-V3-013 MS-1 산출:
//! - `settings_modal.rs` — SettingsModal (880×640 컨테이너, sidebar + main pane)
//! - `settings_state.rs` — SettingsViewState (selected_section, AppearanceState)
//! - `panes/appearance.rs` — AppearancePane (4 controls in-memory)
//!
//! ActiveTheme global + UserSettings 영속화는 MS-3.

pub mod panes;
pub mod settings_modal;
pub mod settings_state;

pub use panes::AppearancePane;
pub use settings_modal::SettingsModal;
pub use settings_state::{
    AccentColor, AppearanceState, Density, SettingsSection, SettingsViewState, ThemeMode,
};
