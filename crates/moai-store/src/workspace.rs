//! 워크스페이스 CRUD (SPEC-M1-001 RG-M1-4 산출물).
//!
//! v2 스키마 컬럼 전체를 다루며, 상태 컬럼은 `WorkspaceStatus` enum 과 동기화된다.

use std::str::FromStr;
use std::sync::Arc;

use rusqlite::{OptionalExtension, params};

use crate::state::WorkspaceStatus;
use crate::{SharedConn, StoreError};

/// 워크스페이스 한 행의 전체 상태.
// @MX:ANCHOR: [AUTO] Rust core 에서 Swift UI 까지 공유되는 워크스페이스 도메인 객체 (fan_in>=3)
// @MX:REASON: [AUTO] supervisor/ffi/restore 세 경로에서 동일 구조체를 주고받는다.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceRow {
    /// SQLite rowid (AUTOINCREMENT)
    pub id: i64,
    /// 사용자 노출 이름
    pub name: String,
    /// 프로젝트 루트 절대 경로
    pub project_path: String,
    /// worktree 절대 경로 (없으면 None — 아직 Starting 이전)
    pub worktree_path: Option<String>,
    /// 6-state 상태
    pub status: WorkspaceStatus,
    /// 연결된 SPEC id (없을 수 있음)
    pub spec_id: Option<String>,
    /// 현재 Claude 세션 id (subprocess 가 spawn 되면 세팅)
    pub claude_session_id: Option<String>,
    /// 생성 시각 (ISO8601)
    pub created_at: String,
    /// 마지막 수정 시각 (ISO8601)
    pub updated_at: String,
}

/// 신규 삽입용 파라미터.
#[derive(Debug, Clone)]
pub struct NewWorkspace {
    /// 사용자 노출 이름
    pub name: String,
    /// 프로젝트 루트 절대 경로
    pub project_path: String,
    /// 연결된 SPEC id (없을 수 있음)
    pub spec_id: Option<String>,
}

/// 워크스페이스 CRUD 파사드.
pub struct WorkspaceDao {
    conn: SharedConn,
}

impl WorkspaceDao {
    pub(crate) fn new(conn: SharedConn) -> Self {
        Self { conn }
    }

    /// 새 워크스페이스를 삽입하고 생성된 행을 반환한다.
    pub fn insert(&self, new: &NewWorkspace) -> Result<WorkspaceRow, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        // v1 호환을 위해 working_dir 에도 project_path 를 채운다.
        guard.execute(
            "INSERT INTO workspaces (working_dir, name, project_path, status, spec_id) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                new.project_path,
                new.name,
                new.project_path,
                WorkspaceStatus::Created.as_str(),
                new.spec_id,
            ],
        )?;
        let id = guard.last_insert_rowid();
        drop(guard);
        self.get(id)?.ok_or(StoreError::NotFound(id))
    }

    /// id 로 조회. 없으면 `Ok(None)`.
    pub fn get(&self, id: i64) -> Result<Option<WorkspaceRow>, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let row = guard
            .query_row(
                "SELECT id, name, project_path, worktree_path, status, spec_id, \
                 claude_session_id, created_at, updated_at FROM workspaces WHERE id = ?1",
                params![id],
                map_row,
            )
            .optional()?;
        row.transpose()
    }

    /// 전체 목록 (Deleted 는 제외).
    pub fn list(&self) -> Result<Vec<WorkspaceRow>, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let mut stmt = guard.prepare(
            "SELECT id, name, project_path, worktree_path, status, spec_id, \
             claude_session_id, created_at, updated_at \
             FROM workspaces WHERE status != 'Deleted' ORDER BY id",
        )?;
        let rows = stmt
            .query_map([], map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().collect::<Result<Vec<_>, _>>()
    }

    /// 상태 전이를 시도한다. 허용되지 않으면 오류.
    pub fn update_status(
        &self,
        id: i64,
        new_status: WorkspaceStatus,
    ) -> Result<WorkspaceRow, StoreError> {
        let current = self.get(id)?.ok_or(StoreError::NotFound(id))?;
        let next = current
            .status
            .transition(new_status)
            .map_err(|e| StoreError::InvalidTransition(format!("{e}")))?;
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        guard.execute(
            "UPDATE workspaces SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![next.as_str(), id],
        )?;
        drop(guard);
        self.get(id)?.ok_or(StoreError::NotFound(id))
    }

    /// worktree 경로 업데이트.
    pub fn set_worktree_path(&self, id: i64, path: &str) -> Result<(), StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE workspaces SET worktree_path = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![path, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        Ok(())
    }

    /// Claude 세션 id 업데이트.
    pub fn set_claude_session_id(
        &self,
        id: i64,
        session_id: Option<&str>,
    ) -> Result<(), StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE workspaces SET claude_session_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![session_id, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        Ok(())
    }

    /// 관리자 API: 정상 전이 규칙을 우회하여 강제로 Paused 상태로 설정.
    ///
    /// 앱 재시작 복원 시 dangling Running 상태를 처리하거나, UI 관리자 메뉴에서 긴급 정지할 때 사용.
    // @MX:ANCHOR: [AUTO] 관리자용 강제 일시정지 API (fan_in>=3 예상)
    // @MX:REASON: [AUTO] 재시작 복원(supervisor), UI 관리자 메뉴, 비상 정지 3곳에서 호출. 전이 규칙 우회이므로 신중히 사용.
    pub fn force_pause(&self, id: i64) -> Result<WorkspaceRow, StoreError> {
        // 존재 여부 확인
        self.get(id)?.ok_or(StoreError::NotFound(id))?;
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE workspaces SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![WorkspaceStatus::Paused.as_str(), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        drop(guard);
        self.get(id)?.ok_or(StoreError::NotFound(id))
    }

    /// Soft delete — Deleted 상태로 전이.
    pub fn soft_delete(&self, id: i64) -> Result<(), StoreError> {
        self.update_status(id, WorkspaceStatus::Deleted)?;
        Ok(())
    }

    /// Hard delete — row 물리 삭제.
    pub fn hard_delete(&self, id: i64) -> Result<bool, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<WorkspaceRow, StoreError>> {
    let status_str: String = row.get("status")?;
    let status = WorkspaceStatus::from_str(&status_str)
        .map_err(|e| StoreError::Corrupt(format!("invalid status: {e}")));
    let id: i64 = row.get("id")?;
    let name: String = row.get("name")?;
    let project_path: String = row.get("project_path")?;
    let worktree_path: Option<String> = row.get("worktree_path")?;
    let spec_id: Option<String> = row.get("spec_id")?;
    let claude_session_id: Option<String> = row.get("claude_session_id")?;
    let created_at: String = row.get("created_at")?;
    let updated_at: String = row.get("updated_at")?;
    Ok(status.map(|s| WorkspaceRow {
        id,
        name,
        project_path,
        worktree_path,
        status: s,
        spec_id,
        claude_session_id,
        created_at,
        updated_at,
    }))
}

/// 기존 Store 에 `workspaces()` DAO 접근 포인트를 붙이는 헬퍼 trait.
pub trait WorkspaceStoreExt {
    /// 새 DAO 인스턴스를 반환한다 (내부 Arc 클론만 수행).
    fn workspaces(&self) -> WorkspaceDao;
}

impl WorkspaceStoreExt for crate::Store {
    fn workspaces(&self) -> WorkspaceDao {
        WorkspaceDao::new(Arc::clone(&self.conn))
    }
}
