use thiserror::Error;

/// Claude 서브프로세스 관련 오류 유형
#[derive(Debug, Error)]
pub enum ProcessError {
    /// ANTHROPIC_API_KEY 환경 변수가 설정되지 않은 경우
    #[error("ANTHROPIC_API_KEY is not set")]
    ApiKeyMissing,

    /// claude CLI 바이너리를 지정된 경로에서 찾을 수 없는 경우
    #[error("claude CLI not found at: {path}")]
    ClaudeNotFound { path: String },

    /// 서브프로세스 스폰(spawn) 자체가 실패한 경우
    #[error("failed to spawn claude subprocess: {source}")]
    SpawnFailed {
        #[source]
        source: std::io::Error,
    },

    /// 서브프로세스가 비정상 종료된 경우
    #[error("claude subprocess crashed with exit code: {exit_code:?}")]
    ProcessCrashed { exit_code: Option<i32> },
}
