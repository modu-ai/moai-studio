//! Pane/Surface FFI 통합 테스트 (SPEC-M2-001 MS-1 T-037 + MS-2 T-042)
//!
//! RustCore FFI 를 통해 pane/surface CRUD 및 JSON FFI 를 검증한다.

use moai_ffi::RustCore;

// ── T-035: Pane FFI ──────────────────────────────────────────────────────────

/// create_pane 은 새 pane id (> 0) 를 반환한다.
#[test]
fn create_pane_returns_nonzero_id() {
    let core = RustCore::new();
    // 워크스페이스를 먼저 생성한다
    let ws_uuid = core.create_workspace("test-ws".to_string(), "/tmp/pane-ffi-1".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    assert!(ws_db_id > 0, "ws_db_id={ws_db_id}");

    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);
    assert!(
        pane_id > 0,
        "create_pane 는 >0 을 반환해야 함, 실제={pane_id}"
    );
}

/// list_panes 는 생성된 pane 목록을 반환한다.
#[test]
fn list_panes_returns_created_panes() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-ws2".to_string(), "/tmp/pane-ffi-2".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);

    let p1 = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);
    let p2 = core.create_pane(ws_db_id, 0, "horizontal".to_string(), 0.3);
    assert!(p1 > 0 && p2 > 0);

    let list = core.list_panes(ws_db_id);
    assert_eq!(list.len(), 2);
}

/// update_pane_ratio 는 ratio 를 영속한다.
#[test]
fn update_pane_ratio_persists() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-ws3".to_string(), "/tmp/pane-ffi-3".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);

    let ok = core.update_pane_ratio(pane_id, 0.7);
    assert!(ok, "update_pane_ratio 는 true 를 반환해야 함");

    // ratio 가 변경됐는지 확인: list_panes 로 검증
    let list = core.list_panes(ws_db_id);
    let found = list.iter().find(|p| p.id == pane_id).unwrap();
    assert!((found.ratio - 0.7).abs() < f64::EPSILON);
}

/// delete_pane 은 pane 을 삭제하고 true 를 반환한다.
#[test]
fn delete_pane_removes_it() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-ws4".to_string(), "/tmp/pane-ffi-4".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);

    assert!(core.delete_pane(pane_id));
    let list = core.list_panes(ws_db_id);
    assert!(list.is_empty());
}

// ── T-036: Surface FFI ───────────────────────────────────────────────────────

/// create_surface + list_surfaces 는 tab_order 순서를 유지한다.
#[test]
fn create_surface_list_ordered_by_tab_order() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-ws5".to_string(), "/tmp/surface-ffi-1".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);

    let s1 = core.create_surface(pane_id, "terminal".to_string(), "".to_string(), 2);
    let s2 = core.create_surface(pane_id, "markdown".to_string(), "".to_string(), 0);
    let s3 = core.create_surface(pane_id, "image".to_string(), "".to_string(), 1);
    assert!(s1 > 0 && s2 > 0 && s3 > 0);

    let list = core.list_surfaces(pane_id);
    assert_eq!(list.len(), 3);
    // tab_order 오름차순
    assert_eq!(list[0].tab_order, 0);
    assert_eq!(list[1].tab_order, 1);
    assert_eq!(list[2].tab_order, 2);
}

/// update_surface_tab_order 는 tab_order 를 변경한다.
#[test]
fn update_surface_tab_order_persists() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-ws6".to_string(), "/tmp/surface-ffi-2".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);
    let surf_id = core.create_surface(pane_id, "terminal".to_string(), "".to_string(), 0);

    let ok = core.update_surface_tab_order(surf_id, 5);
    assert!(ok);

    let list = core.list_surfaces(pane_id);
    assert_eq!(list[0].tab_order, 5);
}

/// delete_pane 은 FK CASCADE 로 하위 surface 도 삭제한다.
#[test]
fn delete_pane_cascades_surfaces() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-ws7".to_string(), "/tmp/surface-ffi-3".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);
    let _surf_id = core.create_surface(pane_id, "terminal".to_string(), "".to_string(), 0);

    // pane 삭제
    assert!(core.delete_pane(pane_id));

    // surface 도 사라져야 한다
    let list = core.list_surfaces(pane_id);
    assert!(
        list.is_empty(),
        "pane 삭제 시 surface 도 CASCADE 삭제되어야 함"
    );
}

// ── T-042 (JSON FFI): list_panes_json / list_surfaces_json ──────────────────

/// list_panes_json 은 유효한 JSON 배열을 반환한다.
#[test]
fn list_panes_json_returns_valid_json() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-json-1".to_string(), "/tmp/json-ffi-1".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let _pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);

    let json = core.list_panes_json(ws_db_id);
    // 유효한 JSON 배열인지 확인
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("list_panes_json 은 유효한 JSON 을 반환해야 함");
    assert!(parsed.is_array(), "JSON 최상위는 배열이어야 함");
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

/// list_panes_json 은 올바른 필드를 포함한 pane 데이터를 반환한다.
#[test]
fn list_panes_json_contains_correct_fields() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-json-2".to_string(), "/tmp/json-ffi-2".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "horizontal".to_string(), 0.7);

    let json = core.list_panes_json(ws_db_id);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);

    let pane = &arr[0];
    assert_eq!(pane["id"].as_i64().unwrap(), pane_id);
    assert_eq!(pane["workspace_id"].as_i64().unwrap(), ws_db_id);
    assert_eq!(pane["split"].as_str().unwrap(), "horizontal");
    assert!((pane["ratio"].as_f64().unwrap() - 0.7).abs() < f64::EPSILON);
}

/// pane 이 없을 때 list_panes_json 은 빈 배열 "[]" 를 반환한다.
#[test]
fn list_panes_json_empty_returns_empty_array() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-json-3".to_string(), "/tmp/json-ffi-3".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);

    let json = core.list_panes_json(ws_db_id);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 0);
}

/// list_surfaces_json 은 유효한 JSON 배열을 반환한다.
#[test]
fn list_surfaces_json_returns_valid_json() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-json-4".to_string(), "/tmp/json-ffi-4".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);
    let _surf_id = core.create_surface(pane_id, "terminal".to_string(), "".to_string(), 0);

    let json = core.list_surfaces_json(pane_id);
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("list_surfaces_json 은 유효한 JSON 을 반환해야 함");
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

/// list_surfaces_json 은 kind 필드를 포함한다.
#[test]
fn list_surfaces_json_contains_kind_field() {
    let core = RustCore::new();
    let ws_uuid = core.create_workspace("test-json-5".to_string(), "/tmp/json-ffi-5".to_string());
    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    let pane_id = core.create_pane(ws_db_id, 0, "leaf".to_string(), 0.5);
    let surf_id = core.create_surface(pane_id, "markdown".to_string(), "".to_string(), 1);

    let json = core.list_surfaces_json(pane_id);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    let surf = &arr[0];
    assert_eq!(surf["id"].as_i64().unwrap(), surf_id);
    assert_eq!(surf["kind"].as_str().unwrap(), "markdown");
    assert_eq!(surf["tab_order"].as_i64().unwrap(), 1);
}
