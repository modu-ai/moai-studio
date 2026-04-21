//! moai-claude-host: Claude 서브프로세스 스폰(spawn) 관리
//!
//! T-012: `spawn` 모듈 — 워크스페이스별 claude --bare subprocess spawn
//! T-013: `stdin` / `monitor` 모듈 — stdin 메시지 전송 + crash 감지

mod error;
mod process;

pub mod monitor;
pub mod spawn;
pub mod stdin;

pub use error::ProcessError;
pub use process::{ClaudeProcess, ClaudeProcessConfig, SDKUserMessage};
pub use spawn::{DEFAULT_TOOLS, workspace_config};
