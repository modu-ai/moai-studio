//! Surface FFI 함수 (SPEC-M2-001 RG-M2-2 산출물).
//!
//! Swift 측에서 pane 내 surface(탭)를 생성/조회/수정/삭제하는 FFI 진입점.
//! state_json == "" 은 None 으로 해석된다.

use moai_store::{NewSurface, Store, SurfaceKind, SurfaceStoreExt};

use crate::ffi::SurfaceInfo;

/// Swift 에서 전달받은 kind 문자열을 `SurfaceKind` 로 변환한다.
/// 알 수 없는 값은 Terminal 로 폴백한다.
fn parse_kind(s: &str) -> SurfaceKind {
    match s {
        "terminal" => SurfaceKind::Terminal,
        "code" => SurfaceKind::Code,
        "markdown" => SurfaceKind::Markdown,
        "image" => SurfaceKind::Image,
        "browser" => SurfaceKind::Browser,
        "filetree" => SurfaceKind::FileTree,
        "agent_run" => SurfaceKind::AgentRun,
        "kanban" => SurfaceKind::Kanban,
        "memory" => SurfaceKind::Memory,
        "instructions_graph" => SurfaceKind::InstructionsGraph,
        _ => SurfaceKind::Terminal,
    }
}

/// 새 surface 를 생성하고 id 를 반환한다. 오류 시 0.
///
/// `state_json == ""` 은 빈 상태 (None) 를 의미한다.
// @MX:ANCHOR: [AUTO] Swift → Rust surface 생성 FFI 진입점 (fan_in>=3)
// @MX:REASON: [AUTO] T-036 FFI, T-037 통합테스트, MS-3 TabBarViewModel 세 경로에서 호출
pub(crate) fn create_surface(
    store: &Store,
    pane_id: i64,
    kind: String,
    state_json: String,
    tab_order: i64,
) -> i64 {
    let dao = store.surfaces();
    let new = NewSurface {
        pane_id,
        kind: parse_kind(&kind),
        state_json: if state_json.is_empty() {
            None
        } else {
            Some(state_json)
        },
        tab_order,
    };
    match dao.insert(&new) {
        Ok(row) => row.id,
        Err(_) => 0,
    }
}

/// pane 내 surface 목록을 tab_order 오름차순으로 반환한다.
pub(crate) fn list_surfaces(store: &Store, pane_id: i64) -> Vec<SurfaceInfo> {
    let dao = store.surfaces();
    match dao.list_by_pane(pane_id) {
        Ok(rows) => rows
            .into_iter()
            .map(|r| SurfaceInfo {
                id: r.id,
                pane_id: r.pane_id,
                kind: r.kind.as_str().to_string(),
                state_json: r.state_json.unwrap_or_default(),
                tab_order: r.tab_order,
            })
            .collect(),
        Err(_) => vec![],
    }
}

/// surface 의 tab_order 를 업데이트한다. 성공 시 true.
pub(crate) fn update_surface_tab_order(store: &Store, surface_id: i64, tab_order: i64) -> bool {
    store
        .surfaces()
        .update_tab_order(surface_id, tab_order)
        .is_ok()
}

/// surface 를 삭제한다. 삭제되면 true.
pub(crate) fn delete_surface(store: &Store, surface_id: i64) -> bool {
    store.surfaces().delete(surface_id).unwrap_or(false)
}

/// pane 내 surface 목록을 JSON 문자열로 반환한다.
///
/// swift-bridge 0.1 의 Vectorizable 미생성 한계를 우회하여 Swift 측 Codable 로 파싱한다.
/// surface 가 없거나 오류 시 "[]" 를 반환한다.
// @MX:NOTE: [AUTO] MS-3 T-047 TabBarViewModel.load() 의 JSON 소스.
//           C-5 (Vectorizable workaround 제거) 해소 시 deprecated 처리.
pub(crate) fn list_surfaces_json(store: &Store, pane_id: i64) -> String {
    let dao = store.surfaces();
    match dao.list_by_pane(pane_id) {
        Ok(rows) => {
            let json_rows: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "pane_id": r.pane_id,
                        "kind": r.kind.as_str(),
                        "state_json": r.state_json.unwrap_or_default(),
                        "tab_order": r.tab_order,
                    })
                })
                .collect();
            serde_json::to_string(&json_rows).unwrap_or_else(|_| "[]".to_string())
        }
        Err(_) => "[]".to_string(),
    }
}
