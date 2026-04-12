//! T-009: RootSupervisor 다중 워크스페이스 오케스트레이션 검증.

use std::path::PathBuf;

use moai_store::Store;
use moai_supervisor::{RootSupervisor, WorkspaceCreateRequest, lifecycle::create_workspace};
use tempfile::tempdir;

fn make_store() -> Store {
    Store::open_in_memory().unwrap()
}

fn make_req(name: &str, proj: PathBuf, wt: PathBuf) -> WorkspaceCreateRequest {
    WorkspaceCreateRequest {
        name: name.to_string(),
        project_path: proj,
        worktree_path: wt,
        spec_id: None,
    }
}

#[tokio::test]
async fn create_multiple_workspaces_are_tracked() {
    let tmp = tempdir().unwrap();
    let sup = RootSupervisor::new(make_store());

    for i in 0..3 {
        let proj = tmp.path().join(format!("proj-{i}"));
        std::fs::create_dir_all(&proj).unwrap();
        let wt = tmp.path().join(format!("wt-{i}"));
        let id = create_workspace(&sup, make_req(&format!("w{i}"), proj, wt))
            .await
            .expect("create should succeed");
        assert!(id.as_i64() > 0);
    }

    assert_eq!(sup.len().await, 3);
    let list = sup.list().await;
    assert_eq!(list.len(), 3);
}

#[tokio::test]
async fn terminate_removes_from_runtime_map() {
    let tmp = tempdir().unwrap();
    let sup = RootSupervisor::new(make_store());
    let proj = tmp.path().join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    let wt = tmp.path().join("wt");

    let id = create_workspace(&sup, make_req("a", proj, wt))
        .await
        .unwrap();
    assert_eq!(sup.len().await, 1);
    sup.terminate(id).await.unwrap();
    assert_eq!(sup.len().await, 0);
    assert!(sup.get(id).await.is_none());
}

#[tokio::test]
async fn event_bus_is_shared_across_workspaces() {
    let sup = RootSupervisor::new(make_store());
    // 슈퍼바이저는 공유 event bus 를 제공한다 — 여러 구독자가 붙을 수 있다.
    let bus = sup.event_bus();
    let _r1 = bus.subscribe();
    let _r2 = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 2);
}
