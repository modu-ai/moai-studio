//! 워크스페이스 생성 5단계 오케스트레이션 (SPEC-M1-001 RG-M1-4, T-010).
//!
//! 1. store insert (Created)
//! 2. git worktree init
//! 3. fs watcher start
//! 4. claude-host spawn stub (MS-3 T-012 에서 교체)
//! 5. 상태 전이 Created → Starting → Running

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

use moai_git::WorktreeManager;
use moai_store::{NewWorkspace, WorkspaceStatus, WorkspaceStoreExt};

use crate::root::{RootSupervisor, SupervisorError};
use crate::workspace::{WorkspaceHandle, WorkspaceId, WorkspaceState};

/// 워크스페이스 생성 요청 파라미터.
#[derive(Debug, Clone)]
pub struct WorkspaceCreateRequest {
    /// 사용자 노출 이름
    pub name: String,
    /// 프로젝트 루트 경로 (이 경로에 .git 이 이미 존재해야 한다)
    pub project_path: PathBuf,
    /// worktree 를 생성할 경로
    pub worktree_path: PathBuf,
    /// 연결 SPEC id
    pub spec_id: Option<String>,
}

/// 생명주기 단계 실패.
#[derive(Debug, Error)]
pub enum LifecycleError {
    /// 슈퍼바이저/스토어 오류
    #[error(transparent)]
    Supervisor(#[from] SupervisorError),

    /// store 오류
    #[error(transparent)]
    Store(#[from] moai_store::StoreError),

    /// git 오류
    #[error("git worktree 오류: {0}")]
    Git(#[from] moai_git::GitError),

    /// fs 감시 오류
    #[error("fs 감시 오류: {0}")]
    Fs(#[from] moai_fs::FsWatcherError),

    /// claude-host spawn 오류 (MS-3 에서 실제 서브프로세스로 교체)
    #[error("claude-host spawn 오류: {0}")]
    ClaudeHost(String),
}

/// 워크스페이스 생성 (5단계 오케스트레이션).
///
/// 실패 시 이전 단계의 부수 효과를 best-effort 로 롤백한다.
// @MX:ANCHOR: [AUTO] Workspace 생성 단일 진입점 (fan_in>=2: ffi/restore)
// @MX:REASON: [AUTO] 5단계 트랜잭션 경계는 여기서만 정의된다 — 우회 금지.
pub async fn create_workspace(
    supervisor: &Arc<RootSupervisor>,
    req: WorkspaceCreateRequest,
) -> Result<WorkspaceId, LifecycleError> {
    // 1. store insert (status = Created)
    let store = supervisor.store();
    let dao = store.workspaces();
    let row = dao.insert(&NewWorkspace {
        name: req.name.clone(),
        project_path: req.project_path.to_string_lossy().into(),
        spec_id: req.spec_id.clone(),
        color_tag: None,
    })?;
    let ws_id = WorkspaceId(row.id);

    // 런타임 핸들 등록
    supervisor
        .upsert_handle(WorkspaceHandle {
            id: ws_id,
            name: req.name.clone(),
            project_path: req.project_path.clone(),
            worktree_path: None,
            state: WorkspaceState::Created,
        })
        .await;

    // 2. git worktree init
    match init_worktree(&req.project_path, &req.name, &req.worktree_path) {
        Ok(()) => {
            supervisor
                .set_worktree_path(ws_id, req.worktree_path.clone())
                .await?;
        }
        Err(e) => {
            rollback_store(supervisor, ws_id).await;
            return Err(e);
        }
    }

    // 3. fs watcher start (worktree 경로 감시)
    let bus = supervisor.event_bus();
    if let Err(e) = bus.start_watching(ws_id.as_i64() as u64, &req.worktree_path) {
        rollback_store(supervisor, ws_id).await;
        return Err(e.into());
    }

    // 4. claude-host spawn (MS-3 T-012) — claude 바이너리 + ANTHROPIC_API_KEY 존재 시 실 spawn,
    //    없으면 stub 으로 폴백 (단위 테스트 / CI 환경 대응).
    // @MX:NOTE: [AUTO] 실제 subprocess 수명 관리는 MS-4 WorkspaceSupervisor 에서 담당.
    spawn_claude_host_or_stub(ws_id, &req.worktree_path).await?;

    // 5. Created -> Starting -> Running 전이
    supervisor
        .transition(ws_id, WorkspaceStatus::Starting)
        .await?;
    supervisor
        .transition(ws_id, WorkspaceStatus::Running)
        .await?;

    Ok(ws_id)
}

fn init_worktree(
    project_path: &Path,
    name: &str,
    worktree_path: &Path,
) -> Result<(), LifecycleError> {
    // project_path 가 .git 이 없으면 init 한다 (테스트 편의).
    let mgr = if project_path.join(".git").exists() {
        WorktreeManager::open(project_path)?
    } else {
        WorktreeManager::init(project_path)?
    };
    mgr.create_worktree(name, worktree_path)?;
    Ok(())
}

/// T-012 통합 지점: claude subprocess를 실제로 spawn 하거나 stub 으로 폴백한다.
///
/// 실 spawn 조건:
///   - `which claude` 가 성공
///   - `ANTHROPIC_API_KEY` 환경 변수 설정
///
/// 조건 미충족 시 즉시 Ok(()) 를 반환하여 lifecycle 오케스트레이션을 차단하지 않는다.
///
/// @MX:NOTE: [AUTO] subprocess handle 자체는 현재 즉시 drop (shutdown). MS-4에서
///   WorkspaceSupervisor가 handle을 소유하도록 확장된다.
async fn spawn_claude_host_or_stub(
    _id: WorkspaceId,
    worktree_path: &Path,
) -> Result<(), LifecycleError> {
    let claude_available = std::process::Command::new("which")
        .arg("claude")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();

    if !claude_available || api_key.is_empty() {
        tracing::debug!(
            "claude-host spawn skipped (claude={claude_available}, api_key_set={})",
            !api_key.is_empty()
        );
        return Ok(());
    }

    let cfg = moai_claude_host::workspace_config(
        "claude",
        api_key,
        worktree_path.to_path_buf(),
        None,
        None,
    );

    match cfg.spawn().await {
        Ok(mut proc) => {
            // MS-4에서 WorkspaceSupervisor에 이관될 때까지 즉시 종료.
            proc.shutdown().await;
            Ok(())
        }
        Err(e) => Err(LifecycleError::ClaudeHost(format!("{e}"))),
    }
}

async fn rollback_store(supervisor: &Arc<RootSupervisor>, id: WorkspaceId) {
    // 실패 시 조용히 hard delete — 더 이상 store 에 잔류하지 않도록.
    let _ = supervisor.store().workspaces().hard_delete(id.as_i64());
    // 런타임 맵에서도 제거
    let _ = supervisor.terminate(id).await;
}
