// @MX:ANCHOR: [AUTO] fs-node-tree
// @MX:REASON: [AUTO] FsNode 는 File Explorer 전체의 핵심 도메인 모델. fan_in >= 3:
//   walk_loaded, watcher delta apply (MS-2), RG-FE-2/3/4 소비자들(view/git_status/search).
//   FsNode shape 는 explorer 모듈의 계약이다.
// @MX:SPEC: SPEC-V3-005

use std::path::PathBuf;

// ============================================================
// FsError — ChildState::Failed 페이로드
// ============================================================

/// 파일 시스템 오류 — I/O 또는 권한 거부.
/// REQ-FE-006: read_dir 실패를 panic 없이 흡수하기 위한 타입.
#[derive(Debug, Clone, PartialEq)]
pub enum FsError {
    /// 권한 거부 에러
    PermissionDenied(PathBuf),
    /// 기타 I/O 에러
    Io(String),
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::PermissionDenied(p) => write!(f, "권한 거부: {}", p.display()),
            FsError::Io(s) => write!(f, "I/O 에러: {s}"),
        }
    }
}

// ============================================================
// ChildState — lazy load 생명주기
// ============================================================

// @MX:NOTE: [AUTO] child-state-lazy-load
// @MX:SPEC: SPEC-V3-005 RG-FE-1 REQ-FE-002
// ChildState 는 Dir 노드의 자식 로딩 상태를 나타낸다.
// NotLoaded → Loading → Loaded(children) or Failed(err) 전이가 lazy load 의 계약이다.

/// 디렉토리 자식 로딩 상태 — REQ-FE-002 4변형.
#[derive(Debug, Clone, PartialEq)]
pub enum ChildState {
    /// 아직 로드하지 않은 상태 (초기값)
    NotLoaded,
    /// 비동기 read_dir 진행 중
    Loading,
    /// 로드 완료 — 자식 목록 보유
    Loaded(Vec<FsNode>),
    /// 로드 실패 — 오류 정보 보유 (REQ-FE-006)
    Failed(FsError),
}

// ============================================================
// FsNode — 파일 시스템 트리 노드
// ============================================================

/// 파일 시스템 트리 노드 — GPUI 의존 없는 logic-only 도메인 모델 (REQ-FE-001).
/// File Explorer 의 모든 동작(watch delta apply, git status, search filter, render)이
/// 이 enum 을 통해 트리를 조작한다.
#[derive(Debug, Clone, PartialEq)]
pub enum FsNode {
    /// 일반 파일 노드
    File {
        /// 워크스페이스 루트 기준 상대 경로
        rel_path: PathBuf,
        /// 표시 이름
        name: String,
        /// fuzzy filter 가시성 플래그 (쿼리 빈 상태에서 true)
        is_visible_under_filter: bool,
    },
    /// 디렉토리 노드
    Dir {
        /// 워크스페이스 루트 기준 상대 경로
        rel_path: PathBuf,
        /// 표시 이름
        name: String,
        /// 자식 로딩 상태
        children: ChildState,
        /// 트리에서 펼쳐진 상태 여부
        is_expanded: bool,
        /// fuzzy filter 가시성 플래그
        is_visible_under_filter: bool,
    },
}

impl FsNode {
    /// 기본값으로 File 노드를 생성한다.
    pub fn file(rel_path: PathBuf, name: String) -> Self {
        FsNode::File {
            rel_path,
            name,
            is_visible_under_filter: true,
        }
    }

    /// 기본값으로 Dir 노드를 생성한다 (ChildState::NotLoaded, is_expanded=false).
    pub fn dir_unloaded(rel_path: PathBuf, name: String) -> Self {
        FsNode::Dir {
            rel_path,
            name,
            children: ChildState::NotLoaded,
            is_expanded: false,
            is_visible_under_filter: true,
        }
    }

    /// 이 노드가 Dir 인지 반환한다.
    pub fn is_dir(&self) -> bool {
        matches!(self, FsNode::Dir { .. })
    }

    /// 표시 이름을 반환한다.
    pub fn name(&self) -> &str {
        match self {
            FsNode::File { name, .. } => name,
            FsNode::Dir { name, .. } => name,
        }
    }

    /// 워크스페이스 루트 기준 상대 경로를 반환한다.
    pub fn path(&self) -> &PathBuf {
        match self {
            FsNode::File { rel_path, .. } => rel_path,
            FsNode::Dir { rel_path, .. } => rel_path,
        }
    }

    /// Loaded 상태의 자식들을 깊이 우선 순회한다.
    /// NotLoaded / Loading / Failed 자식은 방문하지 않는다 (REQ-FE-002).
    pub fn walk_loaded(&self, visitor: &mut impl FnMut(&FsNode)) {
        visitor(self);
        if let FsNode::Dir {
            children: ChildState::Loaded(kids),
            ..
        } = self
        {
            for kid in kids {
                kid.walk_loaded(visitor);
            }
        }
    }
}

// ============================================================
// 단위 테스트 — AC-FE-1, AC-FE-2
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_node_file_basic_fields() {
        // AC-FE-1: File 노드의 기본 필드 검증
        let path = PathBuf::from("src/main.rs");
        let node = FsNode::file(path.clone(), "main.rs".to_string());

        assert!(!node.is_dir());
        assert_eq!(node.name(), "main.rs");
        assert_eq!(node.path(), &path);

        if let FsNode::File {
            is_visible_under_filter,
            ..
        } = &node
        {
            assert!(*is_visible_under_filter, "초기 가시성은 true 여야 한다");
        } else {
            panic!("File 노드여야 한다");
        }
    }

    #[test]
    fn fs_node_dir_starts_not_loaded() {
        // AC-FE-1: Dir 노드의 초기 상태 검증
        let path = PathBuf::from("src");
        let node = FsNode::dir_unloaded(path.clone(), "src".to_string());

        assert!(node.is_dir());
        assert_eq!(node.name(), "src");
        assert_eq!(node.path(), &path);

        if let FsNode::Dir {
            children,
            is_expanded,
            ..
        } = &node
        {
            assert_eq!(
                *children,
                ChildState::NotLoaded,
                "초기 상태는 NotLoaded 여야 한다"
            );
            assert!(!is_expanded, "초기에 펼쳐진 상태가 아니어야 한다");
        } else {
            panic!("Dir 노드여야 한다");
        }
    }

    #[test]
    fn child_state_transitions_not_loaded_to_loading_to_loaded() {
        // AC-FE-2: ChildState 전이 검증
        let mut state = ChildState::NotLoaded;
        assert_eq!(state, ChildState::NotLoaded);

        // NotLoaded → Loading
        state = ChildState::Loading;
        assert_eq!(state, ChildState::Loading);

        // Loading → Loaded
        let child = FsNode::file(PathBuf::from("src/lib.rs"), "lib.rs".to_string());
        state = ChildState::Loaded(vec![child.clone()]);
        if let ChildState::Loaded(kids) = &state {
            assert_eq!(kids.len(), 1);
            assert_eq!(kids[0].name(), "lib.rs");
        } else {
            panic!("Loaded 상태여야 한다");
        }

        // Failed 전이도 검증
        let err_state = ChildState::Failed(FsError::PermissionDenied(PathBuf::from("/root")));
        if let ChildState::Failed(FsError::PermissionDenied(p)) = &err_state {
            assert_eq!(*p, PathBuf::from("/root"));
        } else {
            panic!("Failed(PermissionDenied) 상태여야 한다");
        }
    }

    #[test]
    fn walk_loaded_visits_only_loaded_children() {
        // AC-FE-1: walk_loaded 는 Loaded 자식만 방문한다
        let child_file = FsNode::file(PathBuf::from("src/main.rs"), "main.rs".to_string());
        let child_dir_unloaded = FsNode::dir_unloaded(PathBuf::from("src/sub"), "sub".to_string());

        let root = FsNode::Dir {
            rel_path: PathBuf::from("src"),
            name: "src".to_string(),
            children: ChildState::Loaded(vec![child_file, child_dir_unloaded]),
            is_expanded: true,
            is_visible_under_filter: true,
        };

        let mut visited_names = Vec::new();
        root.walk_loaded(&mut |node| {
            visited_names.push(node.name().to_string());
        });

        // root(src), child_file(main.rs), child_dir_unloaded(sub) 방문
        // child_dir_unloaded 의 자식은 NotLoaded 이므로 더 이상 내려가지 않는다
        assert_eq!(visited_names, vec!["src", "main.rs", "sub"]);
    }
}
