//! Banner variants 모듈 — SPEC-V3-014 MS-2.
//!
//! @MX:ANCHOR: [AUTO] variants-module-public-api
//! @MX:REASON: [AUTO] SPEC-V3-014 RG-V14-5. 5 variant factory 진입점.
//!   fan_in >= 3: BannerStack::push_crash/push_update/push_lsp/push_pty/push_workspace (MS-3),
//!   통합 테스트, banner_stack.rs 확장.
//!
//! 5 variant:
//! - [`crash`] — CrashBanner (Critical, manual dismiss)
//! - [`update`] — UpdateBanner (Info, auto-dismiss 8s)
//! - [`lsp`] — LspBanner (Warning, manual dismiss)
//! - [`pty`] — PtyBanner (Error, manual dismiss)
//! - [`workspace`] — WorkspaceBanner (Warning, manual dismiss)

pub mod crash;
pub mod lsp;
pub mod pty;
pub mod update;
pub mod workspace;

pub use crash::CrashBanner;
pub use lsp::LspBanner;
pub use pty::PtyBanner;
pub use update::UpdateBanner;
pub use workspace::WorkspaceBanner;
