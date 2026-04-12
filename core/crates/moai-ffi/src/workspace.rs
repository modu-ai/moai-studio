//! 워크스페이스 레지스트리: FFI 호출이 조작하는 메모리 내 상태.
//!
//! 실제 스토어/Supervisor 연결은 M1 MS-2 (T-009~T-011) 에서 이 레이어를
//! `moai_supervisor::RootSupervisor` 로 교체한다. 현재는 in-memory stub 으로
//! Swift 바인딩이 정상 동작하는지 검증하는 데만 사용된다.

use std::collections::HashMap;
use std::sync::Mutex;

use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::events::{EventQueue, EventQueueHandle};
use crate::ffi::WorkspaceInfo;

/// 워크스페이스 한 개의 내부 상태.
// @MX:NOTE: [AUTO] M1 MS-2 에서 moai-store / moai-supervisor 로 치환될 stub.
struct WorkspaceEntry {
    name: String,
    #[allow(dead_code)] // M1 MS-2 에서 worktree 기준 경로로 사용 예정
    project_path: String,
    status: String,
    events: EventQueueHandle,
}

/// 전역 워크스페이스 레지스트리.
pub(crate) struct WorkspaceRegistry {
    inner: Mutex<HashMap<String, WorkspaceEntry>>,
}

impl WorkspaceRegistry {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// 새 워크스페이스를 등록하고 UUID 를 반환한다.
    pub(crate) fn create(&self, name: String, project_path: String) -> String {
        let id = Uuid::new_v4().to_string();
        let entry = WorkspaceEntry {
            name,
            project_path,
            status: "Created".to_string(),
            events: EventQueue::new_handle(),
        };
        // @MX:NOTE: [AUTO] Mutex poisoning 은 프로세스 불변식 위반이므로 expect.
        self.inner
            .lock()
            .expect("WorkspaceRegistry mutex poisoned")
            .insert(id.clone(), entry);
        id
    }

    /// 워크스페이스를 제거한다. 존재하지 않으면 false.
    pub(crate) fn delete(&self, id: &str) -> bool {
        self.inner
            .lock()
            .expect("WorkspaceRegistry mutex poisoned")
            .remove(id)
            .is_some()
    }

    /// 스냅샷 목록. FFI 경계에서 호출되므로 잠금 범위를 최소화한다.
    pub(crate) fn list(&self) -> Vec<WorkspaceInfo> {
        let guard = self.inner.lock().expect("WorkspaceRegistry mutex poisoned");
        guard
            .iter()
            .map(|(id, entry)| WorkspaceInfo {
                id: id.clone(),
                name: entry.name.clone(),
                status: entry.status.clone(),
            })
            .collect()
    }

    /// 메시지를 큐에 발행한다. 워크스페이스가 없으면 false.
    pub(crate) fn send_message(&self, id: &str, message: String, runtime: &Runtime) -> bool {
        let handle = {
            let guard = self.inner.lock().expect("WorkspaceRegistry mutex poisoned");
            match guard.get(id) {
                Some(entry) => entry.events.clone(),
                None => return false,
            }
        };
        // tokio::spawn 으로 비동기 발행 — FFI 호출 오버헤드를 <1ms 로 유지
        let payload = serde_json_event("user_message", &message);
        runtime.spawn(async move {
            handle.push(payload).await;
        });
        true
    }

    /// 이벤트 구독을 활성화한다 (현재 구현에서는 큐가 이미 준비되어 있으므로 no-op).
    pub(crate) fn subscribe(&self, id: &str, _runtime: &Runtime) -> bool {
        let guard = self.inner.lock().expect("WorkspaceRegistry mutex poisoned");
        guard.contains_key(id)
    }

    /// 큐에서 이벤트 하나를 꺼낸다.
    pub(crate) fn poll_event(&self, id: &str) -> Option<String> {
        let handle = {
            let guard = self.inner.lock().expect("WorkspaceRegistry mutex poisoned");
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
        let id = reg.create("alpha".into(), "/tmp/alpha".into());
        let listed = reg.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, id);
        assert_eq!(listed[0].name, "alpha");
        assert_eq!(listed[0].status, "Created");
    }

    #[test]
    fn delete_removes_entry() {
        let reg = WorkspaceRegistry::new();
        let id = reg.create("beta".into(), "/tmp/beta".into());
        assert!(reg.delete(&id));
        assert!(reg.list().is_empty());
        assert!(!reg.delete(&id), "두 번째 삭제는 false");
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
