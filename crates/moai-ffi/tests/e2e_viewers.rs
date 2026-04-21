//! M2 E2E: workspace → pane split → tab → surface → command palette 흐름 (프로그래밍 방식).
//!
//! SPEC-M2-001 AC-8 통합 검증.

use moai_ffi::RustCore;

/// M2 전체 파이프라인: workspace 생성 → pane 생성 → surface 추가 → 탭 재배치 → 정리.
#[test]
fn m2_e2e_pane_tab_surface() {
    let core = RustCore::new();

    // 1. 워크스페이스 생성
    let ws_uuid = core.create_workspace("e2e-m2".into(), "/tmp".into());
    assert!(!ws_uuid.is_empty(), "워크스페이스 UUID가 비어있음");

    let ws_db_id = core.get_workspace_db_id(&ws_uuid);
    assert!(ws_db_id > 0, "워크스페이스 DB ID가 유효하지 않음");

    // 2. 루트 pane 생성 (leaf)
    let root = core.create_pane(ws_db_id, 0, "leaf".into(), 0.5);
    assert!(root > 0, "루트 pane 생성 실패");

    // 3. pane ratio 업데이트
    assert!(
        core.update_pane_ratio(root, 0.6),
        "pane ratio 업데이트 실패"
    );

    // 4. list_panes_json — 구조 확인
    let panes_json = core.list_panes_json(ws_db_id);
    assert!(!panes_json.is_empty(), "list_panes_json 결과가 비어있음");
    // JSON 배열 또는 유효한 JSON 구조 포함 확인
    assert!(
        panes_json.starts_with('[') || panes_json.starts_with('{'),
        "list_panes_json 결과가 유효한 JSON 아님: {panes_json}"
    );

    // 5. Terminal surface 생성
    let surf_term = core.create_surface(root, "terminal".into(), "".into(), 0);
    assert!(surf_term > 0, "Terminal surface 생성 실패");

    // 6. Markdown surface 생성
    let surf_md = core.create_surface(
        root,
        "markdown".into(),
        "{\"path\":\"/tmp/test.md\"}".into(),
        1,
    );
    assert!(surf_md > 0, "Markdown surface 생성 실패");

    // 7. list_surfaces_json — terminal + markdown 확인
    let surfaces_json = core.list_surfaces_json(root);
    assert!(
        surfaces_json.contains("terminal"),
        "list_surfaces_json에 terminal 없음: {surfaces_json}"
    );
    assert!(
        surfaces_json.contains("markdown"),
        "list_surfaces_json에 markdown 없음: {surfaces_json}"
    );

    // 8. 탭 재배치: markdown을 index 0으로
    assert!(
        core.update_surface_tab_order(surf_md, 0),
        "surface tab order 업데이트 실패"
    );

    // 9. FileTree surface 생성
    let surf_ft = core.create_surface(root, "filetree".into(), "".into(), 2);
    assert!(surf_ft > 0, "FileTree surface 생성 실패");

    // 10. 정리
    assert!(core.delete_surface(surf_ft), "FileTree surface 삭제 실패");
    assert!(core.delete_surface(surf_md), "Markdown surface 삭제 실패");
    assert!(core.delete_surface(surf_term), "Terminal surface 삭제 실패");
    assert!(core.delete_pane(root), "루트 pane 삭제 실패");
    assert!(core.delete_workspace(ws_uuid), "워크스페이스 삭제 실패");
}

/// M2 E2E: 분할 시뮬레이션 (수평 분할 — 두 child pane 생성).
#[test]
fn m2_e2e_pane_split_simulation() {
    let core = RustCore::new();

    let ws_uuid = core.create_workspace("e2e-split".into(), "/tmp".into());
    let ws_id = core.get_workspace_db_id(&ws_uuid);

    // 루트 pane (horizontal 분할 부모 역할)
    let root = core.create_pane(ws_id, 0, "horizontal".into(), 0.5);
    assert!(root > 0);

    // left child
    let left = core.create_pane(ws_id, root, "leaf".into(), 0.5);
    assert!(left > 0);

    // right child
    let right = core.create_pane(ws_id, root, "leaf".into(), 0.5);
    assert!(right > 0);

    // 각 child에 surface 생성
    let sl = core.create_surface(left, "terminal".into(), "".into(), 0);
    let sr = core.create_surface(right, "filetree".into(), "".into(), 0);
    assert!(sl > 0 && sr > 0);

    // 정리
    core.delete_surface(sl);
    core.delete_surface(sr);
    core.delete_pane(left);
    core.delete_pane(right);
    core.delete_pane(root);
    core.delete_workspace(ws_uuid);
}

/// M2 E2E: list_directory_json 기본 동작 확인.
#[test]
fn m2_e2e_filetree_list_tmp() {
    let core = RustCore::new();
    let json = core.list_directory_json("/tmp".into(), "".into());
    // /tmp 는 항상 존재하고 최소 빈 JSON 배열 반환
    assert!(
        json.starts_with('['),
        "list_directory_json 결과가 JSON 배열 아님: {json}"
    );
}
