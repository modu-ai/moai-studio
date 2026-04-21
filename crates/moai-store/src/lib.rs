//! moai-store: rusqlite WAL 모드 스토어
//!
//! 훅 이벤트, 비용 추적, 워크스페이스 메타데이터(v2 스키마)를 위한 SQLite WAL 스토어.
//!
//! # 동시성
//!
//! `Store` 는 내부적으로 `Arc<Mutex<Connection>>` 를 사용한다. SQLite WAL 은
//! 여러 리더와 단일 라이터를 허용하지만, 본 구현은 모든 접근을 단일 뮤텍스로
//! 직렬화하여 안전성을 우선시한다. `Store::clone_handle()` 로 얻은 핸들은
//! 같은 커넥션을 공유하므로 스레드 간 안전하게 이동할 수 있다.

pub mod pane;
pub mod state;
pub mod surface;
pub mod workspace;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

pub use pane::{NewPane, PaneDao, PaneRow, PaneStoreExt, SplitKind};
pub use state::{InvalidTransition, WorkspaceStatus};
pub use surface::{NewSurface, SurfaceDao, SurfaceKind, SurfaceRow, SurfaceStoreExt};
pub use workspace::{NewWorkspace, WorkspaceDao, WorkspaceRow, WorkspaceStoreExt};

/// 스토어 오류 타입
#[derive(Debug, Error)]
pub enum StoreError {
    /// SQLite 오류
    #[error("SQL 오류: {0}")]
    SqlError(#[from] rusqlite::Error),

    /// 마이그레이션 오류
    #[error("마이그레이션 오류: {0}")]
    MigrationError(String),

    /// 레코드를 찾을 수 없음
    #[error("레코드를 찾을 수 없음: id={0}")]
    NotFound(i64),

    /// 상태 전이 오류
    #[error("상태 전이 오류: {0}")]
    InvalidTransition(String),

    /// Mutex poisoned — 이론상 발생하면 안 되나 방어적으로 처리
    #[error("커넥션 락 오염(poisoned)")]
    PoisonedLock,

    /// DB 데이터 손상 (예: 알 수 없는 status 문자열)
    #[error("데이터 손상: {0}")]
    Corrupt(String),
}

/// 내부적으로 공유되는 커넥션 핸들.
pub(crate) type SharedConn = Arc<Mutex<Connection>>;

/// SQLite 스토어.
pub struct Store {
    conn: SharedConn,
}

impl Store {
    /// 파일 경로로 데이터베이스를 열거나 생성한다.
    ///
    /// WAL 모드를 활성화하고 v1 + v2 마이그레이션을 실행한다.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        // WAL 모드 — journal_mode 는 PRAGMA 결과를 반환하므로 query 로 실행한다.
        let journal: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
        if journal.to_lowercase() != "wal" {
            return Err(StoreError::MigrationError(format!(
                "WAL 모드 활성화 실패: journal_mode={journal}"
            )));
        }
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    /// 인메모리 데이터베이스를 생성한다 (테스트용).
    ///
    /// 주의: 인메모리는 WAL 모드를 지원하지 않는다.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    /// 전체 마이그레이션 (v1 → v2 → v3) 실행.
    fn migrate(&self) -> Result<(), StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        guard
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS workspaces (
                    id INTEGER PRIMARY KEY,
                    working_dir TEXT NOT NULL,
                    state TEXT NOT NULL DEFAULT 'starting',
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE TABLE IF NOT EXISTS hook_events (
                    id INTEGER PRIMARY KEY,
                    workspace_id INTEGER NOT NULL,
                    event_name TEXT NOT NULL,
                    payload TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
                );
                CREATE TABLE IF NOT EXISTS schema_version (
                    version INTEGER PRIMARY KEY
                );
                ",
            )
            .map_err(|e| StoreError::MigrationError(e.to_string()))?;

        // 현재 버전 조회
        let current: i64 = guard
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if current < 2 {
            let v2_sql = include_str!("../migrations/V2__workspaces_expand.sql");
            guard
                .execute_batch(v2_sql)
                .map_err(|e| StoreError::MigrationError(format!("V2: {e}")))?;
            guard
                .execute("INSERT INTO schema_version (version) VALUES (2)", [])
                .map_err(|e| StoreError::MigrationError(e.to_string()))?;
        }

        // @MX:NOTE: [AUTO] V3: panes + surfaces 테이블 추가 (SPEC-M2-001 RG-M2-1, RG-M2-2)
        if current < 3 {
            let v3_sql = include_str!("../migrations/V3__panes_surfaces.sql");
            guard
                .execute_batch(v3_sql)
                .map_err(|e| StoreError::MigrationError(format!("V3: {e}")))?;
            guard
                .execute("INSERT INTO schema_version (version) VALUES (3)", [])
                .map_err(|e| StoreError::MigrationError(e.to_string()))?;
        }
        Ok(())
    }

    /// 테스트 전용: 내부 커넥션 락 가드를 반환한다.
    ///
    /// 통합 테스트에서 PRAGMA 질의 등 직접 DB 접근이 필요할 때만 사용한다.
    pub fn conn_for_test(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.conn.lock().expect("conn_for_test: mutex poisoned")
    }

    /// (deprecated v1 API) 워크스페이스를 삽입하고 생성된 ID를 반환한다.
    ///
    /// M1 부터는 `WorkspaceDao::insert` 를 권장한다. 기존 테스트 호환성을 위해 유지.
    pub fn insert_workspace(&self, working_dir: &str) -> Result<i64, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        guard.execute(
            "INSERT INTO workspaces (working_dir, name, project_path) VALUES (?1, ?1, ?1)",
            params![working_dir],
        )?;
        Ok(guard.last_insert_rowid())
    }

    /// 훅 이벤트를 삽입하고 생성된 ID를 반환한다.
    pub fn insert_hook_event(
        &self,
        workspace_id: i64,
        event_name: &str,
        payload: &str,
    ) -> Result<i64, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        guard.execute(
            "INSERT INTO hook_events (workspace_id, event_name, payload) VALUES (?1, ?2, ?3)",
            params![workspace_id, event_name, payload],
        )?;
        Ok(guard.last_insert_rowid())
    }

    /// 관리자 경로: 상태 머신을 우회하여 status 를 덮어쓴다.
    ///
    /// 앱 재시작 복원 등 특수한 경우에만 사용해야 한다.
    // @MX:WARN: [AUTO] state machine 우회 관리자 API
    // @MX:REASON: [AUTO] 재시작 시 dangling Running/Starting row 를 Paused 로 되돌릴 필요가 있다.
    pub fn set_workspace_status_raw(
        &self,
        id: i64,
        status: WorkspaceStatus,
    ) -> Result<(), StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE workspaces SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        Ok(())
    }

    /// 주어진 ID의 워크스페이스 상태(v1 state 컬럼)를 반환한다.
    pub fn get_workspace_state(&self, id: i64) -> Result<String, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        guard
            .query_row(
                "SELECT state FROM workspaces WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StoreError::NotFound(id))
    }
}

impl Clone for Store {
    /// 같은 커넥션을 공유하는 핸들을 반환한다.
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let store = Store::open_in_memory();
        assert!(store.is_ok());
    }

    #[test]
    fn test_migrate_creates_tables() {
        let store = Store::open_in_memory().unwrap();
        let guard = store.conn.lock().unwrap();
        let ws_count: i64 = guard
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workspaces'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ws_count, 1);
        let he_count: i64 = guard
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='hook_events'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(he_count, 1);
    }

    #[test]
    fn test_v2_columns_exist() {
        let store = Store::open_in_memory().unwrap();
        let guard = store.conn.lock().unwrap();
        // PRAGMA table_info 를 통해 v2 컬럼 존재 확인
        let mut stmt = guard.prepare("PRAGMA table_info(workspaces)").unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        for expected in [
            "name",
            "project_path",
            "worktree_path",
            "status",
            "spec_id",
            "claude_session_id",
            "updated_at",
        ] {
            assert!(
                cols.contains(&expected.to_string()),
                "missing column: {expected}"
            );
        }
    }

    #[test]
    fn test_insert_workspace() {
        let store = Store::open_in_memory().unwrap();
        let id = store.insert_workspace("/home/user/project").unwrap();
        assert!(id > 0);
        let id2 = store.insert_workspace("/home/user/another").unwrap();
        assert_ne!(id, id2);
    }

    #[test]
    fn test_insert_hook_event() {
        let store = Store::open_in_memory().unwrap();
        let ws_id = store.insert_workspace("/workspace").unwrap();
        let event_id = store
            .insert_hook_event(ws_id, "PreToolUse", r#"{"tool":"bash"}"#)
            .unwrap();
        assert!(event_id > 0);
    }

    #[test]
    fn test_get_workspace_state() {
        let store = Store::open_in_memory().unwrap();
        let id = store.insert_workspace("/workspace/state-test").unwrap();
        let state = store.get_workspace_state(id).unwrap();
        assert_eq!(state, "starting");
    }

    #[test]
    fn test_get_workspace_state_not_found() {
        let store = Store::open_in_memory().unwrap();
        let result = store.get_workspace_state(9999);
        assert!(matches!(result, Err(StoreError::NotFound(9999))));
    }

    #[test]
    fn test_insert_hook_event_fk_constraint() {
        let store = Store::open_in_memory().unwrap();
        let result = store.insert_hook_event(9999, "SomeEvent", "{}");
        assert!(result.is_err());
    }
}
