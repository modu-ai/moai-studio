//! Pane FFI 함수 (SPEC-M2-001 RG-M2-1 산출물).
//!
//! Swift 측에서 pane binary tree 를 생성/조회/수정/삭제하는 FFI 진입점.
//! parent_id == 0 은 None (루트 pane) 으로 해석된다.

use moai_store::{NewPane, PaneStoreExt, SplitKind, Store};

use crate::ffi::PaneInfo;

/// Swift 에서 전달받은 split 문자열을 `SplitKind` 로 변환한다.
/// 알 수 없는 값은 Leaf 로 폴백한다.
fn parse_split(s: &str) -> SplitKind {
    match s {
        "horizontal" => SplitKind::Horizontal,
        "vertical" => SplitKind::Vertical,
        _ => SplitKind::Leaf,
    }
}

/// 새 pane 을 생성하고 id 를 반환한다. 오류 시 0.
///
/// `parent_id == 0` 은 루트 pane (부모 없음) 을 의미한다.
// @MX:ANCHOR: [AUTO] Swift → Rust pane 생성 FFI 진입점 (fan_in>=3)
// @MX:REASON: [AUTO] T-035 FFI, T-037 통합테스트, MS-2 PaneSplitView 세 경로에서 호출
pub(crate) fn create_pane(
    store: &Store,
    workspace_id: i64,
    parent_id: i64,
    split: String,
    ratio: f64,
) -> i64 {
    let dao = store.panes();
    let new = NewPane {
        workspace_id,
        parent_id: if parent_id == 0 {
            None
        } else {
            Some(parent_id)
        },
        split: parse_split(&split),
        ratio,
    };
    match dao.insert(&new) {
        Ok(row) => row.id,
        Err(_) => 0,
    }
}

/// 워크스페이스 내 pane 목록을 반환한다.
pub(crate) fn list_panes(store: &Store, workspace_id: i64) -> Vec<PaneInfo> {
    let dao = store.panes();
    match dao.list_by_workspace(workspace_id) {
        Ok(rows) => rows
            .into_iter()
            .map(|r| PaneInfo {
                id: r.id,
                workspace_id: r.workspace_id,
                parent_id: r.parent_id.unwrap_or(0),
                split: r.split.as_str().to_string(),
                ratio: r.ratio,
            })
            .collect(),
        Err(_) => vec![],
    }
}

/// pane 의 ratio 를 업데이트한다. 성공 시 true.
pub(crate) fn update_pane_ratio(store: &Store, pane_id: i64, ratio: f64) -> bool {
    store.panes().update_ratio(pane_id, ratio).is_ok()
}

/// pane 을 삭제한다. 삭제되면 true.
pub(crate) fn delete_pane(store: &Store, pane_id: i64) -> bool {
    store.panes().delete(pane_id).unwrap_or(false)
}

/// 워크스페이스 내 pane 목록을 JSON 문자열로 반환한다.
///
/// swift-bridge 0.1 의 Vectorizable 미생성 한계를 우회하여 Swift 측 Codable 로 파싱한다.
/// pane 이 없거나 오류 시 "[]" 를 반환한다.
// @MX:NOTE: [AUTO] MS-2 T-042 PaneTreeModel.load() 의 JSON 소스.
//           C-5 (Vectorizable workaround 제거) 해소 시 deprecated 처리.
pub(crate) fn list_panes_json(store: &Store, workspace_id: i64) -> String {
    let dao = store.panes();
    match dao.list_by_workspace(workspace_id) {
        Ok(rows) => {
            // PaneRow 를 직접 JSON 으로 직렬화 (serde_json 사용)
            let json_rows: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "workspace_id": r.workspace_id,
                        "parent_id": r.parent_id.unwrap_or(0),
                        "split": r.split.as_str(),
                        "ratio": r.ratio,
                    })
                })
                .collect();
            serde_json::to_string(&json_rows).unwrap_or_else(|_| "[]".to_string())
        }
        Err(_) => "[]".to_string(),
    }
}
