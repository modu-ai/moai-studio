//! T-010: 5단계 워크스페이스 생성 오케스트레이션 검증.

use moai_store::{Store, WorkspaceStatus, WorkspaceStoreExt};
use moai_supervisor::{
    RootSupervisor, WorkspaceCreateRequest, lifecycle::create_workspace, workspace::WorkspaceState,
};
use tempfile::tempdir;

#[tokio::test]
async fn full_pipeline_ends_in_running_state() {
    let tmp = tempdir().unwrap();
    let sup = RootSupervisor::new(Store::open_in_memory().unwrap());
    let proj = tmp.path().join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    let wt = tmp.path().join("wt");
    let id = create_workspace(
        &sup,
        WorkspaceCreateRequest {
            name: "pipeline".into(),
            project_path: proj.clone(),
            worktree_path: wt.clone(),
            spec_id: None,
        },
    )
    .await
    .unwrap();

    // 런타임 상태 = Running
    let state = sup.get_state(id).await.unwrap();
    assert_eq!(state, WorkspaceState::Running);

    // store 상태 = Running
    let row = sup.store().workspaces().get(id.as_i64()).unwrap().unwrap();
    assert_eq!(row.status, WorkspaceStatus::Running);

    // worktree 경로가 기록되어 있어야 한다
    assert_eq!(
        row.worktree_path.as_deref(),
        Some(wt.to_string_lossy().as_ref())
    );
    // worktree 디렉터리가 실제로 생성되어 있어야 한다
    assert!(wt.exists(), "worktree 경로가 실제 디스크에 존재해야 한다");

    // fs watcher 가 등록되었어야 한다
    assert_eq!(sup.event_bus().watch_count(), 1);
}

#[tokio::test]
async fn failure_in_worktree_rollbacks_store_row() {
    // 존재하지 않는 부모 경로를 worktree 로 주면 git2 가 실패한다.
    let tmp = tempdir().unwrap();
    let sup = RootSupervisor::new(Store::open_in_memory().unwrap());
    let proj = tmp.path().join("proj");
    std::fs::create_dir_all(&proj).unwrap();

    // worktree 경로의 부모가 존재하지 않음 → 실패 유도
    let wt = std::path::PathBuf::from("/non/existent/dir/ws-bad");
    let result = create_workspace(
        &sup,
        WorkspaceCreateRequest {
            name: "bad".into(),
            project_path: proj,
            worktree_path: wt,
            spec_id: None,
        },
    )
    .await;
    assert!(result.is_err());
    // 롤백 후 store 에는 row 가 남아있지 않아야 한다.
    assert_eq!(sup.store().workspaces().list().unwrap().len(), 0);
}
