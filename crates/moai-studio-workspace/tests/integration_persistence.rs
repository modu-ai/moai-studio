//! T13 통합 테스트 — Persistence e2e
//!
//! AC-P-12 e2e: shutdown_save_then_restart_restores_layout
//! AC-P-14 e2e: cwd_deleted_between_runs_falls_back_to_home

use moai_studio_workspace::persistence::{
    load_panes, resolve_cwd_with_fallback, save_panes, PaneLayoutV1, PaneTreeSnapshotV1,
    TabSnapshotV1, SCHEMA_VERSION,
};
use moai_studio_workspace::panes_convert::{
    layout_v1_to_tab_inputs, tab_container_to_layout_v1, PaneTreeInput, TabSnapshotInput,
};
use std::path::PathBuf;

// ---- 헬퍼 ----

fn temp_layout_path(suffix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("moai-t13-e2e-{}.json", suffix))
}

/// AC-P-12 e2e: 종료 시 save → 재시작 시 load → 구조 동일 검증.
///
/// 2개 탭 × 각 Split 트리(Leaf + Leaf) 구성, cwd 포함.
#[test]
fn shutdown_save_then_restart_restores_layout() {
    let path = temp_layout_path("save-restore");
    let _ = std::fs::remove_file(&path);

    // --- "shutdown" 단계: layout 구성 후 저장 ---
    let layout = PaneLayoutV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        tabs: vec![
            TabSnapshotV1 {
                id: "tab-1".to_string(),
                title: "Main".to_string(),
                last_focused_pane: Some("pane-a1".to_string()),
                pane_tree: PaneTreeSnapshotV1::Split {
                    id: "split-1".to_string(),
                    direction: "horizontal".to_string(),
                    ratio: 0.5,
                    first: Box::new(PaneTreeSnapshotV1::Leaf {
                        id: "pane-a1".to_string(),
                        cwd: Some("/tmp".to_string()),
                    }),
                    second: Box::new(PaneTreeSnapshotV1::Leaf {
                        id: "pane-a2".to_string(),
                        cwd: Some("/home".to_string()),
                    }),
                },
            },
            TabSnapshotV1 {
                id: "tab-2".to_string(),
                title: "Second".to_string(),
                last_focused_pane: None,
                pane_tree: PaneTreeSnapshotV1::Split {
                    id: "split-2".to_string(),
                    direction: "vertical".to_string(),
                    ratio: 0.6,
                    first: Box::new(PaneTreeSnapshotV1::Leaf {
                        id: "pane-b1".to_string(),
                        cwd: None,
                    }),
                    second: Box::new(PaneTreeSnapshotV1::Leaf {
                        id: "pane-b2".to_string(),
                        cwd: Some("/var".to_string()),
                    }),
                },
            },
        ],
    };

    save_panes(&path, &layout).expect("save_panes 성공");
    assert!(path.exists(), "저장 후 파일 존재");

    // --- "restart" 단계: 파일에서 로드 ---
    let restored = load_panes(&path).expect("load_panes 성공");

    // 구조 완전 동일 검증
    assert_eq!(layout, restored, "save → load 후 레이아웃 동일");

    // 탭 수 검증
    assert_eq!(restored.tabs.len(), 2, "탭 2개 복원");

    // 탭-1 상세 검증
    let t1 = &restored.tabs[0];
    assert_eq!(t1.id, "tab-1");
    assert_eq!(t1.title, "Main");
    assert_eq!(t1.last_focused_pane, Some("pane-a1".to_string()));
    match &t1.pane_tree {
        PaneTreeSnapshotV1::Split {
            direction, ratio, ..
        } => {
            assert_eq!(direction, "horizontal");
            assert!((ratio - 0.5).abs() < f32::EPSILON);
        }
        _ => panic!("tab-1 root should be Split"),
    }

    // 탭-2 상세 검증
    let t2 = &restored.tabs[1];
    assert_eq!(t2.id, "tab-2");
    assert_eq!(t2.last_focused_pane, None);
    match &t2.pane_tree {
        PaneTreeSnapshotV1::Split {
            direction, ratio, ..
        } => {
            assert_eq!(direction, "vertical");
            assert!((ratio - 0.6).abs() < f32::EPSILON);
        }
        _ => panic!("tab-2 root should be Split"),
    }

    let _ = std::fs::remove_file(&path);
}

/// AC-P-14 e2e: 재시작 시 저장된 cwd 가 삭제돼 있으면 $HOME 으로 fallback.
///
/// 임시 디렉토리를 생성 → cwd 로 저장 → 디렉토리 삭제 → 로드 → fallback 확인.
#[test]
fn cwd_deleted_between_runs_falls_back_to_home() {
    let path = temp_layout_path("cwd-fallback");
    let _ = std::fs::remove_file(&path);

    // 임시 cwd 디렉토리 생성
    let tmp_cwd = std::env::temp_dir().join("moai-t13-cwd-to-delete");
    std::fs::create_dir_all(&tmp_cwd).expect("임시 cwd 생성");
    let cwd_str = tmp_cwd.to_str().expect("UTF-8 경로").to_string();

    // layout 저장: cwd 를 존재하는 임시 디렉토리로 설정
    let layout = PaneLayoutV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        tabs: vec![TabSnapshotV1 {
            id: "tab-cwd-test".to_string(),
            title: "CWD Test".to_string(),
            last_focused_pane: Some("pane-c1".to_string()),
            pane_tree: PaneTreeSnapshotV1::Split {
                id: "split-cwd".to_string(),
                direction: "horizontal".to_string(),
                ratio: 0.5,
                first: Box::new(PaneTreeSnapshotV1::Leaf {
                    id: "pane-c1".to_string(),
                    cwd: Some(cwd_str.clone()),
                }),
                second: Box::new(PaneTreeSnapshotV1::Leaf {
                    id: "pane-c2".to_string(),
                    cwd: None,
                }),
            },
        }],
    };

    save_panes(&path, &layout).expect("save 성공");

    // cwd 디렉토리 삭제 (재시작 사이에 삭제된 상황 시뮬레이션)
    std::fs::remove_dir_all(&tmp_cwd).expect("임시 cwd 삭제");
    assert!(!tmp_cwd.exists(), "cwd 삭제 확인");

    // 재시작: 로드
    let restored = load_panes(&path).expect("load 성공");
    assert_eq!(restored.tabs.len(), 1);

    // pane-c1 의 cwd 는 삭제됐으므로 $HOME 으로 fallback
    let expected_home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME 환경변수 필요");

    // 로드된 레이아웃에서 pane-c1 의 cwd 를 추출
    let pane_c1_cwd = match &restored.tabs[0].pane_tree {
        PaneTreeSnapshotV1::Split { first, .. } => match first.as_ref() {
            PaneTreeSnapshotV1::Leaf { cwd, .. } => cwd.as_deref().map(PathBuf::from),
            _ => None,
        },
        PaneTreeSnapshotV1::Leaf { cwd, .. } => cwd.as_deref().map(PathBuf::from),
    };

    // resolve_cwd_with_fallback 적용 — 삭제된 경로면 $HOME 반환
    let resolved = resolve_cwd_with_fallback(pane_c1_cwd.as_deref());
    assert_eq!(
        resolved, expected_home,
        "삭제된 cwd → $HOME fallback 확인"
    );

    // pane-c2 (cwd=None) 도 $HOME fallback
    let pane_c2_resolved = resolve_cwd_with_fallback(None);
    assert_eq!(pane_c2_resolved, expected_home, "None cwd → $HOME fallback");

    let _ = std::fs::remove_file(&path);
}
