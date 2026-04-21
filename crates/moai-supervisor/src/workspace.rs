//! `WorkspaceSupervisor` child 액터 정의 (SPEC-M1-001 RG-M1-4, T-009).
//!
//! 각 워크스페이스는 한 개의 `WorkspaceHandle` 로 표현된다. 내부적으로는
//! store row id, worktree path, 현재 상태를 보관한다. Claude subprocess /
//! MCP 서버 / hook endpoint 연결은 MS-3 에서 추가된다 (stub placeholder).

use std::path::PathBuf;

use moai_store::WorkspaceStatus;

/// 워크스페이스 식별 ID (store rowid 와 동일).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub i64);

impl WorkspaceId {
    /// 내부 정수 값을 반환한다.
    pub fn as_i64(self) -> i64 {
        self.0
    }
}

/// UI 로 노출되는 워크스페이스 스냅샷 (Rust 내부용 — FFI 와 분리).
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceSnapshot {
    /// 워크스페이스 id
    pub id: WorkspaceId,
    /// 이름
    pub name: String,
    /// 프로젝트 루트 경로
    pub project_path: PathBuf,
    /// worktree 체크아웃 경로 (아직 없으면 None)
    pub worktree_path: Option<PathBuf>,
    /// 현재 상태
    pub status: WorkspaceStatus,
}

/// 현재 런타임 상태. store 의 영속 상태보다 세부 정보가 풍부할 수 있다.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceState {
    /// store 에만 존재, 아직 spawn 되지 않음
    Created,
    /// spawn/restore 진행 중
    Starting,
    /// 정상 동작 중
    Running,
    /// 일시 정지 (Claude subprocess 가 없는 상태로 복원됨)
    Paused,
    /// 에러 상태 — 사유 메시지 포함
    Error(String),
    /// 종료됨
    Deleted,
}

impl From<WorkspaceStatus> for WorkspaceState {
    fn from(s: WorkspaceStatus) -> Self {
        match s {
            WorkspaceStatus::Created => WorkspaceState::Created,
            WorkspaceStatus::Starting => WorkspaceState::Starting,
            WorkspaceStatus::Running => WorkspaceState::Running,
            WorkspaceStatus::Paused => WorkspaceState::Paused,
            WorkspaceStatus::Error => WorkspaceState::Error("unknown".into()),
            WorkspaceStatus::Deleted => WorkspaceState::Deleted,
        }
    }
}

/// 각 워크스페이스 super의 핸들 (RootSupervisor 내부에서 관리).
// @MX:NOTE: [AUTO] MS-3 에서 claude_session / mcp_port / hook_endpoint 필드가 추가될 예정.
#[derive(Debug, Clone)]
pub struct WorkspaceHandle {
    /// 워크스페이스 id
    pub id: WorkspaceId,
    /// 이름
    pub name: String,
    /// 프로젝트 루트
    pub project_path: PathBuf,
    /// worktree 경로
    pub worktree_path: Option<PathBuf>,
    /// 현재 상태
    pub state: WorkspaceState,
}

impl WorkspaceHandle {
    /// 스냅샷 뷰를 생성한다.
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        let status = match &self.state {
            WorkspaceState::Created => WorkspaceStatus::Created,
            WorkspaceState::Starting => WorkspaceStatus::Starting,
            WorkspaceState::Running => WorkspaceStatus::Running,
            WorkspaceState::Paused => WorkspaceStatus::Paused,
            WorkspaceState::Error(_) => WorkspaceStatus::Error,
            WorkspaceState::Deleted => WorkspaceStatus::Deleted,
        };
        WorkspaceSnapshot {
            id: self.id,
            name: self.name.clone(),
            project_path: self.project_path.clone(),
            worktree_path: self.worktree_path.clone(),
            status,
        }
    }
}
