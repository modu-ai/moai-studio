//! Palette variant 모듈 — CmdPalette / CommandPalette / SlashBar.
//!
//! @MX:NOTE: [AUTO] 3 variant re-export. 각 variant 는 PaletteView + mock 데이터 소스 조합.
//! @MX:SPEC: SPEC-V3-012 MS-2

pub mod cmd_palette;
pub mod command_palette;
pub mod slash_bar;

pub use cmd_palette::CmdPalette;
pub use command_palette::CommandPalette;
pub use slash_bar::SlashBar;
