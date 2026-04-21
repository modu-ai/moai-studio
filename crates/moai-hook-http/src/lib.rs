//! moai-hook-http: Claude Code 플러그인을 위한 HTTP 훅 수신기
//!
//! Claude 가 전송하는 훅 이벤트(PreToolUse, PostToolUse, SessionStart 등)를
//! HTTP POST 로 수신하고 처리합니다.

pub mod auth;
pub mod endpoint;
mod error;
mod server;
mod types;

pub use auth::RotatingAuthToken;
pub use endpoint::HookEndpoint;
pub use error::HookServerError;
pub use server::HookServer;
pub use types::{HookEventRequest, HookResponse, HookSpecificOutput};
