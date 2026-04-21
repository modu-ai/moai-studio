//! 앱 재시작 시 store 로부터 워크스페이스를 복원한다 (SPEC-M1-001 RG-M1-4, T-011).
//!
//! store 에서 Deleted 를 제외한 모든 row 를 읽어 `WorkspaceHandle` 을 `Paused`
//! 상태로 rehydrate 한다. Claude subprocess 는 사용자가 실제로 해당 워크스페이스
//! 를 활성화할 때까지 spawn 하지 않는다 (lazy restart).

use std::path::PathBuf;
use std::sync::Arc;

use moai_store::{WorkspaceStatus, WorkspaceStoreExt};

use crate::lifecycle::LifecycleError;
use crate::root::RootSupervisor;
use crate::workspace::{WorkspaceHandle, WorkspaceId, WorkspaceState};

/// store 의 모든 비-Deleted 워크스페이스를 Paused 런타임 상태로 복원한다.
///
/// 반환값은 복원된 워크스페이스 수.
// @MX:ANCHOR: [AUTO] App 재시작 복원 단일 진입점 (fan_in>=2: ffi init, integration tests)
// @MX:REASON: [AUTO] 복원 로직이 여러 경로로 흩어지면 Claude 재spawn 타이밍 버그 유발.
pub async fn restore_from_store(supervisor: &Arc<RootSupervisor>) -> Result<usize, LifecycleError> {
    let rows = supervisor.store().workspaces().list()?;
    let mut restored = 0usize;
    for row in rows {
        let id = WorkspaceId(row.id);
        // 모든 비-Deleted 워크스페이스를 Paused 로 rehydrate.
        // Error 상태로 저장된 경우도 Paused 로 복원해 사용자 개입을 기다린다.
        let worktree_path = row.worktree_path.map(PathBuf::from);
        let handle = WorkspaceHandle {
            id,
            name: row.name,
            project_path: PathBuf::from(row.project_path),
            worktree_path,
            state: WorkspaceState::Paused,
        };
        supervisor.upsert_handle(handle).await;
        // store 상태도 Paused 로 정렬 (lazy restart 대기).
        if row.status != WorkspaceStatus::Paused && row.status != WorkspaceStatus::Deleted {
            // 일부 상태는 직접 Paused 로 전이가 불가하므로 store 컬럼을 강제로 덮어쓴다.
            force_paused(&supervisor.store(), id.as_i64())?;
        }
        restored += 1;
    }
    Ok(restored)
}

fn force_paused(store: &moai_store::Store, id: i64) -> Result<(), LifecycleError> {
    // state machine 우회 — 재시작 시에만 허용되는 관리자 경로.
    // @MX:WARN: [AUTO] state machine 우회 경로.
    // @MX:REASON: [AUTO] 재시작 시 Starting/Running 에 걸린 row 를 안전하게 Paused 로 되돌려야 한다.
    use moai_store::WorkspaceStoreExt;
    let dao = store.workspaces();
    // Running/Starting/Error → Paused 직접 전이는 state-machine 이 금지한다.
    // 재시작 경로이므로 row 를 먼저 Error → 다음 Paused 로 우회한다.
    // 간단한 UPDATE 쿼리를 위해 dao 의 transitions 를 우회하지 않고 2단계로 가면
    // 여전히 금지 상태가 많다. 따라서 여기서는 SQL 레벨로 직접 업데이트한다.
    //
    // 안전성: 이 함수는 restore 경로에서만 호출되며 앱 단일 프로세스 기준
    // 다른 액터가 동시에 row 를 만지지 않는다는 보장이 있다.
    let _ = dao; // placeholder — 실제 SQL 은 store API 로 노출하는 것이 이상적
    // 별도 저수준 API 를 추가하기보다, set_status_raw 를 사용한다.
    store.set_workspace_status_raw(id, WorkspaceStatus::Paused)?;
    Ok(())
}
