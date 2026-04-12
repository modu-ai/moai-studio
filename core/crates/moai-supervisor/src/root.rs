//! `RootSupervisor` — 다중 워크스페이스 오케스트레이션 (SPEC-M1-001 RG-M1-4).

// @MX:ANCHOR: [AUTO] 모든 workspace 생명주기의 단일 진입점 (fan_in>=5: ffi/ui/lifecycle/restore/tests)
// @MX:REASON: [AUTO] 단일 Arc<RootSupervisor> 가 Swift UI, FFI, 내부 모듈 모두에서 공유된다.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::RwLock;

use moai_fs::FsEventBus;
use moai_store::{Store, StoreError, WorkspaceStatus, WorkspaceStoreExt};

use crate::workspace::{WorkspaceHandle, WorkspaceId, WorkspaceSnapshot, WorkspaceState};

/// 루트 슈퍼바이저 오류.
#[derive(Debug, Error)]
pub enum SupervisorError {
    /// 스토어 오류
    #[error("store 오류: {0}")]
    Store(#[from] StoreError),

    /// 워크스페이스가 존재하지 않음
    #[error("워크스페이스 없음: id={0}")]
    NotFound(i64),

    /// 잘못된 상태 (예: 이미 종료된 워크스페이스)
    #[error("잘못된 상태: {0}")]
    InvalidState(String),
}

/// 모든 워크스페이스를 보유하고 생명주기를 관리하는 루트 슈퍼바이저.
pub struct RootSupervisor {
    store: Store,
    event_bus: Arc<FsEventBus>,
    workspaces: RwLock<HashMap<WorkspaceId, WorkspaceHandle>>,
}

impl RootSupervisor {
    /// 새 슈퍼바이저를 생성한다. 인메모리 스토어를 사용하려면 `Store::open_in_memory()` 결과를 넘기면 된다.
    pub fn new(store: Store) -> Arc<Self> {
        Arc::new(Self {
            store,
            event_bus: FsEventBus::new(),
            workspaces: RwLock::new(HashMap::new()),
        })
    }

    /// 내부 store 핸들 클론.
    pub fn store(&self) -> Store {
        self.store.clone()
    }

    /// 내부 event bus 핸들 클론.
    pub fn event_bus(&self) -> Arc<FsEventBus> {
        Arc::clone(&self.event_bus)
    }

    /// 워크스페이스 개수.
    pub async fn len(&self) -> usize {
        self.workspaces.read().await.len()
    }

    /// 비어있는지 여부.
    pub async fn is_empty(&self) -> bool {
        self.workspaces.read().await.is_empty()
    }

    /// 현재 등록된 워크스페이스 스냅샷 목록.
    pub async fn list(&self) -> Vec<WorkspaceSnapshot> {
        self.workspaces
            .read()
            .await
            .values()
            .map(|w| w.snapshot())
            .collect()
    }

    /// 단일 스냅샷 조회.
    pub async fn get(&self, id: WorkspaceId) -> Option<WorkspaceSnapshot> {
        self.workspaces.read().await.get(&id).map(|w| w.snapshot())
    }

    /// 런타임 상태 조회.
    pub async fn get_state(&self, id: WorkspaceId) -> Option<WorkspaceState> {
        self.workspaces
            .read()
            .await
            .get(&id)
            .map(|w| w.state.clone())
    }

    /// 내부 전용 — handle 삽입/업데이트 (lifecycle/restore 에서 사용).
    pub async fn upsert_handle(&self, handle: WorkspaceHandle) {
        self.workspaces.write().await.insert(handle.id, handle);
    }

    /// 내부 전용 — 상태만 갱신 (store 와 독립).
    pub(crate) async fn set_runtime_state(
        &self,
        id: WorkspaceId,
        state: WorkspaceState,
    ) -> Result<(), SupervisorError> {
        let mut guard = self.workspaces.write().await;
        let handle = guard
            .get_mut(&id)
            .ok_or(SupervisorError::NotFound(id.as_i64()))?;
        handle.state = state;
        Ok(())
    }

    /// 워크스페이스를 종료한다. store 는 Deleted 로 soft-delete 하고,
    /// fs watcher 를 중지한다. worktree 경로는 상위 lifecycle 에서 정리한다.
    pub async fn terminate(&self, id: WorkspaceId) -> Result<(), SupervisorError> {
        // store soft delete
        let dao = self.store.workspaces();
        dao.soft_delete(id.as_i64())?;
        // fs watcher 중지
        let _ = self.event_bus.stop_watching(id.as_i64() as u64);
        // 런타임 맵에서 제거
        self.workspaces.write().await.remove(&id);
        Ok(())
    }

    /// 내부 전용 — worktree path 업데이트.
    pub(crate) async fn set_worktree_path(
        &self,
        id: WorkspaceId,
        path: PathBuf,
    ) -> Result<(), SupervisorError> {
        let mut guard = self.workspaces.write().await;
        let handle = guard
            .get_mut(&id)
            .ok_or(SupervisorError::NotFound(id.as_i64()))?;
        handle.worktree_path = Some(path.clone());
        drop(guard);
        self.store
            .workspaces()
            .set_worktree_path(id.as_i64(), path.to_string_lossy().as_ref())?;
        Ok(())
    }

    /// store 상태와 런타임 상태를 모두 갱신 (상태 전이 검증 포함).
    pub async fn transition(
        &self,
        id: WorkspaceId,
        next: WorkspaceStatus,
    ) -> Result<(), SupervisorError> {
        self.store.workspaces().update_status(id.as_i64(), next)?;
        let runtime: WorkspaceState = next.into();
        self.set_runtime_state(id, runtime).await?;
        Ok(())
    }
}
