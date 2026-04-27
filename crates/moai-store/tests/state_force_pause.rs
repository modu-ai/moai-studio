//! C-8: WorkspaceDao::force_pause 정식 API 테스트.
//!
//! force_pause 는 정상 전이 규칙을 우회하여 임의 상태에서 Paused 로 강제 전환한다.

use moai_store::{NewWorkspace, Store, WorkspaceStatus, WorkspaceStoreExt};

fn new_ws(name: &str) -> NewWorkspace {
    NewWorkspace {
        name: name.to_string(),
        project_path: "/tmp".to_string(),
        spec_id: None,
        color_tag: None,
    }
}

#[test]
fn force_pause_from_error_state() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let ws = dao.insert(&new_ws("ws-error")).unwrap();
    // Created → Error (정상 전이)
    dao.update_status(ws.id, WorkspaceStatus::Error).unwrap();
    // Error → Paused 는 정상 전이 불가; force_pause 로 우회
    let paused = dao.force_pause(ws.id).unwrap();
    assert_eq!(paused.status, WorkspaceStatus::Paused);
}

#[test]
fn force_pause_from_running_state() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let ws = dao.insert(&new_ws("ws-running")).unwrap();
    dao.update_status(ws.id, WorkspaceStatus::Starting).unwrap();
    dao.update_status(ws.id, WorkspaceStatus::Running).unwrap();
    // Running → Paused 는 정상 전이로도 가능하지만 force_pause 도 동작해야 함
    let paused = dao.force_pause(ws.id).unwrap();
    assert_eq!(paused.status, WorkspaceStatus::Paused);
}

#[test]
fn force_pause_subsequent_transitions_work() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let ws = dao.insert(&new_ws("ws-resume")).unwrap();
    dao.update_status(ws.id, WorkspaceStatus::Error).unwrap();
    dao.force_pause(ws.id).unwrap();
    // force_pause 후 정상 전이 (Paused → Starting) 가 동작해야 함
    let starting = dao.update_status(ws.id, WorkspaceStatus::Starting).unwrap();
    assert_eq!(starting.status, WorkspaceStatus::Starting);
}

#[test]
fn force_pause_not_found_returns_error() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let result = dao.force_pause(9999);
    assert!(
        result.is_err(),
        "존재하지 않는 id에 force_pause는 오류 반환"
    );
}
