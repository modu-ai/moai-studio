//! Settings panes — SettingsModal 의 각 section pane 모듈.

pub mod advanced;
pub mod agent;
pub mod appearance;
pub mod editor;
pub mod keyboard;
pub mod terminal;

pub use advanced::AdvancedPane;
pub use agent::AgentPane;
pub use appearance::AppearancePane;
pub use editor::EditorPane;
pub use keyboard::KeyboardPane;
pub use terminal::TerminalPane;
