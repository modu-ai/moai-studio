//! moai-store: rusqlite WAL 모드 스토어
//!
//! 훅 이벤트, 비용 추적, 워크스페이스 메타데이터를 위한 SQLite WAL 모드 스토어.

use std::path::Path;
use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

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
}

/// SQLite 연결을 감싸는 스토어
pub struct Store {
    conn: Connection,
}

impl Store {
    /// 파일 경로로 데이터베이스를 열거나 생성한다.
    ///
    /// WAL 모드를 활성화하고 초기 마이그레이션을 실행한다.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        let mut store = Self { conn };
        // WAL 모드 활성화
        store.conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        store.migrate()?;
        Ok(store)
    }

    /// 인메모리 데이터베이스를 생성한다 (테스트용).
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        let mut store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// 초기 테이블 스키마를 생성한다.
    fn migrate(&mut self) -> Result<(), StoreError> {
        self.conn
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
                ",
            )
            .map_err(|e| StoreError::MigrationError(e.to_string()))?;
        Ok(())
    }

    /// 워크스페이스를 삽입하고 생성된 ID를 반환한다.
    pub fn insert_workspace(&self, working_dir: &str) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO workspaces (working_dir) VALUES (?1)",
            params![working_dir],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 훅 이벤트를 삽입하고 생성된 ID를 반환한다.
    pub fn insert_hook_event(
        &self,
        workspace_id: i64,
        event_name: &str,
        payload: &str,
    ) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO hook_events (workspace_id, event_name, payload) VALUES (?1, ?2, ?3)",
            params![workspace_id, event_name, payload],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 주어진 ID의 워크스페이스 상태를 반환한다.
    pub fn get_workspace_state(&self, id: i64) -> Result<String, StoreError> {
        self.conn
            .query_row(
                "SELECT state FROM workspaces WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StoreError::NotFound(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        // 인메모리 스토어가 오류 없이 생성되어야 한다.
        let store = Store::open_in_memory();
        assert!(store.is_ok());
    }

    #[test]
    fn test_migrate_creates_tables() {
        // 마이그레이션 후 workspaces, hook_events 테이블이 존재해야 한다.
        let store = Store::open_in_memory().unwrap();

        // workspaces 테이블 확인
        let ws_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workspaces'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ws_count, 1);

        // hook_events 테이블 확인
        let he_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='hook_events'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(he_count, 1);
    }

    #[test]
    fn test_insert_workspace() {
        // 워크스페이스 삽입 후 유효한 ID를 반환해야 한다.
        let store = Store::open_in_memory().unwrap();

        let id = store.insert_workspace("/home/user/project").unwrap();
        assert!(id > 0);

        // 두 번째 삽입은 다른 ID를 반환해야 한다.
        let id2 = store.insert_workspace("/home/user/another").unwrap();
        assert_ne!(id, id2);
    }

    #[test]
    fn test_insert_hook_event() {
        // 훅 이벤트 삽입 후 유효한 ID를 반환해야 한다.
        let store = Store::open_in_memory().unwrap();
        let ws_id = store.insert_workspace("/workspace").unwrap();

        let event_id = store
            .insert_hook_event(ws_id, "PreToolUse", r#"{"tool":"bash"}"#)
            .unwrap();
        assert!(event_id > 0);
    }

    #[test]
    fn test_get_workspace_state() {
        // 새로 삽입된 워크스페이스의 기본 상태는 'starting'이어야 한다.
        let store = Store::open_in_memory().unwrap();
        let id = store.insert_workspace("/workspace/state-test").unwrap();

        let state = store.get_workspace_state(id).unwrap();
        assert_eq!(state, "starting");
    }

    #[test]
    fn test_get_workspace_state_not_found() {
        // 존재하지 않는 ID 조회 시 NotFound 오류를 반환해야 한다.
        let store = Store::open_in_memory().unwrap();

        let result = store.get_workspace_state(9999);
        assert!(matches!(result, Err(StoreError::NotFound(9999))));
    }

    #[test]
    fn test_insert_hook_event_fk_constraint() {
        // 외래 키 제약 조건: 존재하지 않는 workspace_id로 삽입하면 오류가 발생해야 한다.
        let store = Store::open_in_memory().unwrap();

        // SQLite는 기본적으로 외래 키 강제를 비활성화하므로 명시적으로 활성화한다.
        store
            .conn
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();

        let result = store.insert_hook_event(9999, "SomeEvent", "{}");
        assert!(result.is_err());
    }
}
