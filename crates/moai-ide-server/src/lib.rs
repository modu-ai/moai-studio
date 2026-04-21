//! moai-ide-server: rmcp + axum 기반 MCP 서버
//!
//! Claude ↔ MoAI Studio 통신의 주요 통합 경로.
//! Streamable HTTP 전송을 통해 `--mcp-config` 로 Claude 에 노출된다.

pub mod auth;
pub mod config;
pub mod instance;
pub mod server;

pub use instance::WorkspaceInstance;
