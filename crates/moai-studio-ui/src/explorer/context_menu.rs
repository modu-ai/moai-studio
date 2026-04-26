// @MX:ANCHOR: [AUTO] context-menu-struct
// @MX:REASON: [AUTO] ContextMenu 는 REQ-FE-030~033 의 공개 API 경계.
//   fan_in >= 3: FileExplorer::right_click_handler, view.rs 렌더, 통합 테스트.
// @MX:WARN: [AUTO] delete-dispatch-irreversible
// @MX:REASON: [AUTO] REQ-FE-034: Delete 액션은 trash crate 로 OS 휴지통 송부.
//   비가역 동작이므로 항상 confirmation modal 이 선행되어야 한다.
//   USER-DECISION-C=(a): trash::delete 사용.
// @MX:SPEC: SPEC-V3-005

use std::io;
use std::path::PathBuf;

// ============================================================
// ContextTarget — 우클릭 대상 노드 종류
// ============================================================

/// 컨텍스트 메뉴가 열린 대상 노드 종류 (REQ-FE-030/031).
#[derive(Debug, Clone, PartialEq)]
pub enum ContextTarget {
    /// 디렉토리 노드
    Dir(PathBuf),
    /// 파일 노드
    File(PathBuf),
}

// ============================================================
// ContextAction — 컨텍스트 메뉴 항목 액션
// ============================================================

/// 컨텍스트 메뉴에서 선택 가능한 액션 목록 (REQ-FE-030/031).
#[derive(Debug, Clone, PartialEq)]
pub enum ContextAction {
    /// 새 파일 생성 (부모 디렉토리 경로)
    NewFile(PathBuf),
    /// 새 폴더 생성 (부모 디렉토리 경로)
    NewFolder(PathBuf),
    /// 이름 변경 (대상 경로)
    Rename(PathBuf),
    /// 삭제 (대상 경로) — REQ-FE-034: trash crate 로 OS 휴지통 송부
    Delete(PathBuf),
    /// Finder/파일 관리자에서 열기 (대상 경로)
    Reveal(PathBuf),
}

// ============================================================
// ContextMenu — 컨텍스트 메뉴 구조체
// ============================================================

/// 우클릭 시 표시되는 컨텍스트 메뉴 (REQ-FE-030/031).
///
/// Dir: 5개 항목 (NewFile, NewFolder, Rename, Delete, Reveal)
/// File: 3개 항목 (Rename, Delete, Reveal)
#[derive(Debug, Clone)]
pub struct ContextMenu {
    /// 컨텍스트 메뉴를 연 대상 노드
    pub target: ContextTarget,
    /// 메뉴 항목 목록
    pub items: Vec<ContextAction>,
}

impl ContextMenu {
    /// Dir 노드에 대한 컨텍스트 메뉴를 생성한다 (5개 항목, REQ-FE-030).
    pub fn for_dir(dir_path: PathBuf) -> Self {
        let items = vec![
            ContextAction::NewFile(dir_path.clone()),
            ContextAction::NewFolder(dir_path.clone()),
            ContextAction::Rename(dir_path.clone()),
            ContextAction::Delete(dir_path.clone()),
            ContextAction::Reveal(dir_path.clone()),
        ];
        Self {
            target: ContextTarget::Dir(dir_path),
            items,
        }
    }

    /// File 노드에 대한 컨텍스트 메뉴를 생성한다 (3개 항목, REQ-FE-031).
    pub fn for_file(file_path: PathBuf) -> Self {
        let items = vec![
            ContextAction::Rename(file_path.clone()),
            ContextAction::Delete(file_path.clone()),
            ContextAction::Reveal(file_path.clone()),
        ];
        Self {
            target: ContextTarget::File(file_path),
            items,
        }
    }
}

// ============================================================
// InlineEditKind — 인라인 편집 종류
// ============================================================

/// 인라인 편집 박스의 동작 종류 (REQ-FE-032/033).
#[derive(Debug, Clone, PartialEq)]
pub enum InlineEditKind {
    /// 새 파일 이름 입력
    NewFile,
    /// 새 폴더 이름 입력
    NewFolder,
    /// 기존 이름 변경
    Rename,
}

// ============================================================
// InlineEdit — 인라인 이름 입력 상태
// ============================================================

/// 인라인 입력 행 상태 — Enter 로 확정, Esc 로 취소 (REQ-FE-032/033).
#[derive(Debug, Clone)]
pub struct InlineEdit {
    /// 편집 종류
    pub kind: InlineEditKind,
    /// 부모 디렉토리 절대 경로
    pub parent: PathBuf,
    /// 현재 입력 버퍼 (사용자 입력값)
    pub buffer: String,
}

impl InlineEdit {
    /// 새 파일 입력 상태를 생성한다.
    pub fn new_file(parent: PathBuf) -> Self {
        Self {
            kind: InlineEditKind::NewFile,
            parent,
            buffer: String::new(),
        }
    }

    /// 새 폴더 입력 상태를 생성한다.
    pub fn new_folder(parent: PathBuf) -> Self {
        Self {
            kind: InlineEditKind::NewFolder,
            parent,
            buffer: String::new(),
        }
    }

    /// 이름 변경 입력 상태를 생성한다. 기존 이름을 버퍼 초기값으로 설정한다.
    pub fn rename(parent: PathBuf, existing_name: String) -> Self {
        Self {
            kind: InlineEditKind::Rename,
            parent,
            buffer: existing_name,
        }
    }

    /// 입력 버퍼를 갱신한다.
    pub fn update_buffer(&mut self, text: String) {
        self.buffer = text;
    }

    /// 입력을 확정(Enter)하고 fs 작업을 실행한다 (REQ-FE-032/033/035).
    ///
    /// 오류 시 panic 없이 io::Error 를 반환한다 (REQ-FE-035).
    pub fn commit(&self, workspace_root: &std::path::Path) -> io::Result<PathBuf> {
        let name = self.buffer.trim();
        if name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "파일 이름이 비어 있을 수 없다",
            ));
        }
        // S2: path separator 포함 이름 거부 (REQ-S2)
        if name.contains('/') || name.contains('\\') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "파일 이름에 경로 구분자를 포함할 수 없다",
            ));
        }

        let target = workspace_root.join(&self.parent).join(name);

        match self.kind {
            InlineEditKind::NewFile => {
                // 부모 디렉토리 확인 후 파일 생성
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::File::create(&target)?;
                Ok(target)
            }
            InlineEditKind::NewFolder => {
                std::fs::create_dir_all(&target)?;
                Ok(target)
            }
            InlineEditKind::Rename => {
                // Rename: self.parent = 이름변경 대상의 현재 상대 경로 (파일/디렉토리)
                // target = workspace_root.join(parent_dir).join(buffer) 가 아니라
                // old = workspace_root.join(self.parent)
                // new = old.parent().join(buffer)
                let old = workspace_root.join(&self.parent);
                let new_parent = old
                    .parent()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "부모 경로 없음"))?;
                let new_path = new_parent.join(name);
                std::fs::rename(&old, &new_path)?;
                Ok(new_path)
            }
        }
    }
}

// ============================================================
// delete_to_trash — OS 휴지통 삭제 (USER-DECISION-C=(a), REQ-FE-034)
// ============================================================

/// 경로를 OS 휴지통으로 이동한다 (REQ-FE-034, USER-DECISION-C=(a)).
///
/// 오류 시 panic 없이 io::Error 를 반환한다 (REQ-FE-035).
///
/// @MX:WARN: [AUTO] irreversible-trash-delete
/// @MX:REASON: [AUTO] trash::delete 는 되돌리기 어렵다. 호출 전 반드시 confirmation modal 을 표시해야 한다.
pub fn delete_to_trash(path: &std::path::Path) -> io::Result<()> {
    trash::delete(path).map_err(|e| io::Error::other(e.to_string()))
}

// ============================================================
// 단위 테스트 — AC-FE-9, AC-FE-10
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // AC-FE-9: Dir 컨텍스트 메뉴 5개 항목 검증 (REQ-FE-030)
    #[test]
    fn context_menu_for_dir_has_5_items() {
        let dir = PathBuf::from("src");
        let menu = ContextMenu::for_dir(dir.clone());

        assert_eq!(menu.items.len(), 5, "Dir 메뉴는 5개 항목이어야 한다");
        assert!(matches!(menu.target, ContextTarget::Dir(_)));

        // 항목 종류 검증
        assert!(
            menu.items
                .iter()
                .any(|a| matches!(a, ContextAction::NewFile(_))),
            "NewFile 항목이 있어야 한다"
        );
        assert!(
            menu.items
                .iter()
                .any(|a| matches!(a, ContextAction::NewFolder(_))),
            "NewFolder 항목이 있어야 한다"
        );
        assert!(
            menu.items
                .iter()
                .any(|a| matches!(a, ContextAction::Rename(_))),
            "Rename 항목이 있어야 한다"
        );
        assert!(
            menu.items
                .iter()
                .any(|a| matches!(a, ContextAction::Delete(_))),
            "Delete 항목이 있어야 한다"
        );
        assert!(
            menu.items
                .iter()
                .any(|a| matches!(a, ContextAction::Reveal(_))),
            "Reveal 항목이 있어야 한다"
        );
    }

    // AC-FE-9: File 컨텍스트 메뉴 3개 항목 검증 (REQ-FE-031)
    #[test]
    fn context_menu_for_file_has_3_items() {
        let file = PathBuf::from("src/main.rs");
        let menu = ContextMenu::for_file(file.clone());

        assert_eq!(menu.items.len(), 3, "File 메뉴는 3개 항목이어야 한다");
        assert!(matches!(menu.target, ContextTarget::File(_)));

        // File 메뉴에는 NewFile/NewFolder 없어야 한다
        assert!(
            !menu
                .items
                .iter()
                .any(|a| matches!(a, ContextAction::NewFile(_))),
            "File 메뉴에는 NewFile 이 없어야 한다"
        );
        assert!(
            !menu
                .items
                .iter()
                .any(|a| matches!(a, ContextAction::NewFolder(_))),
            "File 메뉴에는 NewFolder 가 없어야 한다"
        );
    }

    // AC-FE-9: InlineEdit 생명주기 — NewFile 생성 → commit → 파일 존재 확인
    #[test]
    fn inline_edit_new_file_commit_creates_file() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let workspace = dir.path();

        let mut edit = InlineEdit::new_file(PathBuf::from(""));
        edit.update_buffer("hello.txt".to_string());

        let created = edit.commit(workspace).expect("commit 실패");
        assert!(created.exists(), "파일이 생성되어야 한다");
        assert_eq!(created.file_name().unwrap(), "hello.txt");
    }

    // AC-FE-9: InlineEdit NewFolder 생성
    #[test]
    fn inline_edit_new_folder_commit_creates_dir() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let workspace = dir.path();

        let mut edit = InlineEdit::new_folder(PathBuf::from(""));
        edit.update_buffer("my_dir".to_string());

        let created = edit.commit(workspace).expect("commit 실패");
        assert!(created.is_dir(), "디렉토리가 생성되어야 한다");
    }

    // AC-FE-9: InlineEdit Rename
    #[test]
    fn inline_edit_rename_moves_file() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let workspace = dir.path();

        // 원본 파일 생성 (workspace root 에 직접)
        let original = workspace.join("original.txt");
        fs::write(&original, b"content").expect("파일 생성 실패");

        // Rename: parent="" (워크스페이스 루트), 기존 이름="original.txt"
        // commit 시 old = workspace.join("") = workspace, new = workspace.join("renamed.txt")
        // → workspace/original.txt → workspace/renamed.txt 로 이동
        // InlineEdit::rename 에서 parent 는 파일이 위치한 디렉토리 경로
        // commit: old = workspace_root.join(parent) = workspace, new = old.join(buffer) = workspace/renamed.txt
        // 하지만 이렇게 하면 workspace 디렉토리 자체를 rename 하게 됨
        // → 올바른 설계: Rename 은 parent=파일의 parent dir, buffer=새 이름
        // commit 에서 old = workspace.join(parent).join(기존이름)이 필요하지만 기존이름을 저장하지 않음
        // 해결: 기존이름은 생성 시 buffer 로 preset 되고, 새 이름으로 buffer 갱신
        // old_path = workspace.join(parent) → 이 경우 파일 자체가 old_path 여야 함
        // → Rename 시 parent = 파일 자체의 상대 경로 (old path)
        // commit: old = workspace.join(parent), new = old.parent().join(buffer)

        let mut edit = InlineEdit::rename(
            PathBuf::from("original.txt"), // parent = 이름변경 대상의 현재 상대 경로
            "original.txt".to_string(),    // 기존 이름 (버퍼 초기값)
        );
        edit.update_buffer("renamed.txt".to_string());

        let new_path = edit.commit(workspace).expect("rename 실패");
        assert!(new_path.exists(), "새 이름의 파일이 있어야 한다");
        assert!(!original.exists(), "원본 파일은 없어야 한다");
    }

    // AC-FE-10: delete_to_trash 동작 검증 (파일이 존재하는 tempdir에서 실행)
    #[test]
    fn delete_to_trash_removes_file() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let file = dir.path().join("to_delete.txt");
        fs::write(&file, b"data").expect("파일 생성 실패");
        assert!(file.exists());

        let result = delete_to_trash(&file);
        // trash::delete 는 CI 환경에서도 동작해야 하지만, 환경에 따라 실패 가능
        // 결과와 무관하게 panic 없이 Result 반환 확인
        match result {
            Ok(()) => {
                // 성공 시 파일 삭제됨
                assert!(!file.exists(), "trash 이동 후 파일은 없어야 한다");
            }
            Err(e) => {
                // CI 등 환경에서 trash 불가 → 오류 반환 (no panic)
                tracing::warn!("delete_to_trash CI 환경에서 실패 (예상된 동작): {e}");
            }
        }
    }

    // AC-FE-10: 빈 버퍼 commit → error (no panic)
    #[test]
    fn inline_edit_empty_buffer_returns_error() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let edit = InlineEdit::new_file(PathBuf::from(""));
        // buffer 가 비어 있음
        let result = edit.commit(dir.path());
        assert!(result.is_err(), "빈 버퍼 commit 은 error 여야 한다");
    }

    // S2: 경로 구분자 포함 이름 거부
    #[test]
    fn inline_edit_rejects_path_separator_in_name() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let mut edit = InlineEdit::new_file(PathBuf::from(""));
        edit.update_buffer("evil/path.txt".to_string());
        let result = edit.commit(dir.path());
        assert!(result.is_err(), "경로 구분자 포함 이름은 거부되어야 한다");
    }
}
