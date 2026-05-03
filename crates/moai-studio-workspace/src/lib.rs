//! MoAI Studio Multi-Project Workspace.
//!
//! Phase 1.5 (SPEC-V3-001 RG-V3-2): JSON persistence + CRUD.
//!
//! ## 기능
//! - `pick_project_folder()` — 네이티브 파일 picker
//! - `Workspace::from_path()` — 폴더로부터 워크스페이스 객체
//! - `WorkspacesStore::load()` — `~/.moai/studio/workspaces.json` 로드
//! - `WorkspacesStore::save()` — 파일 저장
//! - `WorkspacesStore::add()` — 워크스페이스 추가 + 저장
//! - `WorkspacesStore::remove()` — 삭제
//! - `WorkspacesStore::touch()` — last_active 갱신
//!
//! ## Phase 5 (SPEC-V3-004) 확장 예정
//! - 사이드바 스위처, 최근 사용, 글로벌 검색
//! - 드래그앤드롭 폴더 추가

pub mod panes_convert;
pub mod persistence;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

// ============================================================
// Workspace 메타데이터
// ============================================================

/// 하나의 MoAI Studio 워크스페이스 (= 하나의 프로젝트 폴더).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub project_path: PathBuf,
    pub moai_config: PathBuf,
    pub color: u32,
    pub last_active: u64,
}

impl Workspace {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(WorkspaceError::NotADirectory(path.to_path_buf()));
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        Ok(Self {
            id: generate_id(),
            name,
            project_path: path.to_path_buf(),
            moai_config: PathBuf::from(".moai"),
            color: 0xff6a3d,
            last_active: now_unix_secs(),
        })
    }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ws-{:x}", nanos)
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================
// JSON persistence
// ============================================================

/// `~/.moai/studio/workspaces.json` 파일 스키마.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspacesFile {
    #[serde(rename = "$schema", default = "default_schema")]
    pub schema: String,
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

fn default_schema() -> String {
    "moai-studio/workspace-v1".to_string()
}

impl WorkspacesFile {
    pub fn new() -> Self {
        Self {
            schema: default_schema(),
            workspaces: Vec::new(),
        }
    }
}

/// 워크스페이스 persistence 핸들.
///
/// 파일이 없으면 빈 리스트로 시작. `save()` 호출로 JSON 저장.
pub struct WorkspacesStore {
    path: PathBuf,
    pub file: WorkspacesFile,
}

impl WorkspacesStore {
    /// Create an empty store bound to the given path without reading from disk.
    ///
    /// Useful for constructing a default store when the real path is not yet
    /// known, or for test helpers that inject a pre-populated store afterward.
    pub fn empty(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            file: WorkspacesFile::new(),
        }
    }

    /// 기본 경로 (`~/.moai/studio/workspaces.json`) 로 로드.
    pub fn load_default() -> Result<Self, WorkspaceError> {
        let path = default_storage_path()?;
        Self::load(path)
    }

    /// 지정 경로로 로드. 파일이 없으면 빈 store 반환.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, WorkspaceError> {
        let path = path.into();
        let file = if path.exists() {
            let bytes = std::fs::read(&path)?;
            serde_json::from_slice(&bytes)?
        } else {
            info!(
                "WorkspacesStore::load: {} 없음, 빈 store 초기화",
                path.display()
            );
            WorkspacesFile::new()
        };
        Ok(Self { path, file })
    }

    /// 현재 상태를 파일에 저장. 부모 디렉토리 자동 생성.
    pub fn save(&self) -> Result<(), WorkspaceError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.file)?;
        std::fs::write(&self.path, json)?;
        info!("WorkspacesStore::save: {}", self.path.display());
        Ok(())
    }

    /// 워크스페이스 추가 + 저장.
    pub fn add(&mut self, ws: Workspace) -> Result<(), WorkspaceError> {
        self.file.workspaces.push(ws);
        self.save()
    }

    /// ID 로 워크스페이스 제거 + 저장. 존재하지 않으면 `Ok(false)`.
    pub fn remove(&mut self, id: &str) -> Result<bool, WorkspaceError> {
        let len_before = self.file.workspaces.len();
        self.file.workspaces.retain(|w| w.id != id);
        let removed = self.file.workspaces.len() < len_before;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// ID 로 last_active 갱신 + 저장.
    pub fn touch(&mut self, id: &str) -> Result<(), WorkspaceError> {
        let now = now_unix_secs();
        let mut changed = false;
        for ws in self.file.workspaces.iter_mut() {
            if ws.id == id {
                ws.last_active = now;
                changed = true;
                break;
            }
        }
        if changed {
            self.save()?;
        }
        Ok(())
    }

    // @MX:ANCHOR: [AUTO] workspace-store-rename
    // @MX:REASON: [AUTO] REQ-D2-MS5-1. rename is the primary mutation entry point for
    //   workspace identity changes. fan_in >= 3: dispatch_workspace_menu_action (T6),
    //   RootView::handle_workspace_menu_action (T7), integration tests (T1).
    /// Rename workspace by `id` to `new_name` and persist.
    ///
    /// Returns `Err(EmptyName)` when `new_name` trims to blank.
    /// Returns `Err(NotFound)` when `id` does not exist in the store.
    pub fn rename(&mut self, id: &str, new_name: &str) -> Result<(), WorkspaceError> {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return Err(WorkspaceError::EmptyName);
        }
        let ws = self
            .file
            .workspaces
            .iter_mut()
            .find(|w| w.id == id)
            .ok_or_else(|| WorkspaceError::NotFound(id.to_string()))?;
        ws.name = trimmed.to_string();
        self.save()
    }

    // @MX:ANCHOR: [AUTO] workspace-store-move-up
    // @MX:REASON: [AUTO] REQ-D2-MS5-2. move_up is the reorder entry point for sidebar list.
    //   fan_in >= 3: dispatch_workspace_menu_action (T6), RootView::handle_workspace_menu_action (T7),
    //   integration tests (T2).
    /// Move workspace one position upward in the ordered list and persist.
    ///
    /// No-op when the workspace is already at index 0.
    /// Returns `Err(NotFound)` when `id` does not exist.
    pub fn move_up(&mut self, id: &str) -> Result<(), WorkspaceError> {
        let idx = self
            .file
            .workspaces
            .iter()
            .position(|w| w.id == id)
            .ok_or_else(|| WorkspaceError::NotFound(id.to_string()))?;
        if idx > 0 {
            self.file.workspaces.swap(idx, idx - 1);
            self.save()?;
        }
        Ok(())
    }

    // @MX:ANCHOR: [AUTO] workspace-store-move-down
    // @MX:REASON: [AUTO] REQ-D2-MS5-2. move_down is the reorder entry point for sidebar list.
    //   fan_in >= 3: dispatch_workspace_menu_action (T6), RootView::handle_workspace_menu_action (T7),
    //   integration tests (T3).
    /// Move workspace one position downward in the ordered list and persist.
    ///
    /// No-op when the workspace is already at the last index.
    /// Returns `Err(NotFound)` when `id` does not exist.
    pub fn move_down(&mut self, id: &str) -> Result<(), WorkspaceError> {
        let last = self.file.workspaces.len().saturating_sub(1);
        let idx = self
            .file
            .workspaces
            .iter()
            .position(|w| w.id == id)
            .ok_or_else(|| WorkspaceError::NotFound(id.to_string()))?;
        if idx < last {
            self.file.workspaces.swap(idx, idx + 1);
            self.save()?;
        }
        Ok(())
    }

    pub fn list(&self) -> &[Workspace] {
        &self.file.workspaces
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// OS 별 기본 경로 (`~/.moai/studio/workspaces.json` on macOS/Linux).
///
/// Windows 는 `%APPDATA%\moai\studio\workspaces.json`.
pub fn default_storage_path() -> Result<PathBuf, WorkspaceError> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|p| p.join("moai").join("studio").join("workspaces.json"))
            .ok_or(WorkspaceError::NoHome)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|p| p.join(".moai").join("studio").join("workspaces.json"))
            .ok_or(WorkspaceError::NoHome)
    }
}

// ============================================================
// 네이티브 파일 picker
// ============================================================

pub fn pick_project_folder() -> Option<PathBuf> {
    info!("pick_project_folder: 네이티브 폴더 다이얼로그 호출");
    let path = rfd::FileDialog::new()
        .set_title("MoAI Studio — 프로젝트 폴더 선택")
        .pick_folder();
    match &path {
        Some(p) => info!("pick_project_folder: 선택됨 — {}", p.display()),
        None => warn!("pick_project_folder: 취소됨"),
    }
    path
}

pub fn pick_and_create() -> Result<Option<Workspace>, WorkspaceError> {
    match pick_project_folder() {
        Some(path) => Workspace::from_path(&path).map(Some),
        None => Ok(None),
    }
}

/// 편의: picker + store 추가 + save 를 한 번에.
pub fn pick_and_save(store: &mut WorkspacesStore) -> Result<Option<Workspace>, WorkspaceError> {
    match pick_and_create()? {
        Some(ws) => {
            store.add(ws.clone())?;
            Ok(Some(ws))
        }
        None => Ok(None),
    }
}

// ============================================================
// 에러 타입
// ============================================================

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("경로가 디렉토리가 아닙니다: {0}")]
    NotADirectory(PathBuf),

    #[error("워크스페이스 I/O 실패: {0}")]
    Io(#[from] std::io::Error),

    #[error("워크스페이스 직렬화 실패: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("홈 디렉토리를 찾을 수 없습니다")]
    NoHome,

    // @MX:NOTE: [AUTO] REQ-D2-MS5-1 — EmptyName guards rename against blank workspace names.
    /// Workspace name must be non-empty after trimming whitespace (REQ-D2-MS5-1).
    #[error("워크스페이스 이름이 비어있습니다")]
    EmptyName,

    // @MX:NOTE: [AUTO] REQ-D2-MS5-1/2 — NotFound returned by rename/move_up/move_down
    /// No workspace with the given id was found in the store (REQ-D2-MS5-1, REQ-D2-MS5-2).
    #[error("워크스페이스를 찾을 수 없습니다: {0}")]
    NotFound(String),
}

// ============================================================
// 스캐폴드 hello
// ============================================================

pub fn hello() {
    info!("moai-studio-workspace: Phase 1.5 — JSON persistence + CRUD 활성");
}

// ============================================================
// 유닛 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── T1: WorkspacesStore::rename ──────────────────────────────────────────

    #[test]
    fn test_rename_existing_workspace_updates_name() {
        let tmp_file = std::env::temp_dir().join("moai-ws-rename-ok.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = tmp_dir("project-rename-ok");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws = Workspace::from_path(&project).unwrap();
        let id = ws.id.clone();
        store.add(ws).unwrap();

        store.rename(&id, "NewName").unwrap();

        let reloaded = WorkspacesStore::load(&tmp_file).unwrap();
        assert_eq!(reloaded.list()[0].name, "NewName");

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
    }

    #[test]
    fn test_rename_unknown_workspace_returns_not_found() {
        let tmp_file = std::env::temp_dir().join("moai-ws-rename-unknown.json");
        std::fs::remove_file(&tmp_file).ok();

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let result = store.rename("nonexistent-id", "SomeName");
        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));

        std::fs::remove_file(&tmp_file).ok();
    }

    #[test]
    fn test_rename_empty_name_returns_empty_name_error() {
        let tmp_file = std::env::temp_dir().join("moai-ws-rename-empty.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = tmp_dir("project-rename-empty");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws = Workspace::from_path(&project).unwrap();
        let id = ws.id.clone();
        store.add(ws).unwrap();

        let result = store.rename(&id, "   ");
        assert!(matches!(result, Err(WorkspaceError::EmptyName)));

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
    }

    // ── T2: WorkspacesStore::move_up ────────────────────────────────────────

    #[test]
    fn test_move_up_middle_workspace() {
        let tmp_file = std::env::temp_dir().join("moai-ws-moveup-mid.json");
        std::fs::remove_file(&tmp_file).ok();
        let p1 = tmp_dir("project-moveup-a");
        let p2 = tmp_dir("project-moveup-b");
        let p3 = tmp_dir("project-moveup-c");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws1 = Workspace::from_path(&p1).unwrap();
        let ws2 = Workspace::from_path(&p2).unwrap();
        let ws3 = Workspace::from_path(&p3).unwrap();
        let id2 = ws2.id.clone();
        store.add(ws1).unwrap();
        store.add(ws2).unwrap();
        store.add(ws3).unwrap();

        store.move_up(&id2).unwrap();

        // ws2 should now be at index 0
        assert_eq!(store.list()[0].id, id2);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&p1).ok();
        std::fs::remove_dir_all(&p2).ok();
        std::fs::remove_dir_all(&p3).ok();
    }

    #[test]
    fn test_move_up_first_workspace_no_op() {
        let tmp_file = std::env::temp_dir().join("moai-ws-moveup-first.json");
        std::fs::remove_file(&tmp_file).ok();
        let p1 = tmp_dir("project-moveup-first-a");
        let p2 = tmp_dir("project-moveup-first-b");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws1 = Workspace::from_path(&p1).unwrap();
        let ws2 = Workspace::from_path(&p2).unwrap();
        let id1 = ws1.id.clone();
        store.add(ws1).unwrap();
        store.add(ws2).unwrap();

        store.move_up(&id1).unwrap();

        // id1 should remain at index 0
        assert_eq!(store.list()[0].id, id1);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&p1).ok();
        std::fs::remove_dir_all(&p2).ok();
    }

    #[test]
    fn test_move_up_unknown_returns_not_found() {
        let tmp_file = std::env::temp_dir().join("moai-ws-moveup-unknown.json");
        std::fs::remove_file(&tmp_file).ok();

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let result = store.move_up("no-such-id");
        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));

        std::fs::remove_file(&tmp_file).ok();
    }

    // ── T3: WorkspacesStore::move_down ──────────────────────────────────────

    #[test]
    fn test_move_down_middle_workspace() {
        let tmp_file = std::env::temp_dir().join("moai-ws-movedown-mid.json");
        std::fs::remove_file(&tmp_file).ok();
        let p1 = tmp_dir("project-movedown-a");
        let p2 = tmp_dir("project-movedown-b");
        let p3 = tmp_dir("project-movedown-c");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws1 = Workspace::from_path(&p1).unwrap();
        let ws2 = Workspace::from_path(&p2).unwrap();
        let ws3 = Workspace::from_path(&p3).unwrap();
        let id2 = ws2.id.clone();
        let id3 = ws3.id.clone();
        store.add(ws1).unwrap();
        store.add(ws2).unwrap();
        store.add(ws3).unwrap();

        store.move_down(&id2).unwrap();

        // ws2 should now be at index 2, ws3 at index 1
        assert_eq!(store.list()[2].id, id2);
        assert_eq!(store.list()[1].id, id3);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&p1).ok();
        std::fs::remove_dir_all(&p2).ok();
        std::fs::remove_dir_all(&p3).ok();
    }

    #[test]
    fn test_move_down_last_workspace_no_op() {
        let tmp_file = std::env::temp_dir().join("moai-ws-movedown-last.json");
        std::fs::remove_file(&tmp_file).ok();
        let p1 = tmp_dir("project-movedown-last-a");
        let p2 = tmp_dir("project-movedown-last-b");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws1 = Workspace::from_path(&p1).unwrap();
        let ws2 = Workspace::from_path(&p2).unwrap();
        let id2 = ws2.id.clone();
        store.add(ws1).unwrap();
        store.add(ws2).unwrap();

        store.move_down(&id2).unwrap();

        // id2 should remain at index 1 (last)
        assert_eq!(store.list()[1].id, id2);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&p1).ok();
        std::fs::remove_dir_all(&p2).ok();
    }

    #[test]
    fn test_move_down_unknown_returns_not_found() {
        let tmp_file = std::env::temp_dir().join("moai-ws-movedown-unknown.json");
        std::fs::remove_file(&tmp_file).ok();

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let result = store.move_down("no-such-id");
        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));

        std::fs::remove_file(&tmp_file).ok();
    }

    fn tmp_dir(suffix: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("moai-ws-test-{}", suffix));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn workspace_from_path_extracts_folder_name() {
        let tmp = tmp_dir("folder-name");
        let ws = Workspace::from_path(&tmp).expect("should create workspace");
        assert_eq!(ws.name, tmp.file_name().unwrap().to_str().unwrap());
        assert!(!ws.id.is_empty());
        assert_eq!(ws.color, 0xff6a3d);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn workspace_from_path_rejects_non_directory() {
        let tmp = std::env::temp_dir().join("moai-ws-test-not-a-dir.txt");
        std::fs::write(&tmp, b"not a dir").unwrap();
        let result = Workspace::from_path(&tmp);
        assert!(matches!(result, Err(WorkspaceError::NotADirectory(_))));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn workspace_serializes_to_json() {
        let tmp = tmp_dir("serialize");
        let ws = Workspace::from_path(&tmp).unwrap();
        let json = serde_json::to_string(&ws).unwrap();
        let expected_color = 0xff6a3du32;
        assert_eq!(expected_color, 16_738_877);
        assert!(json.contains(&format!("\"color\":{}", expected_color)));
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn store_load_creates_empty_when_missing() {
        let tmp = std::env::temp_dir().join("moai-ws-missing.json");
        std::fs::remove_file(&tmp).ok();
        let store = WorkspacesStore::load(&tmp).expect("empty store on missing file");
        assert!(store.list().is_empty());
        assert_eq!(store.file.schema, "moai-studio/workspace-v1");
    }

    #[test]
    fn store_save_and_reload_roundtrip() {
        let tmp_file = std::env::temp_dir().join("moai-ws-roundtrip.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = tmp_dir("project-roundtrip");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws = Workspace::from_path(&project).unwrap();
        store.add(ws.clone()).expect("add + save");

        // 재로드
        let reloaded = WorkspacesStore::load(&tmp_file).expect("reload");
        assert_eq!(reloaded.list().len(), 1);
        assert_eq!(reloaded.list()[0], ws);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
    }

    #[test]
    fn store_remove_deletes_by_id() {
        let tmp_file = std::env::temp_dir().join("moai-ws-remove.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = tmp_dir("project-remove");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let ws = Workspace::from_path(&project).unwrap();
        let id = ws.id.clone();
        store.add(ws).unwrap();
        assert_eq!(store.list().len(), 1);

        let removed = store.remove(&id).unwrap();
        assert!(removed);
        assert!(store.list().is_empty());

        // 두 번째 호출은 false
        let again = store.remove(&id).unwrap();
        assert!(!again);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
    }

    #[test]
    fn store_touch_updates_last_active() {
        let tmp_file = std::env::temp_dir().join("moai-ws-touch.json");
        std::fs::remove_file(&tmp_file).ok();
        let project = tmp_dir("project-touch");

        let mut store = WorkspacesStore::load(&tmp_file).unwrap();
        let mut ws = Workspace::from_path(&project).unwrap();
        ws.last_active = 1000; // 과거 시각
        let id = ws.id.clone();
        store.add(ws).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        store.touch(&id).unwrap();

        let reloaded = WorkspacesStore::load(&tmp_file).unwrap();
        assert!(reloaded.list()[0].last_active > 1000);

        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_dir_all(&project).ok();
    }

    #[test]
    fn default_storage_path_resolves() {
        let path = default_storage_path();
        assert!(path.is_ok());
        let p = path.unwrap();
        assert!(p.ends_with("workspaces.json"));
    }
}
