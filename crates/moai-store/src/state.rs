//! 워크스페이스 6-state 상태 머신 (SPEC-M1-001 RG-M1-4).
//!
//! 상태 전이 다이어그램:
//! ```text
//! Created -> Starting -> Running <-> Paused
//!    |          |          |           |
//!    |          +------> Error <-------+
//!    |                     |
//!    +----> Deleted <------+  (모든 상태에서 Deleted 로 전이 가능)
//! ```

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// 워크스페이스 6-state 상태 머신.
// @MX:ANCHOR: [AUTO] 워크스페이스 생명주기 단일 소스 (fan_in>=4: store/supervisor/ffi/ui)
// @MX:REASON: [AUTO] 상태 전이 규칙은 SPEC-M1-001 의 계약이며, 상위 계층이 이 enum 에 의존한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WorkspaceStatus {
    /// DB 레코드만 존재, 아직 리소스 할당 전
    Created,
    /// git worktree / fs watcher / claude-host 부팅 중
    Starting,
    /// 정상 동작 중
    Running,
    /// 사용자가 일시 정지함 (lazy restart 대상)
    Paused,
    /// 비정상 종료 또는 초기화 실패
    Error,
    /// 삭제됨 (tombstone — 실제 row 는 이 직후 물리 삭제)
    Deleted,
}

/// 상태 전이 오류.
#[derive(Debug, Error, PartialEq)]
#[error("허용되지 않는 상태 전이: {from:?} -> {to:?}")]
pub struct InvalidTransition {
    /// 현재 상태
    pub from: WorkspaceStatus,
    /// 시도한 목표 상태
    pub to: WorkspaceStatus,
}

impl WorkspaceStatus {
    /// 주어진 상태로의 전이가 허용되는지 확인한다.
    // @MX:NOTE: [AUTO] 허용 전이 집합은 SPEC-M1-001 §2 lifecycle diagram 과 일치해야 한다.
    pub fn can_transition_to(self, to: WorkspaceStatus) -> bool {
        use WorkspaceStatus::*;
        // Deleted 는 terminal — 어떤 전이도 허용하지 않는다.
        if matches!(self, Deleted) {
            return false;
        }
        match (self, to) {
            // Deleted 로는 (Deleted 자신을 제외한) 어떤 활성 상태에서도 갈 수 있다.
            (_, Deleted) => true,
            // 초기 상태에서
            (Created, Starting) => true,
            (Created, Error) => true,
            // 부팅 흐름
            (Starting, Running) => true,
            (Starting, Error) => true,
            // 정상 동작 중
            (Running, Paused) => true,
            (Running, Error) => true,
            // 일시 정지에서 복귀 (lazy restart)
            (Paused, Starting) => true,
            (Paused, Error) => true,
            // 에러 복구 — 사용자가 재시작 버튼을 누른 경우
            (Error, Starting) => true,
            // Deleted 는 terminal — 자기 자신으로의 no-op 도 허용하지 않음
            _ => false,
        }
    }

    /// 전이를 시도하고, 허용되지 않으면 오류를 반환한다.
    pub fn transition(self, to: WorkspaceStatus) -> Result<WorkspaceStatus, InvalidTransition> {
        if self.can_transition_to(to) {
            Ok(to)
        } else {
            Err(InvalidTransition { from: self, to })
        }
    }

    /// DB 저장용 문자열 표현.
    pub fn as_str(self) -> &'static str {
        match self {
            WorkspaceStatus::Created => "Created",
            WorkspaceStatus::Starting => "Starting",
            WorkspaceStatus::Running => "Running",
            WorkspaceStatus::Paused => "Paused",
            WorkspaceStatus::Error => "Error",
            WorkspaceStatus::Deleted => "Deleted",
        }
    }
}

impl fmt::Display for WorkspaceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 문자열 파싱 오류.
#[derive(Debug, Error, PartialEq)]
#[error("알 수 없는 워크스페이스 상태 문자열: {0}")]
pub struct ParseStatusError(pub String);

impl FromStr for WorkspaceStatus {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Created" => Ok(WorkspaceStatus::Created),
            "Starting" => Ok(WorkspaceStatus::Starting),
            "Running" => Ok(WorkspaceStatus::Running),
            "Paused" => Ok(WorkspaceStatus::Paused),
            "Error" => Ok(WorkspaceStatus::Error),
            "Deleted" => Ok(WorkspaceStatus::Deleted),
            other => Err(ParseStatusError(other.to_string())),
        }
    }
}
