//! 워크스페이스 레지스트리 — M1 MS-2 부터 `moai_supervisor::RootSupervisor` 로 교체됨.
//!
//! FFI 는 기존 UUID string ID 계약을 유지하면서 내부적으로 supervisor 의 i64 id 로
//! 매핑한다. 이벤트 큐는 UI 폴링 모델을 유지하기 위해 계속 사용한다.
//! Claude subprocess / MCP / hook 연결은 MS-3 에서 이 레이어에 bind 된다.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;
use uuid::Uuid;

use moai_store::Store;
use moai_supervisor::{
    RootSupervisor, WorkspaceCreateRequest,
    lifecycle::create_workspace as lifecycle_create,
    workspace::{WorkspaceHandle, WorkspaceId, WorkspaceState},
};

use crate::events::{EventQueue, EventQueueHandle};
use crate::ffi::WorkspaceInfo;

// @MX:NOTE: [AUTO] FFI 경계 UUID ↔ supervisor i64 id 매핑 테이블
struct EventEntry {
    supervisor_id: WorkspaceId,
    events: EventQueueHandle,
}

/// 전역 워크스페이스 레지스트리 (supervisor wrapper).
pub(crate) struct WorkspaceRegistry {
    supervisor: Arc<RootSupervisor>,
    /// UUID(FFI) → { WorkspaceId(supervisor), 이벤트 큐 }
    index: Mutex<HashMap<String, EventEntry>>,
}

impl WorkspaceRegistry {
    pub(crate) fn new() -> Self {
        // FFI 컨텍스트에서는 인메모리 스토어를 기본값으로 사용한다.
        // 실제 앱에서는 향후 영속 경로가 주입된다 (MS-3 에서 추가).
        let store = Store::open_in_memory().expect("in-memory Store 생성 실패");
        Self {
            supervisor: RootSupervisor::new(store),
            index: Mutex::new(HashMap::new()),
        }
    }

    /// 새 워크스페이스를 등록하고 UUID 를 반환한다.
    ///
    /// project_path 가 실제 디스크 디렉터리이고 git worktree 생성이 가능하면
    /// 전체 5단계 lifecycle 을 실행하고, 그렇지 않으면 lightweight 등록만 수행한다
    /// (FFI 단위 테스트 호환).
    pub(crate) fn create(&self, name: String, project_path: String, runtime: &Runtime) -> String {
        let uuid = Uuid::new_v4().to_string();
        let supervisor = Arc::clone(&self.supervisor);
        let proj = PathBuf::from(&project_path);

        // 디렉터리가 존재하고 쓰기 가능하면 full lifecycle 시도
        let sup_id: WorkspaceId = if proj.is_dir() {
            let worktree_root = std::env::temp_dir().join("moai-ffi-wt").join(&uuid);
            match runtime.block_on(lifecycle_create(
                &supervisor,
                WorkspaceCreateRequest {
                    name: name.clone(),
                    project_path: proj.clone(),
                    worktree_path: worktree_root,
                    spec_id: None,
                },
            )) {
                Ok(id) => id,
                Err(_) => runtime.block_on(self.insert_lightweight(&name, &proj)),
            }
        } else {
            runtime.block_on(self.insert_lightweight(&name, &proj))
        };

        let handle = EventEntry {
            supervisor_id: sup_id,
            events: EventQueue::new_handle(),
        };
        self.index
            .lock()
            .expect("WorkspaceRegistry mutex poisoned")
            .insert(uuid.clone(), handle);
        uuid
    }

    /// store row 만 삽입하고 런타임 핸들을 Created 로 등록 (git/fs/claude 생략).
    async fn insert_lightweight(&self, name: &str, proj: &std::path::Path) -> WorkspaceId {
        use moai_store::{NewWorkspace, WorkspaceStoreExt};
        let dao = self.supervisor.store().workspaces();
        let row = dao
            .insert(&NewWorkspace {
                name: name.to_string(),
                project_path: proj.to_string_lossy().into(),
                spec_id: None,
            })
            .expect("store insert 실패");
        let id = WorkspaceId(row.id);
        self.supervisor
            .upsert_handle(WorkspaceHandle {
                id,
                name: name.to_string(),
                project_path: proj.to_path_buf(),
                worktree_path: None,
                state: WorkspaceState::Created,
            })
            .await;
        id
    }

    /// 워크스페이스를 제거한다. 존재하지 않으면 false.
    pub(crate) fn delete(&self, id: &str, runtime: &Runtime) -> bool {
        let entry = {
            let mut guard = self.index.lock().expect("WorkspaceRegistry mutex poisoned");
            guard.remove(id)
        };
        let Some(entry) = entry else { return false };
        // supervisor 에서도 제거 — 실패는 무시 (이미 사라졌을 수 있음).
        let sup = Arc::clone(&self.supervisor);
        let sup_id = entry.supervisor_id;
        let _ = runtime.block_on(sup.terminate(sup_id));
        true
    }

    /// 스냅샷 목록 (FFI 경계 UUID string id 기준).
    pub(crate) fn list(&self, runtime: &Runtime) -> Vec<WorkspaceInfo> {
        let pairs: Vec<(String, WorkspaceId)> = {
            let guard = self.index.lock().expect("WorkspaceRegistry mutex poisoned");
            guard
                .iter()
                .map(|(uuid, e)| (uuid.clone(), e.supervisor_id))
                .collect()
        };
        let mut out = Vec::with_capacity(pairs.len());
        let sup = Arc::clone(&self.supervisor);
        for (uuid, sup_id) in pairs {
            if let Some(snap) = runtime.block_on(sup.get(sup_id)) {
                out.push(WorkspaceInfo {
                    id: uuid,
                    name: snap.name,
                    status: snap.status.to_string(),
                });
            }
        }
        out
    }

    /// 메시지를 큐에 발행한다. 워크스페이스가 없으면 false.
    pub(crate) fn send_message(&self, id: &str, message: String, runtime: &Runtime) -> bool {
        let handle = {
            let guard = self.index.lock().expect("WorkspaceRegistry mutex poisoned");
            match guard.get(id) {
                Some(entry) => entry.events.clone(),
                None => return false,
            }
        };
        let payload = serde_json_event("user_message", &message);
        runtime.spawn(async move {
            handle.push(payload).await;
        });
        true
    }

    /// 이벤트 구독을 활성화한다 (현재 구현에서는 큐가 이미 준비되어 있으므로 no-op).
    pub(crate) fn subscribe(&self, id: &str, _runtime: &Runtime) -> bool {
        let guard = self.index.lock().expect("WorkspaceRegistry mutex poisoned");
        guard.contains_key(id)
    }

    /// 큐에서 이벤트 하나를 꺼낸다.
    pub(crate) fn poll_event(&self, id: &str) -> Option<String> {
        let handle = {
            let guard = self.index.lock().expect("WorkspaceRegistry mutex poisoned");
            guard.get(id)?.events.clone()
        };
        handle.try_pop()
    }
}

/// 간단한 JSON 직렬화 (serde_json 의존을 피하기 위한 소형 포맷터).
// @MX:NOTE: [AUTO] 현 단계 FFI 시연용. T-014 stream-json 통합 시점에 serde_json 으로 교체.
fn serde_json_event(kind: &str, message: &str) -> String {
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"type\":\"{kind}\",\"message\":\"{escaped}\"}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_list_contains_entry() {
        let reg = WorkspaceRegistry::new();
        let rt = Runtime::new().unwrap();
        let id = reg.create("alpha".into(), "/tmp/alpha-nonexistent".into(), &rt);
        let listed = reg.list(&rt);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, id);
        assert_eq!(listed[0].name, "alpha");
        // lightweight 경로이므로 Created 상태
        assert_eq!(listed[0].status, "Created");
    }

    #[test]
    fn delete_removes_entry() {
        let reg = WorkspaceRegistry::new();
        let rt = Runtime::new().unwrap();
        let id = reg.create("beta".into(), "/tmp/beta-nonexistent".into(), &rt);
        assert!(reg.delete(&id, &rt));
        assert!(reg.list(&rt).is_empty());
        assert!(!reg.delete(&id, &rt), "두 번째 삭제는 false");
    }

    #[test]
    fn send_message_to_unknown_workspace_returns_false() {
        let reg = WorkspaceRegistry::new();
        let rt = Runtime::new().unwrap();
        assert!(!reg.send_message("nope", "hi".into(), &rt));
    }

    #[test]
    fn serialize_event_escapes_quotes() {
        let out = serde_json_event("user_message", "hello \"world\"");
        assert!(out.contains("\\\"world\\\""));
    }
}
