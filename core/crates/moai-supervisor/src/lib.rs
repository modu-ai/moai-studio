//! moai-supervisor: tokio 기반 액터 트리 슈퍼바이저
//!
//! 워크스페이스 생명주기를 액터 트리로 관리한다.
//! 각 워크스페이스는 Claude 서브프로세스, MCP 서버, 훅 수신자를 소유하는
//! 슈퍼바이저를 갖는다.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// 워크스페이스를 식별하는 고유 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(u64);

impl WorkspaceId {
    /// 내부 u64 값을 반환한다.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// 워크스페이스의 현재 상태
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceState {
    /// 시작 중
    Starting,
    /// 실행 중
    Running,
    /// 종료 중
    Stopping,
    /// 종료 완료
    Stopped,
    /// 오류 발생 (메시지 포함)
    Error(String),
}

/// 워크스페이스 생성 설정
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    /// 작업 디렉터리 경로
    pub working_dir: PathBuf,
    /// Claude API 키
    pub api_key: String,
    /// MCP 설정 파일 경로 (선택 사항)
    pub mcp_config_path: Option<PathBuf>,
}

/// 여러 워크스페이스를 관리하는 루트 슈퍼바이저
pub struct RootSupervisor {
    /// 워크스페이스 상태 맵: ID → 상태
    workspaces: RwLock<HashMap<WorkspaceId, WorkspaceState>>,
    /// 다음 워크스페이스 ID를 생성하기 위한 카운터
    next_id: AtomicU64,
}

impl RootSupervisor {
    /// 새 루트 슈퍼바이저를 생성한다.
    pub fn new() -> Self {
        Self {
            workspaces: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// 새 워크스페이스를 생성하고 고유 ID를 반환한다.
    ///
    /// 초기 상태는 `Starting`이다.
    pub async fn create_workspace(&self, _config: WorkspaceConfig) -> WorkspaceId {
        let id = WorkspaceId(self.next_id.fetch_add(1, Ordering::SeqCst));
        let mut map = self.workspaces.write().await;
        map.insert(id, WorkspaceState::Starting);
        id
    }

    /// 주어진 ID의 워크스페이스 상태를 반환한다.
    ///
    /// 존재하지 않으면 `None`을 반환한다.
    pub async fn get_state(&self, id: WorkspaceId) -> Option<WorkspaceState> {
        let map = self.workspaces.read().await;
        map.get(&id).cloned()
    }

    /// 모든 워크스페이스의 (ID, 상태) 목록을 반환한다.
    pub async fn list_workspaces(&self) -> Vec<(WorkspaceId, WorkspaceState)> {
        let map = self.workspaces.read().await;
        map.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    /// 워크스페이스를 제거한다.
    ///
    /// 성공적으로 제거되면 `true`, 존재하지 않으면 `false`를 반환한다.
    pub async fn remove_workspace(&self, id: WorkspaceId) -> bool {
        let mut map = self.workspaces.write().await;
        map.remove(&id).is_some()
    }
}

impl Default for RootSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 테스트용 기본 설정을 생성하는 헬퍼
    fn make_config(dir: &str) -> WorkspaceConfig {
        WorkspaceConfig {
            working_dir: PathBuf::from(dir),
            api_key: "test-key".to_string(),
            mcp_config_path: None,
        }
    }

    #[tokio::test]
    async fn test_create_workspace_returns_unique_ids() {
        // 여러 워크스페이스를 생성하면 각각 고유한 ID를 받아야 한다.
        let supervisor = RootSupervisor::new();

        let id1 = supervisor.create_workspace(make_config("/workspace/a")).await;
        let id2 = supervisor.create_workspace(make_config("/workspace/b")).await;
        let id3 = supervisor.create_workspace(make_config("/workspace/c")).await;

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[tokio::test]
    async fn test_get_state_returns_starting() {
        // 새로 생성된 워크스페이스의 초기 상태는 Starting이어야 한다.
        let supervisor = RootSupervisor::new();
        let id = supervisor.create_workspace(make_config("/workspace/x")).await;

        let state = supervisor.get_state(id).await;

        assert_eq!(state, Some(WorkspaceState::Starting));
    }

    #[tokio::test]
    async fn test_list_workspaces() {
        // 생성된 워크스페이스 수만큼 목록이 반환되어야 한다.
        let supervisor = RootSupervisor::new();

        assert_eq!(supervisor.list_workspaces().await.len(), 0);

        let id1 = supervisor.create_workspace(make_config("/workspace/a")).await;
        let id2 = supervisor.create_workspace(make_config("/workspace/b")).await;

        let list = supervisor.list_workspaces().await;
        assert_eq!(list.len(), 2);

        // 두 ID 모두 목록에 포함되어야 한다.
        let ids: Vec<WorkspaceId> = list.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[tokio::test]
    async fn test_remove_workspace() {
        // 워크스페이스를 제거하면 목록에서 사라져야 한다.
        let supervisor = RootSupervisor::new();
        let id = supervisor.create_workspace(make_config("/workspace/del")).await;

        // 제거 전: 존재해야 한다.
        assert!(supervisor.get_state(id).await.is_some());

        // 제거: true 반환
        let removed = supervisor.remove_workspace(id).await;
        assert!(removed);

        // 제거 후: 없어야 한다.
        assert!(supervisor.get_state(id).await.is_none());
    }

    #[tokio::test]
    async fn test_get_nonexistent_workspace_returns_none() {
        // 존재하지 않는 ID를 조회하면 None을 반환해야 한다.
        let supervisor = RootSupervisor::new();
        let fake_id = WorkspaceId(9999);

        assert_eq!(supervisor.get_state(fake_id).await, None);
    }

    #[tokio::test]
    async fn test_multiple_workspaces_independent() {
        // 여러 워크스페이스의 상태는 서로 독립적이어야 한다.
        let supervisor = RootSupervisor::new();

        let id1 = supervisor.create_workspace(make_config("/workspace/1")).await;
        let id2 = supervisor.create_workspace(make_config("/workspace/2")).await;

        // id2를 제거해도 id1은 여전히 존재해야 한다.
        supervisor.remove_workspace(id2).await;

        assert_eq!(supervisor.get_state(id1).await, Some(WorkspaceState::Starting));
        assert_eq!(supervisor.get_state(id2).await, None);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_workspace_returns_false() {
        // 존재하지 않는 워크스페이스 제거 시 false를 반환해야 한다.
        let supervisor = RootSupervisor::new();
        let fake_id = WorkspaceId(42);

        assert!(!supervisor.remove_workspace(fake_id).await);
    }
}
