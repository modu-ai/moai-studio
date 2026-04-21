//! 훅 서버 에러 타입

use thiserror::Error;

/// HookServer 에서 발생할 수 있는 에러
#[derive(Debug, Error)]
pub enum HookServerError {
    /// TCP 소켓 바인딩 실패
    #[error("소켓 바인딩 실패: {0}")]
    BindError(#[from] std::io::Error),
}
