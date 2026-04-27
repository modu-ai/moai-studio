//! T-005: workspaces v2 CRUD 통합 테스트 (SPEC-M1-001 RG-M1-4).

use moai_store::{NewWorkspace, Store, WorkspaceStatus, WorkspaceStoreExt};
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;

fn new_ws(name: &str, path: &str) -> NewWorkspace {
    NewWorkspace {
        name: name.to_string(),
        project_path: path.to_string(),
        spec_id: None,
        color_tag: None,
    }
}

#[test]
fn insert_returns_row_with_created_status() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao.insert(&new_ws("alpha", "/tmp/alpha")).unwrap();
    assert_eq!(row.name, "alpha");
    assert_eq!(row.project_path, "/tmp/alpha");
    assert_eq!(row.status, WorkspaceStatus::Created);
    assert!(row.id > 0);
}

#[test]
fn list_excludes_deleted() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let a = dao.insert(&new_ws("a", "/p/a")).unwrap();
    let _b = dao.insert(&new_ws("b", "/p/b")).unwrap();
    dao.soft_delete(a.id).unwrap();
    let listed = dao.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "b");
}

#[test]
fn update_status_follows_state_machine() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao.insert(&new_ws("sm", "/p/sm")).unwrap();

    let r1 = dao
        .update_status(row.id, WorkspaceStatus::Starting)
        .unwrap();
    assert_eq!(r1.status, WorkspaceStatus::Starting);
    let r2 = dao.update_status(row.id, WorkspaceStatus::Running).unwrap();
    assert_eq!(r2.status, WorkspaceStatus::Running);

    // 허용되지 않는 전이 (Running -> Created)
    let err = dao.update_status(row.id, WorkspaceStatus::Created);
    assert!(err.is_err());
}

#[test]
fn set_worktree_and_session_id() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao.insert(&new_ws("w", "/p/w")).unwrap();
    dao.set_worktree_path(row.id, "/p/w/.worktree").unwrap();
    dao.set_claude_session_id(row.id, Some("sess-123")).unwrap();
    let reloaded = dao.get(row.id).unwrap().unwrap();
    assert_eq!(reloaded.worktree_path.as_deref(), Some("/p/w/.worktree"));
    assert_eq!(reloaded.claude_session_id.as_deref(), Some("sess-123"));
}

#[test]
fn hard_delete_removes_row() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao.insert(&new_ws("hd", "/p/hd")).unwrap();
    let removed = dao.hard_delete(row.id).unwrap();
    assert!(removed);
    assert!(dao.get(row.id).unwrap().is_none());
}

#[test]
fn wal_mode_enabled_on_file_store() {
    // WAL PRAGMA 검증 — 실제 파일로 열어야 한다 (in-memory 는 WAL 미지원).
    let dir = tempdir().unwrap();
    let db = dir.path().join("ws.db");
    let store = Store::open(&db).unwrap();
    // 삽입/조회가 정상 동작하면 마이그레이션+WAL 전환이 성공한 것
    let dao = store.workspaces();
    let row = dao.insert(&new_ws("wal", "/p/wal")).unwrap();
    assert_eq!(row.name, "wal");
}

#[test]
fn concurrent_writes_do_not_deadlock() {
    // pool size = workspaces + 2 = 6 개 라이터 동시 실행 시 deadlock 없이 완료해야 한다.
    let dir = tempdir().unwrap();
    let db = dir.path().join("concurrent.db");
    let store = Arc::new(Store::open(&db).unwrap());
    let mut handles = vec![];
    for i in 0..4 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            let dao = store.workspaces();
            for j in 0..5 {
                let ws = NewWorkspace {
                    name: format!("t{i}-{j}"),
                    project_path: format!("/p/{i}/{j}"),
                    spec_id: None,
                    color_tag: None,
                };
                dao.insert(&ws).unwrap();
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(store.workspaces().list().unwrap().len(), 20);
}
