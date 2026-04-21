//! panes/surfaces 통합 테스트 (SPEC-M2-001 MS-1 T-037)
//!
//! T-031~T-036 에서 구현된 기능을 종합적으로 검증한다.

use moai_store::{
    NewPane, NewSurface, PaneStoreExt, SplitKind, Store, StoreError, SurfaceKind, SurfaceStoreExt,
};

// ── T-031 / T-032: V3 마이그레이션 컬럼 존재 확인 ──────────────────────────

/// panes 테이블 컬럼이 V3 마이그레이션 후 존재해야 한다.
#[test]
fn v3_panes_columns_exist() {
    let store = Store::open_in_memory().unwrap();
    let guard = store.conn_for_test();
    let mut stmt = guard.prepare("PRAGMA table_info(panes)").unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |r: &rusqlite::Row<'_>| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r: Result<String, _>| r.ok())
        .collect();
    for expected in [
        "id",
        "workspace_id",
        "parent_id",
        "split",
        "ratio",
        "created_at",
        "updated_at",
    ] {
        assert!(
            cols.contains(&expected.to_string()),
            "panes 테이블에 컬럼 없음: {expected}"
        );
    }
}

/// surfaces 테이블 컬럼이 V3 마이그레이션 후 존재해야 한다.
#[test]
fn v3_surfaces_columns_exist() {
    let store = Store::open_in_memory().unwrap();
    let guard = store.conn_for_test();
    let mut stmt = guard.prepare("PRAGMA table_info(surfaces)").unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |r: &rusqlite::Row<'_>| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r: Result<String, _>| r.ok())
        .collect();
    for expected in [
        "id",
        "pane_id",
        "kind",
        "state_json",
        "tab_order",
        "created_at",
        "updated_at",
    ] {
        assert!(
            cols.contains(&expected.to_string()),
            "surfaces 테이블에 컬럼 없음: {expected}"
        );
    }
}

/// schema_version 이 3 이상이어야 한다.
#[test]
fn v3_schema_version_is_3() {
    let store = Store::open_in_memory().unwrap();
    let guard = store.conn_for_test();
    let version: i64 = guard
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |r: &rusqlite::Row<'_>| r.get(0),
        )
        .unwrap();
    assert!(version >= 3, "schema_version={version}, 3 이상이어야 함");
}

// ── T-033: Pane CRUD ─────────────────────────────────────────────────────────

/// 워크스페이스 ID 와 함께 루트 leaf pane 을 생성한다.
#[test]
fn pane_insert_and_get() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/pane-insert").unwrap();

    let dao = store.panes();
    let new = NewPane {
        workspace_id: ws_id,
        parent_id: None,
        split: SplitKind::Leaf,
        ratio: 0.5,
    };
    let row = dao.insert(&new).unwrap();
    assert!(row.id > 0);
    assert_eq!(row.workspace_id, ws_id);
    assert_eq!(row.split, SplitKind::Leaf);
    assert!((row.ratio - 0.5).abs() < f64::EPSILON);
    assert!(row.parent_id.is_none());

    // get 으로도 동일하게 조회
    let fetched = dao.get(row.id).unwrap().unwrap();
    assert_eq!(fetched.id, row.id);
}

/// list_by_workspace 는 해당 워크스페이스의 모든 pane 을 반환한다.
#[test]
fn pane_list_by_workspace() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/pane-list").unwrap();
    let dao = store.panes();

    // 루트 pane (horizontal split)
    let root = dao
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Horizontal,
            ratio: 0.5,
        })
        .unwrap();

    // 좌측 leaf
    dao.insert(&NewPane {
        workspace_id: ws_id,
        parent_id: Some(root.id),
        split: SplitKind::Leaf,
        ratio: 0.5,
    })
    .unwrap();

    // 우측 leaf
    dao.insert(&NewPane {
        workspace_id: ws_id,
        parent_id: Some(root.id),
        split: SplitKind::Leaf,
        ratio: 0.5,
    })
    .unwrap();

    let panes = dao.list_by_workspace(ws_id).unwrap();
    assert_eq!(panes.len(), 3);
}

/// update_ratio 는 ratio 를 변경하고 갱신된 행을 반환한다.
#[test]
fn pane_update_ratio() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/pane-ratio").unwrap();
    let dao = store.panes();
    let row = dao
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();

    let updated = dao.update_ratio(row.id, 0.3).unwrap();
    assert!((updated.ratio - 0.3).abs() < f64::EPSILON);
}

/// delete 는 pane 을 삭제하고 true 를 반환한다. 없는 경우 false.
#[test]
fn pane_delete() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/pane-delete").unwrap();
    let dao = store.panes();
    let row = dao
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();

    assert!(dao.delete(row.id).unwrap());
    assert!(!dao.delete(row.id).unwrap(), "두 번째 삭제는 false");
    assert!(dao.get(row.id).unwrap().is_none());
}

// ── T-034: Surface CRUD ──────────────────────────────────────────────────────

/// surface 를 삽입하고 list_by_pane 으로 tab_order 순서로 조회한다.
#[test]
fn surface_insert_list_by_pane_ordered() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/surface-list").unwrap();
    let pane = store
        .panes()
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();

    let sdao = store.surfaces();
    sdao.insert(&NewSurface {
        pane_id: pane.id,
        kind: SurfaceKind::Terminal,
        state_json: None,
        tab_order: 2,
    })
    .unwrap();
    sdao.insert(&NewSurface {
        pane_id: pane.id,
        kind: SurfaceKind::Markdown,
        state_json: Some("{\"path\":\"/foo.md\"}".to_string()),
        tab_order: 0,
    })
    .unwrap();
    sdao.insert(&NewSurface {
        pane_id: pane.id,
        kind: SurfaceKind::Image,
        state_json: None,
        tab_order: 1,
    })
    .unwrap();

    let list = sdao.list_by_pane(pane.id).unwrap();
    assert_eq!(list.len(), 3);
    // tab_order 오름차순
    assert_eq!(list[0].tab_order, 0);
    assert_eq!(list[1].tab_order, 1);
    assert_eq!(list[2].tab_order, 2);
    assert_eq!(list[0].kind, SurfaceKind::Markdown);
}

/// update_tab_order 는 tab_order 를 업데이트한다.
#[test]
fn surface_update_tab_order() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/surface-taborder").unwrap();
    let pane = store
        .panes()
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();
    let sdao = store.surfaces();
    let surf = sdao
        .insert(&NewSurface {
            pane_id: pane.id,
            kind: SurfaceKind::Terminal,
            state_json: None,
            tab_order: 0,
        })
        .unwrap();

    sdao.update_tab_order(surf.id, 5).unwrap();
    let fetched = sdao.get(surf.id).unwrap().unwrap();
    assert_eq!(fetched.tab_order, 5);
}

/// update_state_json 은 state_json 을 업데이트한다.
#[test]
fn surface_update_state_json() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/surface-json").unwrap();
    let pane = store
        .panes()
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();
    let sdao = store.surfaces();
    let surf = sdao
        .insert(&NewSurface {
            pane_id: pane.id,
            kind: SurfaceKind::Browser,
            state_json: None,
            tab_order: 0,
        })
        .unwrap();

    sdao.update_state_json(surf.id, Some("{\"url\":\"https://example.com\"}"))
        .unwrap();
    let fetched = sdao.get(surf.id).unwrap().unwrap();
    assert_eq!(
        fetched.state_json.as_deref(),
        Some("{\"url\":\"https://example.com\"}")
    );
}

/// surface 를 삭제하면 true, 존재하지 않으면 false 를 반환한다.
#[test]
fn surface_delete() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/surface-delete").unwrap();
    let pane = store
        .panes()
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();
    let sdao = store.surfaces();
    let surf = sdao
        .insert(&NewSurface {
            pane_id: pane.id,
            kind: SurfaceKind::FileTree,
            state_json: None,
            tab_order: 0,
        })
        .unwrap();

    assert!(sdao.delete(surf.id).unwrap());
    assert!(!sdao.delete(surf.id).unwrap());
    assert!(sdao.get(surf.id).unwrap().is_none());
}

// ── FK 연쇄 삭제 검증 ────────────────────────────────────────────────────────

/// 워크스페이스를 삭제하면 하위 pane 도 CASCADE 삭제된다.
#[test]
fn workspace_delete_cascades_panes() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/cascade-ws").unwrap();
    let dao = store.panes();
    let pane = dao
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();

    // 워크스페이스 물리 삭제
    {
        let guard = store.conn_for_test();
        guard
            .execute(
                "DELETE FROM workspaces WHERE id = ?1",
                rusqlite::params![ws_id],
            )
            .unwrap();
    }

    // pane 도 사라져야 한다
    assert!(dao.get(pane.id).unwrap().is_none());
}

/// pane 을 삭제하면 하위 surface 도 CASCADE 삭제된다.
#[test]
fn pane_delete_cascades_surfaces() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/cascade-pane").unwrap();
    let pane = store
        .panes()
        .insert(&NewPane {
            workspace_id: ws_id,
            parent_id: None,
            split: SplitKind::Leaf,
            ratio: 0.5,
        })
        .unwrap();
    let sdao = store.surfaces();
    let surf = sdao
        .insert(&NewSurface {
            pane_id: pane.id,
            kind: SurfaceKind::Terminal,
            state_json: None,
            tab_order: 0,
        })
        .unwrap();

    store.panes().delete(pane.id).unwrap();
    assert!(sdao.get(surf.id).unwrap().is_none());
}

// ── ratio CHECK 제약 검증 ─────────────────────────────────────────────────────

/// ratio < 0 은 CHECK 제약으로 거부된다.
#[test]
fn pane_ratio_negative_rejected() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/ratio-neg").unwrap();
    let result = store.panes().insert(&NewPane {
        workspace_id: ws_id,
        parent_id: None,
        split: SplitKind::Leaf,
        ratio: -0.1,
    });
    assert!(
        matches!(result, Err(StoreError::SqlError(_))),
        "ratio < 0 은 SqlError 여야 함"
    );
}

/// ratio > 1 은 CHECK 제약으로 거부된다.
#[test]
fn pane_ratio_over_one_rejected() {
    let store = Store::open_in_memory().unwrap();
    let ws_id = store.insert_workspace("/test/ratio-over").unwrap();
    let result = store.panes().insert(&NewPane {
        workspace_id: ws_id,
        parent_id: None,
        split: SplitKind::Leaf,
        ratio: 1.1,
    });
    assert!(
        matches!(result, Err(StoreError::SqlError(_))),
        "ratio > 1 은 SqlError 여야 함"
    );
}
