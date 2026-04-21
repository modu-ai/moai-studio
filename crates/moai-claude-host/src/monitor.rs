//! T-013: 서브프로세스 crash 감시 및 `Error` 상태 전이 브릿지.
//!
//! `ClaudeProcess::wait()` 결과를 supervisor EventBus에 올려주는 헬퍼다.
//! 실제 상태 전이는 호출자(RootSupervisor)가 `transition(... Error)`으로 수행한다.
//!
//! @MX:ANCHOR [AUTO] 서브프로세스 crash 감시 진입점 (fan_in>=1: supervisor/lifecycle)
//! @MX:WARN [AUTO] `wait()`는 소유권을 점유하므로 monitor를 시작한 이후에는 process를 다시 사용할 수 없다.
//! @MX:REASON [AUTO] tokio::process::Child::wait은 &mut self를 필요로 하므로 handle 공유 불가.

use crate::error::ProcessError;
use crate::process::ClaudeProcess;

/// 서브프로세스 종료 결과.
#[derive(Debug)]
pub enum ExitOutcome {
    /// 정상 종료 (exit 0).
    Normal,
    /// 비정상 종료 — `Error` 상태로 전이시켜야 한다.
    Crashed { exit_code: Option<i32> },
}

/// 서브프로세스가 종료될 때까지 기다린 뒤 `ExitOutcome`을 반환한다.
///
/// 정상 종료 시 `ExitOutcome::Normal`, 비정상 종료 시
/// `ExitOutcome::Crashed`를 반환한다. I/O 오류는 `ProcessError`로 전파된다.
pub async fn wait_for_exit(mut process: ClaudeProcess) -> Result<ExitOutcome, ProcessError> {
    match process.wait().await {
        Ok(()) => Ok(ExitOutcome::Normal),
        Err(ProcessError::ProcessCrashed { exit_code }) => Ok(ExitOutcome::Crashed { exit_code }),
        Err(other) => Err(other),
    }
}
