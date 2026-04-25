// @MX:ANCHOR: [AUTO] file-explorer-entity
// @MX:REASON: [AUTO] SPEC-V3-005 RG-FE-1 REQ-FE-005. FileExplorer 는 RootView::file_explorer 의
//   진입점이며 tree / watch (MS-2) / git_status (MS-3) / menu (MS-3) / dnd (MS-3) / search (MS-3)
//   의 mutation 이 모두 이 Entity 로 수렴한다.
//   fan_in >= 4: RootView (T4), watch (MS-2), menu (MS-3), search (MS-3).
// @MX:TODO(MS-2-watch): expand_dir 비동기 ChildState Loading → Loaded 전이 미구현 (MS-2 T5 책임)
// @MX:TODO(MS-3-menu-search-dnd): context menu / DnD / search input (MS-3 범위)
// @MX:SPEC: SPEC-V3-005

use std::path::PathBuf;

use gpui::{
    Context, EventEmitter, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};

use super::path::normalize_for_display;
#[cfg(test)]
use super::tree::ChildState;
use super::tree::FsNode;
use super::watch::FsDelta;

// ============================================================
// FileOpenEvent — 파일 열기 이벤트 (GPUI EventEmitter 패턴)
// ============================================================

/// FileExplorer 가 파일 열기를 요청할 때 emit 하는 이벤트.
///
/// SPEC-V3-005 → SPEC-V3-006 wiring: RootView 가 subscribe 하여
/// handle_open_file 을 트리거한다 (AC-WIRE-1).
#[derive(Debug, Clone)]
pub struct FileOpenEvent {
    /// 열 파일의 절대 경로
    pub abs_path: PathBuf,
}

/// FileExplorer 는 FileOpenEvent 를 emit 한다.
impl EventEmitter<FileOpenEvent> for FileExplorer {}

// ============================================================
// FileExplorer struct
// ============================================================

/// File Explorer Entity — sidebar 좌측 파일 트리 표시 (REQ-FE-005).
///
/// MS-1 에서는 placeholder render 만 제공한다.
/// MS-2 에서 watch/debounce 가 추가되고, MS-3 에서 context menu/DnD/search 가 추가된다.
pub struct FileExplorer {
    /// 워크스페이스 루트 절대 경로
    pub workspace_root: PathBuf,
    /// 트리 루트 노드 (Dir::NotLoaded 초기 상태)
    pub tree: FsNode,
    /// fuzzy 검색 쿼리 (빈 문자열 = 전체 표시)
    pub search_query: String,
    /// 파일 행 클릭 시 콜백 — (rel_path, abs_path) 전달 (REQ-FE-005)
    pub on_file_open: Option<Box<dyn Fn(PathBuf, PathBuf) + 'static>>,
}

impl FileExplorer {
    /// 주어진 워크스페이스 루트로 FileExplorer 를 생성한다.
    /// tree 는 Dir::NotLoaded 초기 상태이며 on_file_open 은 None.
    pub fn new(workspace_root: PathBuf) -> Self {
        let root_name = workspace_root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| normalize_for_display(&workspace_root));

        let tree = FsNode::dir_unloaded(PathBuf::from(""), root_name);

        Self {
            workspace_root,
            tree,
            search_query: String::new(),
            on_file_open: None,
        }
    }

    /// 파일 열기 콜백을 등록한다.
    pub fn set_on_file_open<F: Fn(PathBuf, PathBuf) + 'static>(&mut self, cb: F) {
        self.on_file_open = Some(Box::new(cb));
    }

    /// Dir 노드를 펼친다. MS-1 에서는 is_expanded 플래그만 전환하며
    /// 실제 비동기 read_dir 는 MS-2 T5 에서 구현한다.
    ///
    /// @MX:TODO(MS-2-watch): ChildState NotLoaded → Loading → Loaded 전이 추가 필요
    pub fn expand_dir(&mut self, _rel_path: &PathBuf, _cx: &mut Context<Self>) {
        // MS-2 에서 구현: ChildState::NotLoaded → Loading 전이 + 비동기 read_dir 시작
    }

    // @MX:ANCHOR: [AUTO] fs-delta-apply
    // @MX:REASON: [AUTO] tree state 변형의 단일 경로. fan_in >= 3: watch_loop, manual refresh,
    //   통합 테스트. FsDelta 수신 후 영향받은 디렉토리 자식을 NotLoaded 로 초기화하고
    //   cx.notify() 로 GPUI 재렌더를 트리거한다.
    // @MX:SPEC: SPEC-V3-005
    /// FsDelta 를 트리에 적용한다 (AC-FE-5/6).
    ///
    /// delta 내 경로의 부모 디렉토리를 ChildState::NotLoaded 로 초기화하여
    /// 다음 expand 시 fresh read_dir 가 수행되도록 한다.
    /// 변경이 있으면 cx.notify() 를 호출한다.
    pub fn apply_delta(&mut self, delta: FsDelta, cx: &mut Context<Self>) {
        if delta.is_empty() {
            return;
        }

        // delta 내 모든 경로의 부모 집합을 수집
        let affected_parents: std::collections::HashSet<std::path::PathBuf> = delta
            .created
            .iter()
            .chain(delta.removed.iter())
            .chain(delta.modified.iter())
            .chain(delta.renamed.iter().flat_map(|(a, b)| [a, b]))
            .filter_map(|p| p.parent().map(|par| par.to_path_buf()))
            .collect();

        // 트리를 순회하며 영향받은 디렉토리를 NotLoaded 로 초기화
        invalidate_dirs(&mut self.tree, &affected_parents);

        cx.notify();
    }

    /// 파일 노드를 클릭했을 때 on_file_open 콜백을 호출한다.
    pub fn open_file(&self, rel_path: &PathBuf) {
        if let Some(cb) = &self.on_file_open {
            let abs_path = self.workspace_root.join(rel_path);
            cb(rel_path.clone(), abs_path);
        }
    }

    /// 파일 열기 이벤트를 GPUI EventEmitter 로 emit 한다 (AC-WIRE-1).
    ///
    /// `wire_file_explorer_callback` 으로 RootView 가 subscribe 하며,
    /// 기존 `on_file_open` 콜백과 독립적으로 동작한다.
    pub fn emit_open_file(&mut self, rel_path: &PathBuf, cx: &mut Context<Self>) {
        let abs_path = self.workspace_root.join(rel_path);
        cx.emit(FileOpenEvent { abs_path });
    }
}

// ============================================================
// invalidate_dirs — delta 적용 헬퍼
// ============================================================

/// `node` 와 그 자손 중 `affected` 에 포함된 rel_path 를 가진 Dir 노드를
/// ChildState::NotLoaded 로 초기화한다.
fn invalidate_dirs(node: &mut FsNode, affected: &std::collections::HashSet<std::path::PathBuf>) {
    if let FsNode::Dir {
        rel_path, children, ..
    } = node
    {
        if affected.contains(rel_path.as_path()) {
            *children = super::tree::ChildState::NotLoaded;
            // 자식을 NotLoaded 로 설정했으므로 더 이상 순회 불필요
            return;
        }

        if let super::tree::ChildState::Loaded(kids) = children {
            for kid in kids.iter_mut() {
                invalidate_dirs(kid, affected);
            }
        }
    }
}

// ============================================================
// impl Render — MS-1 placeholder
// ============================================================

impl Render for FileExplorer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // MS-1: 최소 placeholder — 루트 경로 표시
        // MS-2 에서 실제 트리 렌더 + watch 배선 추가 예정
        let root_label = format!(
            "FileExplorer MS-1 placeholder — root: {}",
            self.workspace_root.display()
        );

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .px(px(8.))
            .py(px(8.))
            .bg(rgb(crate::tokens::BG_SURFACE))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(crate::tokens::FG_MUTED))
                    .child(root_label),
            )
    }
}

// ============================================================
// 단위 테스트 — AC-FE-4
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn file_explorer_new_default_state() {
        // AC-FE-4: 생성 직후 기본 상태 검증
        let root = PathBuf::from("/tmp/test-ws");
        let explorer = FileExplorer::new(root.clone());

        assert_eq!(explorer.workspace_root, root);
        assert!(explorer.on_file_open.is_none());
        assert_eq!(explorer.search_query, "");

        // 트리 루트는 Dir::NotLoaded 여야 한다
        assert!(explorer.tree.is_dir());
        if let FsNode::Dir {
            children,
            is_expanded,
            ..
        } = &explorer.tree
        {
            assert_eq!(*children, ChildState::NotLoaded);
            assert!(!is_expanded);
        } else {
            panic!("루트 트리는 Dir 여야 한다");
        }
    }

    #[test]
    fn set_on_file_open_callback_invoked() {
        // AC-FE-4: on_file_open 콜백 등록 후 open_file 호출 시 1 회 invocation 검증
        let root = PathBuf::from("/tmp/test-ws");
        let mut explorer = FileExplorer::new(root.clone());

        let call_count = Arc::new(Mutex::new(0u32));
        let call_count_clone = call_count.clone();

        explorer.set_on_file_open(move |_rel, _abs| {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
        });

        let rel = PathBuf::from("src/main.rs");
        explorer.open_file(&rel);

        assert_eq!(
            *call_count.lock().unwrap(),
            1,
            "콜백이 정확히 1 회 호출되어야 한다"
        );
    }

    #[test]
    fn open_file_without_callback_does_not_panic() {
        // on_file_open 이 None 일 때 panic 없이 무시해야 한다
        let root = PathBuf::from("/tmp/test-ws");
        let explorer = FileExplorer::new(root);
        let rel = PathBuf::from("src/main.rs");
        // panic 없이 종료되면 통과
        explorer.open_file(&rel);
    }

    // AC-FE-4 (USER-DECISION-B=(a)): GPUI TestAppContext 로 Entity<FileExplorer> 생성 smoke 테스트
    #[test]
    fn file_explorer_entity_can_be_created_via_gpui_context() {
        use gpui::{AppContext, TestAppContext};
        let mut cx = TestAppContext::single();
        let root = PathBuf::from("/tmp/test-ws");
        let entity = cx.new(|_cx| FileExplorer::new(root.clone()));
        // Entity 상태를 읽을 수 있어야 한다
        let ws_root = cx.read(|app| entity.read(app).workspace_root.clone());
        assert_eq!(ws_root, PathBuf::from("/tmp/test-ws"));
    }
}
