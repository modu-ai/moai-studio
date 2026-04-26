//! Settings 모듈 — SettingsModal + 6 panes + in-memory 상태.
//!
//! SPEC-V3-013 MS-1 산출:
//! - `settings_modal.rs` — SettingsModal (880×640 컨테이너, sidebar + main pane)
//! - `settings_state.rs` — SettingsViewState (selected_section, AppearanceState)
//! - `panes/appearance.rs` — AppearancePane (4 controls in-memory)
//!
//! SPEC-V3-013 MS-2 산출:
//! - `panes/keyboard.rs` — KeyboardPane (binding 테이블 + edit dialog + conflict_check)
//! - `panes/editor.rs` — EditorPane (skeleton + tab_size)
//! - `panes/terminal.rs` — TerminalPane (skeleton + scrollback_lines)
//! - `panes/agent.rs` — AgentPane (skeleton + auto_approve)
//! - `panes/advanced.rs` — AdvancedPane (skeleton + experimental_flags)
//!
//! SPEC-V3-013 MS-3 산출:
//! - `user_settings.rs` — UserSettings struct + serde + atomic write + fail-soft load
//! - `design/runtime.rs` — ActiveTheme global dispatch wrapper

pub mod panes;
pub mod settings_modal;
pub mod settings_state;
// SPEC-V3-013 MS-3: UserSettings 영속화 + ActiveTheme
pub mod user_settings;

pub use panes::{AdvancedPane, AgentPane, AppearancePane, EditorPane, KeyboardPane, TerminalPane};
pub use settings_modal::SettingsModal;
pub use settings_state::{
    AccentColor, AdvancedState, AgentState, AppearanceState, Density, EditDialogState, EditorState,
    KeyBinding, KeyboardState, SettingsSection, SettingsViewState, TerminalState, ThemeMode,
    default_key_bindings,
};
pub use user_settings::{UserSettings, load_or_default, save_atomic, settings_path};
