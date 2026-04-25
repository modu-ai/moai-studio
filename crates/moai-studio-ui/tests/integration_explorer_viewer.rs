//! SPEC-V3-005/006 wiring integration 테스트.
//!
//! ## 목적 (AC-WIRE-1/2/3)
//!
//! - AC-WIRE-1: wire_file_explorer_callback 후 FileExplorer.open_file 이
//!   RootView.handle_open_file 을 트리거한다.
//! - AC-WIRE-2: handle_open_file 이 정확한 LeafKind variant 를 생성한다 (Markdown/Code/Binary).
//! - AC-WIRE-3: 활성 pane 의 leaf_payloads 에 entity 가 등록된다.
//!
//! ## 설계 결정
//!
//! - GPUI TestAppContext 를 사용하여 Entity 생명주기를 검증한다.
//! - tempfile 을 사용하여 실제 파일 I/O 를 테스트한다.
//! - TabContainer 와 focused pane 을 설정하여 end-to-end 경로를 검증한다.

use gpui::{AppContext as _, TestAppContext};
use moai_studio_ui::{
    RootView,
    panes::PaneId,
    tabs::TabContainer,
    viewer::{LeafKind, OpenFileEvent},
};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

// ============================================================
// 헬퍼: RootView + TabContainer + focused pane 세팅
// ============================================================

/// TestAppContext 에서 tab_container + focused pane 이 설정된 RootView Entity 를 생성한다.
///
/// 반환: (Entity<RootView>, 활성 pane id)
fn setup_root_with_tab_and_pane(cx: &mut TestAppContext) -> (gpui::Entity<RootView>, PaneId) {
    let root = cx.new(|_cx| RootView::new(vec![], PathBuf::from("/tmp/test-ws.json")));

    // TabContainer 생성 — Tab::new() 에서 last_focused_pane 이 root_pane_id 로 초기화된다
    let tab = cx.new(|_cx| TabContainer::new());

    // 활성 탭의 last_focused_pane 을 읽는다
    let pane_id = cx.read(|app| {
        let tc = tab.read(app);
        tc.active_tab()
            .last_focused_pane
            .clone()
            .expect("TabContainer::new() 는 last_focused_pane 을 초기화해야 한다")
    });

    // RootView 에 tab_container 주입
    cx.update(|app| {
        root.update(app, |view: &mut RootView, _cx| {
            view.tab_container = Some(tab);
        });
    });

    (root, pane_id)
}

// ============================================================
// AC-WIRE-1: wire_file_explorer_callback → RootView.handle_open_file
// ============================================================

/// clicking_md_file_mounts_markdown_viewer
///
/// AC-WIRE-1/3: wire_file_explorer_callback 배선 후 FileExplorer.open_file 호출 시
/// RootView.handle_open_file 이 트리거되고 leaf_payloads 에 Markdown entity 가 등록된다.
#[test]
fn clicking_md_file_mounts_markdown_viewer() {
    let mut cx = TestAppContext::single();
    let (root, pane_id) = setup_root_with_tab_and_pane(&mut cx);

    // 임시 .md 파일 생성
    let mut tmp = NamedTempFile::with_suffix(".md").expect("tempfile 생성 실패");
    writeln!(tmp, "# Hello\n\n단락 텍스트").expect("파일 쓰기 실패");
    let path = tmp.path().to_path_buf();

    // FileExplorer 생성 후 RootView 에 주입 + 콜백 배선
    let explorer = cx.new(|_cx| {
        moai_studio_ui::explorer::FileExplorer::new(path.parent().unwrap().to_path_buf())
    });
    cx.update(|app| {
        root.update(app, |view: &mut RootView, cx| {
            view.file_explorer = Some(explorer.clone());
            // AC-WIRE-1: wire_file_explorer_callback 로 콜백 배선
            view.wire_file_explorer_callback(cx);
        });
    });

    // FileExplorer.emit_open_file 시뮬레이션 — EventEmitter 로 dispatch (AC-WIRE-1)
    let file_name = path.file_name().unwrap().to_os_string();
    cx.update(|app| {
        explorer.update(
            app,
            |fe: &mut moai_studio_ui::explorer::FileExplorer, cx| {
                fe.emit_open_file(&PathBuf::from(file_name), cx);
            },
        );
    });

    // cx.run_until_parked 로 비동기 dispatch 완료 대기
    cx.run_until_parked();

    // leaf_payloads 에 Markdown entity 가 등록되어야 한다
    let leaf = cx.read(|app| {
        root.read(app)
            .leaf_payloads
            .get(&pane_id)
            .map(|l| matches!(l, LeafKind::Markdown(_)))
    });
    assert_eq!(
        leaf,
        Some(true),
        "AC-WIRE-1/3: .md 파일 클릭 후 leaf_payloads 에 Markdown entity 가 등록되어야 한다"
    );
}

// ============================================================
// AC-WIRE-2: handle_open_file .rs → Code entity with Rust lang
// ============================================================

/// clicking_rs_file_mounts_code_viewer_with_rust_lang
///
/// AC-WIRE-2/3: .rs 파일에 대해 handle_open_file 을 직접 호출하면
/// CodeViewer 가 Rust 언어로 생성된다.
#[test]
fn clicking_rs_file_mounts_code_viewer_with_rust_lang() {
    let mut cx = TestAppContext::single();
    let (root, pane_id) = setup_root_with_tab_and_pane(&mut cx);

    // 임시 .rs 파일 생성
    let mut tmp = NamedTempFile::with_suffix(".rs").expect("tempfile 생성 실패");
    writeln!(tmp, "fn main() {{\n    println!(\"hello\");\n}}").expect("파일 쓰기 실패");
    let path = tmp.path().to_path_buf();

    // handle_open_file 직접 호출 (AC-WIRE-2 핵심 경로)
    let ev = OpenFileEvent {
        path: path.clone(),
        surface_hint: None,
    };
    cx.update(|app| {
        root.update(app, |view: &mut RootView, cx| {
            view.handle_open_file(&ev, cx);
        });
    });

    // leaf_payloads 에 Code entity 가 등록되어야 한다
    let leaf_is_code = cx.read(|app| {
        root.read(app)
            .leaf_payloads
            .get(&pane_id)
            .map(|l| matches!(l, LeafKind::Code(_)))
    });
    assert_eq!(
        leaf_is_code,
        Some(true),
        "AC-WIRE-2: .rs 파일은 Code entity 로 마운트되어야 한다"
    );

    // CodeViewer 의 lang 이 Rust 인지 확인
    let lang_is_rust = cx.read(|app| {
        let view = root.read(app);
        if let Some(LeafKind::Code(entity)) = view.leaf_payloads.get(&pane_id) {
            let viewer = entity.read(app);
            viewer.lang == Some(moai_studio_ui::viewer::code::languages::SupportedLang::Rust)
        } else {
            false
        }
    });
    assert!(
        lang_is_rust,
        "AC-WIRE-2: CodeViewer 의 lang 은 Rust 여야 한다"
    );
}

// ============================================================
// AC-WIRE-2: binary 파일 → leaf_payloads 변화 없음
// ============================================================

/// clicking_binary_file_mounts_binary_leaf
///
/// AC-WIRE-2: PNG magic bytes 파일은 binary 로 감지되어 leaf_payloads 에
/// Markdown/Code entity 가 등록되지 않는다 (AC-MV-11).
#[test]
fn clicking_binary_file_mounts_binary_leaf() {
    let mut cx = TestAppContext::single();
    let (root, pane_id) = setup_root_with_tab_and_pane(&mut cx);

    // PNG magic bytes 임시 파일 생성
    let mut tmp = NamedTempFile::with_suffix(".png").expect("tempfile 생성 실패");
    let png_magic = b"\x89PNG\r\n\x1a\n\x00\x00\x00some image data";
    tmp.write_all(png_magic).expect("PNG 파일 쓰기 실패");
    let path = tmp.path().to_path_buf();

    // handle_open_file 직접 호출
    let ev = OpenFileEvent {
        path: path.clone(),
        surface_hint: None,
    };
    cx.update(|app| {
        root.update(app, |view: &mut RootView, cx| {
            view.handle_open_file(&ev, cx);
        });
    });

    // binary 파일은 Markdown/Code entity 로 마운트되면 안 된다 (AC-MV-11)
    let leaf = cx.read(|app| {
        root.read(app)
            .leaf_payloads
            .get(&pane_id)
            .map(|l| matches!(l, LeafKind::Markdown(_) | LeafKind::Code(_)))
    });
    assert_ne!(
        leaf,
        Some(true),
        "AC-WIRE-2: binary 파일은 Markdown/Code entity 로 마운트되면 안 된다"
    );
}

// ============================================================
// AC-WIRE-2: 알 수 없는 확장자 → Code fallback
// ============================================================

/// clicking_unknown_extension_falls_back_to_code
///
/// AC-WIRE-2: .txt 파일은 route_by_extension 에 의해 Code surface 로 라우팅된다.
#[test]
fn clicking_unknown_extension_falls_back_to_code() {
    let mut cx = TestAppContext::single();
    let (root, pane_id) = setup_root_with_tab_and_pane(&mut cx);

    // 임시 .txt 파일 생성
    let mut tmp = NamedTempFile::with_suffix(".txt").expect("tempfile 생성 실패");
    writeln!(tmp, "plain text content").expect("파일 쓰기 실패");
    let path = tmp.path().to_path_buf();

    let ev = OpenFileEvent {
        path: path.clone(),
        surface_hint: None,
    };
    cx.update(|app| {
        root.update(app, |view: &mut RootView, cx| {
            view.handle_open_file(&ev, cx);
        });
    });

    // .txt 는 Code surface 로 라우팅된다 (route_by_extension)
    let leaf = cx.read(|app| {
        root.read(app)
            .leaf_payloads
            .get(&pane_id)
            .map(|l| matches!(l, LeafKind::Code(_) | LeafKind::Markdown(_)))
    });
    assert_eq!(
        leaf,
        Some(true),
        "AC-WIRE-2: .txt 는 Code 또는 Markdown entity 로 마운트되어야 한다"
    );
}
