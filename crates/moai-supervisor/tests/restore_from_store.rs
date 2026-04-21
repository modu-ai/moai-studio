//! T-011: 앱 재시작 시 store 로부터 워크스페이스 복원 검증.

use moai_store::Store;
use moai_supervisor::{
    RootSupervisor, WorkspaceCreateRequest, lifecycle::create_workspace,
    restore::restore_from_store, workspace::WorkspaceState,
};
use tempfile::tempdir;

#[tokio::test]
async fn restart_rehydrates_workspaces_as_paused() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("studio.db");

    // Session 1: 워크스페이스 2개 생성
    {
        let store = Store::open(&db_path).unwrap();
        let sup = RootSupervisor::new(store);
        for i in 0..2 {
            let proj = tmp.path().join(format!("p-{i}"));
            std::fs::create_dir_all(&proj).unwrap();
            let wt = tmp.path().join(format!("w-{i}"));
            create_workspace(
                &sup,
                WorkspaceCreateRequest {
                    name: format!("ws-{i}"),
                    project_path: proj,
                    worktree_path: wt,
                    spec_id: None,
                },
            )
            .await
            .unwrap();
        }
        assert_eq!(sup.len().await, 2);
    }

    // Session 2 (앱 재시작): 새 Store/Supervisor 로 복원
    {
        let store = Store::open(&db_path).unwrap();
        let sup = RootSupervisor::new(store);
        assert_eq!(sup.len().await, 0, "복원 전에는 비어있어야 한다");

        let restored = restore_from_store(&sup).await.unwrap();
        assert_eq!(restored, 2);
        assert_eq!(sup.len().await, 2);

        // 모든 워크스페이스는 Paused 상태로 복원되어야 한다 (lazy restart).
        let list = sup.list().await;
        for snap in &list {
            let state = sup.get_state(snap.id).await.unwrap();
            assert_eq!(
                state,
                WorkspaceState::Paused,
                "복원된 워크스페이스는 Paused 여야 한다"
            );
        }
    }
}

#[tokio::test]
async fn restore_skips_deleted_rows() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("studio.db");

    // 워크스페이스 2개 생성 후 하나 삭제
    {
        let store = Store::open(&db_path).unwrap();
        let sup = RootSupervisor::new(store);
        let mut ids = vec![];
        for i in 0..2 {
            let proj = tmp.path().join(format!("p-{i}"));
            std::fs::create_dir_all(&proj).unwrap();
            let wt = tmp.path().join(format!("w-{i}"));
            let id = create_workspace(
                &sup,
                WorkspaceCreateRequest {
                    name: format!("ws-{i}"),
                    project_path: proj,
                    worktree_path: wt,
                    spec_id: None,
                },
            )
            .await
            .unwrap();
            ids.push(id);
        }
        sup.terminate(ids[0]).await.unwrap();
    }

    // 복원 시 삭제된 것은 제외
    {
        let store = Store::open(&db_path).unwrap();
        let sup = RootSupervisor::new(store);
        let restored = restore_from_store(&sup).await.unwrap();
        assert_eq!(restored, 1);
    }
}
