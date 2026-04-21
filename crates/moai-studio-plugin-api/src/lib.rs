//! MoAI Studio Plugin API (스캐폴드).
//!
//! Phase 6 (SPEC-V3-013) 에서 구현 예정:
//! - `Plugin` trait (manifest, activate, deactivate)
//! - Surface / Command / LinkParser / Sidebar / StatusBar 등록 API
//! - Hook event listener
//! - 인프로세스 Rust 정적 플러그인 (cargo feature flag)
//!
//! 번들 플러그인:
//! - `moai-studio-plugin-moai-adk` (번들, 기본 활성)
//! - `moai-studio-plugin-markdown-viewer` (번들, 기본 활성)
//! - `moai-studio-plugin-monaco` (번들, 기본 활성) — tree-sitter 기반 대체 가능
//! - ... (추가)

use tracing::info;

pub fn hello() {
    info!("moai-studio-plugin-api: scaffold (Phase 0.4). Trait 정의 → Phase 6 (SPEC-V3-013).");
}

/// Phase 6 에서 구현될 Plugin trait 의 시그니처 (placeholder)
pub trait Plugin: Send + Sync {
    /// 플러그인 식별자 (예: "moai-adk")
    fn id(&self) -> &'static str;

    /// 사람이 읽을 수 있는 이름
    fn name(&self) -> &'static str;

    /// 플러그인 활성화 (UI 등록)
    fn on_activate(&self) -> Result<(), PluginError>;

    /// 플러그인 비활성화 (cleanup)
    fn on_deactivate(&self) -> Result<(), PluginError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin activation failed: {0}")]
    ActivationFailed(String),

    #[error("plugin deactivation failed: {0}")]
    DeactivationFailed(String),
}
