//! T-012: Claude 서브프로세스 spawn 공개 API.
//!
//! `process.rs`의 실제 구현을 노출하며, 워크스페이스별 독립 claude --bare
//! 서브프로세스 스폰을 담당한다. 인자 세트는 SPEC-M1-001 RG-M1-5 기준이다.
//!
//! @MX:ANCHOR [AUTO] 워크스페이스별 Claude subprocess 스폰 단일 진입점
//!   fan_in>=2 (supervisor/lifecycle + ffi/bridge_basic)
//! @MX:WARN [AUTO] 서브프로세스 수명·kill 책임은 호출자(Supervisor)가 진다.
//! @MX:REASON [AUTO] orphan subprocess 방지를 위해 상위 레이어의 수명 관리 필수.

pub use crate::process::{ClaudeProcess, ClaudeProcessConfig};

/// 기본 `--tools` 허용 목록 — SPEC-M1-001 §RG-M1-5 AC-18.
///
/// MCP 네임스페이스(`mcp__moai__*`)까지 허용하여 moai-ide-server와
/// 호환되도록 한다.
pub const DEFAULT_TOOLS: &[&str] = &[
    "Read",
    "Edit",
    "Write",
    "Bash",
    "Glob",
    "Grep",
    "mcp__moai__*",
];

/// SPEC-M1-001 §RG-M1-5 기준 워크스페이스 설정 빌더.
///
/// `settings_path`, `mcp_config_path`, `working_dir`만 지정하면
/// 나머지 필수 플래그는 기본값(AC-18)으로 채워진다.
pub fn workspace_config(
    claude_path: impl Into<std::path::PathBuf>,
    api_key: impl Into<String>,
    working_dir: impl Into<std::path::PathBuf>,
    settings_path: Option<std::path::PathBuf>,
    mcp_config_path: Option<std::path::PathBuf>,
) -> ClaudeProcessConfig {
    ClaudeProcessConfig {
        claude_path: claude_path.into(),
        api_key: api_key.into(),
        settings_path,
        mcp_config_path,
        plugin_dir: None,
        tools: DEFAULT_TOOLS.iter().map(|s| (*s).to_string()).collect(),
        permission_mode: "acceptEdits".to_string(),
        working_dir: working_dir.into(),
    }
}
