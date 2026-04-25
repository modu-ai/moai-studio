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
