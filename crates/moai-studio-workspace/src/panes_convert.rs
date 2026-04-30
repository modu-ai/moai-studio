//! T13 변환 레이어 — `moai-studio-ui` 의존 없이 순수 DTO 변환.
//!
//! ## 설계 원칙
//! - `moai-studio-workspace` 가 `moai-studio-ui` 에 의존하면 순환 의존성 발생.
//!   따라서 이 모듈은 중립적인 DTO(`TabSnapshotInput`, `PaneTreeInput`)를 정의하고,
//!   UI 레이어는 자체 타입을 이 DTO 로 변환한 뒤 변환 함수에 전달한다.
//! - `PaneLayoutV1` ↔ DTO 변환은 순수 함수(no side effects).

use crate::persistence::{PaneLayoutV1, PaneTreeSnapshotV1, SCHEMA_VERSION, TabSnapshotV1};
use std::path::PathBuf;

// ============================================================
// DTO 타입 정의
// ============================================================

/// UI 레이어에서 workspace 레이어로 전달하는 탭 입력 DTO.
///
/// `moai-studio-ui` 의 `Tab` / `TabContainer` 를 직접 참조하지 않으므로
/// 순환 의존성 없이 변환 함수를 호출할 수 있다.
#[derive(Debug, Clone, PartialEq)]
pub struct TabSnapshotInput {
    /// 탭 식별자 문자열.
    pub id: String,
    /// 탭 타이틀.
    pub title: String,
    /// 마지막 포커스된 pane ID (없으면 None).
    pub last_focused_pane: Option<String>,
    /// Pane 트리 루트.
    pub pane_tree: PaneTreeInput,
}

/// Pane 트리 노드 DTO.
#[derive(Debug, Clone, PartialEq)]
pub enum PaneTreeInput {
    /// 단말 pane.
    Leaf {
        /// Pane 식별자.
        id: String,
        /// 작업 디렉토리 경로 (없으면 None).
        cwd: Option<PathBuf>,
    },
    /// 분할 노드.
    Split {
        /// SplitNode 식별자.
        id: String,
        /// 분할 방향.
        direction: SplitDirectionInput,
        /// 첫 번째 자식 비율 (0.0 ~ 1.0).
        ratio: f32,
        /// 첫 번째 자식.
        first: Box<PaneTreeInput>,
        /// 두 번째 자식.
        second: Box<PaneTreeInput>,
    },
}

/// 분할 방향 DTO.
///
/// @MX:NOTE: [AUTO] split-direction-string-codec
/// persistence 레이어는 "horizontal" / "vertical" 문자열로 저장하므로
/// `From<&str>` + `Display` 구현으로 codec 책임을 중앙화한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitDirectionInput {
    Horizontal,
    Vertical,
}

impl std::fmt::Display for SplitDirectionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplitDirectionInput::Horizontal => write!(f, "horizontal"),
            SplitDirectionInput::Vertical => write!(f, "vertical"),
        }
    }
}

impl From<&str> for SplitDirectionInput {
    /// "horizontal" / "vertical" 이외의 문자열은 `Horizontal` 로 fallback.
    fn from(s: &str) -> Self {
        match s {
            "vertical" => SplitDirectionInput::Vertical,
            _ => SplitDirectionInput::Horizontal,
        }
    }
}

// ============================================================
// 변환 함수
// ============================================================

// ============================================================
// @MX:ANCHOR: [AUTO] restore-on-startup
// @MX:REASON: session 시작 시 사용자 layout 복원 진입점.
//             fan_in 향후: main 호출 + persistence 모듈 + 통합 테스트 (≥ 3).
//             이 함수 시그니처 변경 시 모든 호출자 동시 업데이트 필요.
// ============================================================

/// `TabSnapshotInput` 슬라이스를 `PaneLayoutV1` 으로 변환한다 (UI → 저장).
///
/// UI 레이어에서 현재 `TabContainer` 상태를 DTO 로 변환한 뒤 이 함수에 전달하면
/// persistence 레이어에 저장 가능한 `PaneLayoutV1` 을 반환한다.
///
/// @MX:NOTE: [AUTO] tab-container-to-layout-v1
/// fan_in ≥ 3 예정: save_panes_on_shutdown / 통합 테스트 / 미래 UI 레이어.
pub fn tab_container_to_layout_v1(tabs: &[TabSnapshotInput]) -> PaneLayoutV1 {
    let tab_snapshots: Vec<TabSnapshotV1> = tabs.iter().map(tab_input_to_snapshot).collect();
    PaneLayoutV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        active_tab_idx: 0,
        tabs: tab_snapshots,
    }
}

/// `TabSnapshotInput` 슬라이스와 `active_tab_idx` 를 `PaneLayoutV1` 으로 변환한다 (UI → 저장).
///
/// `active_tab_idx` 를 포함하는 전체 레이아웃 스냅샷을 생성한다.
/// fan_in >= 3: shutdown save handler, integration tests, TabContainer::into_snapshot_full.
pub fn tab_container_to_layout_v1_with_active(
    tabs: &[TabSnapshotInput],
    active_tab_idx: usize,
) -> PaneLayoutV1 {
    let tab_snapshots: Vec<TabSnapshotV1> = tabs.iter().map(tab_input_to_snapshot).collect();
    PaneLayoutV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        active_tab_idx,
        tabs: tab_snapshots,
    }
}

/// `PaneLayoutV1` 을 `TabSnapshotInput` 벡터로 변환한다 (저장 → UI).
///
/// 앱 시작 시 `load_panes` 로 읽은 레이아웃을 UI 레이어가 처리할 수 있는
/// DTO 형태로 변환한다. UI 레이어는 이 DTO 를 이용해 `TabContainer` 를 복원한다.
///
/// @MX:NOTE: [AUTO] layout-v1-to-tab-inputs
/// restore_panes_on_startup 에서 호출됨. 미래 TabContainer::from_snapshot 연동 예정.
pub fn layout_v1_to_tab_inputs(layout: &PaneLayoutV1) -> Vec<TabSnapshotInput> {
    layout.tabs.iter().map(snapshot_to_tab_input).collect()
}

// ============================================================
// 내부 헬퍼
// ============================================================

fn tab_input_to_snapshot(tab: &TabSnapshotInput) -> TabSnapshotV1 {
    TabSnapshotV1 {
        id: tab.id.clone(),
        title: tab.title.clone(),
        last_focused_pane: tab.last_focused_pane.clone(),
        pane_tree: pane_tree_input_to_snapshot(&tab.pane_tree),
    }
}

fn snapshot_to_tab_input(snap: &TabSnapshotV1) -> TabSnapshotInput {
    TabSnapshotInput {
        id: snap.id.clone(),
        title: snap.title.clone(),
        last_focused_pane: snap.last_focused_pane.clone(),
        pane_tree: pane_tree_snapshot_to_input(&snap.pane_tree),
    }
}

fn pane_tree_input_to_snapshot(node: &PaneTreeInput) -> PaneTreeSnapshotV1 {
    match node {
        PaneTreeInput::Leaf { id, cwd } => PaneTreeSnapshotV1::Leaf {
            id: id.clone(),
            cwd: cwd.as_ref().and_then(|p| p.to_str()).map(String::from),
        },
        PaneTreeInput::Split {
            id,
            direction,
            ratio,
            first,
            second,
        } => PaneTreeSnapshotV1::Split {
            id: id.clone(),
            direction: direction.to_string(),
            ratio: *ratio,
            first: Box::new(pane_tree_input_to_snapshot(first)),
            second: Box::new(pane_tree_input_to_snapshot(second)),
        },
    }
}

fn pane_tree_snapshot_to_input(node: &PaneTreeSnapshotV1) -> PaneTreeInput {
    match node {
        PaneTreeSnapshotV1::Leaf { id, cwd } => PaneTreeInput::Leaf {
            id: id.clone(),
            cwd: cwd.as_ref().map(PathBuf::from),
        },
        PaneTreeSnapshotV1::Split {
            id,
            direction,
            ratio,
            first,
            second,
        } => PaneTreeInput::Split {
            id: id.clone(),
            direction: SplitDirectionInput::from(direction.as_str()),
            ratio: *ratio,
            first: Box::new(pane_tree_snapshot_to_input(first)),
            second: Box::new(pane_tree_snapshot_to_input(second)),
        },
    }
}

// ============================================================
// @MX:NOTE: [AUTO] snapshot-path-convention
// @MX:SPEC: REQ-P-052
// snapshot_path 는 `~/.moai/studio/panes-{workspace_id}.json` 을 반환한다.
// workspace_id 는 WorkspacesStore 의 ws.id (형식: "ws-{hex_nanos}") 를 사용한다.
// ============================================================

/// workspace_id 에 대응하는 pane snapshot 파일 경로를 반환한다 (REQ-P-052).
///
/// 반환 경로: `~/.moai/studio/panes-{workspace_id}.json`
///
/// `dirs` crate 미사용 — `std::env::var("HOME")` 또는 `"/"` fallback.
pub fn snapshot_path(workspace_id: &str) -> std::path::PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));
    home.join(".moai")
        .join("studio")
        .join(format!("panes-{}.json", workspace_id))
}

// ============================================================
// 유닛 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tab_input(id: &str, pane_id: &str, cwd: Option<&str>) -> TabSnapshotInput {
        TabSnapshotInput {
            id: id.to_string(),
            title: id.to_string(),
            last_focused_pane: Some(pane_id.to_string()),
            pane_tree: PaneTreeInput::Leaf {
                id: pane_id.to_string(),
                cwd: cwd.map(PathBuf::from),
            },
        }
    }

    /// SplitDirectionInput Display 코덱 검증.
    #[test]
    fn split_direction_display_roundtrip() {
        assert_eq!(SplitDirectionInput::Horizontal.to_string(), "horizontal");
        assert_eq!(SplitDirectionInput::Vertical.to_string(), "vertical");
        assert_eq!(
            SplitDirectionInput::from("horizontal"),
            SplitDirectionInput::Horizontal
        );
        assert_eq!(
            SplitDirectionInput::from("vertical"),
            SplitDirectionInput::Vertical
        );
        // 알 수 없는 값 → Horizontal fallback
        assert_eq!(
            SplitDirectionInput::from("unknown"),
            SplitDirectionInput::Horizontal
        );
    }

    /// snapshot_path: 예상 경로 형식 검증 (REQ-P-052).
    #[test]
    fn snapshot_path_format() {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        let expected = home
            .join(".moai")
            .join("studio")
            .join("panes-ws-abc123.json");
        assert_eq!(snapshot_path("ws-abc123"), expected);
    }

    /// tab_container_to_layout_v1 + layout_v1_to_tab_inputs 왕복 검증.
    #[test]
    fn convert_roundtrip_leaf() {
        let inputs = vec![
            make_tab_input("tab-1", "pane-1", Some("/home/user")),
            make_tab_input("tab-2", "pane-2", None),
        ];

        let layout = tab_container_to_layout_v1(&inputs);
        assert_eq!(layout.schema_version, SCHEMA_VERSION);
        assert_eq!(layout.tabs.len(), 2);

        let restored = layout_v1_to_tab_inputs(&layout);
        assert_eq!(restored, inputs, "왕복 변환 후 DTO 동일");
    }

    /// Split 트리 왕복 검증.
    #[test]
    fn convert_roundtrip_split() {
        let inputs = vec![TabSnapshotInput {
            id: "tab-s".to_string(),
            title: "Split Tab".to_string(),
            last_focused_pane: Some("p-left".to_string()),
            pane_tree: PaneTreeInput::Split {
                id: "split-root".to_string(),
                direction: SplitDirectionInput::Vertical,
                ratio: 0.4,
                first: Box::new(PaneTreeInput::Leaf {
                    id: "p-left".to_string(),
                    cwd: Some(PathBuf::from("/tmp")),
                }),
                second: Box::new(PaneTreeInput::Leaf {
                    id: "p-right".to_string(),
                    cwd: None,
                }),
            },
        }];

        let layout = tab_container_to_layout_v1(&inputs);
        let restored = layout_v1_to_tab_inputs(&layout);
        assert_eq!(restored, inputs, "Split 왕복 변환 후 DTO 동일");
    }
}
