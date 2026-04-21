//! Surface CRUD (SPEC-M2-001 RG-M2-2 산출물).
//!
//! pane 의 탭 하나에 대응하는 surface 레코드를 DB 에 영속한다.
//! kind 필드는 `SurfaceKind` enum 으로 관리된다.

use std::str::FromStr;
use std::sync::Arc;

use rusqlite::{OptionalExtension, params};

use crate::{SharedConn, StoreError};

/// surface 의 종류 (10종).
// @MX:NOTE: [AUTO] 10종 surface 중 M2 에서 구현되는 것은 terminal/filetree/markdown/image/browser 5종
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SurfaceKind {
    Terminal,
    Code,
    Markdown,
    Image,
    Browser,
    FileTree,
    AgentRun,
    Kanban,
    Memory,
    InstructionsGraph,
}

impl SurfaceKind {
    /// DB 저장용 문자열로 변환.
    pub fn as_str(&self) -> &'static str {
        match self {
            SurfaceKind::Terminal => "terminal",
            SurfaceKind::Code => "code",
            SurfaceKind::Markdown => "markdown",
            SurfaceKind::Image => "image",
            SurfaceKind::Browser => "browser",
            SurfaceKind::FileTree => "filetree",
            SurfaceKind::AgentRun => "agent_run",
            SurfaceKind::Kanban => "kanban",
            SurfaceKind::Memory => "memory",
            SurfaceKind::InstructionsGraph => "instructions_graph",
        }
    }
}

impl FromStr for SurfaceKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "terminal" => Ok(SurfaceKind::Terminal),
            "code" => Ok(SurfaceKind::Code),
            "markdown" => Ok(SurfaceKind::Markdown),
            "image" => Ok(SurfaceKind::Image),
            "browser" => Ok(SurfaceKind::Browser),
            "filetree" => Ok(SurfaceKind::FileTree),
            "agent_run" => Ok(SurfaceKind::AgentRun),
            "kanban" => Ok(SurfaceKind::Kanban),
            "memory" => Ok(SurfaceKind::Memory),
            "instructions_graph" => Ok(SurfaceKind::InstructionsGraph),
            other => Err(format!("알 수 없는 SurfaceKind: {other}")),
        }
    }
}

/// surface 한 행의 전체 상태.
// @MX:ANCHOR: [AUTO] Rust core 와 Swift FFI 사이의 surface 도메인 객체 (fan_in>=3)
// @MX:REASON: [AUTO] SurfaceDao::insert, list_by_pane, FFI surface.rs 세 경로에서 사용
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceRow {
    /// SQLite rowid (AUTOINCREMENT)
    pub id: i64,
    /// 소속 pane id
    pub pane_id: i64,
    /// surface 종류
    pub kind: SurfaceKind,
    /// surface 상태 JSON (없으면 None)
    pub state_json: Option<String>,
    /// 탭 순서 (0부터 시작)
    pub tab_order: i64,
    /// 생성 시각 (ISO8601)
    pub created_at: String,
    /// 마지막 수정 시각 (ISO8601)
    pub updated_at: String,
}

/// 신규 삽입용 파라미터.
#[derive(Debug, Clone)]
pub struct NewSurface {
    /// 소속 pane id
    pub pane_id: i64,
    /// surface 종류
    pub kind: SurfaceKind,
    /// surface 초기 상태 JSON
    pub state_json: Option<String>,
    /// 탭 순서
    pub tab_order: i64,
}

/// Surface CRUD 파사드.
pub struct SurfaceDao {
    conn: SharedConn,
}

impl SurfaceDao {
    pub(crate) fn new(conn: SharedConn) -> Self {
        Self { conn }
    }

    /// 새 surface 를 삽입하고 생성된 행을 반환한다.
    // @MX:ANCHOR: [AUTO] surface 생성 진입점 (fan_in>=3: T-034 DAO, T-036 FFI, T-037 통합테스트)
    // @MX:REASON: [AUTO] pane 내 탭 생성의 최초 진입점
    pub fn insert(&self, new: &NewSurface) -> Result<SurfaceRow, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        guard.execute(
            "INSERT INTO surfaces (pane_id, kind, state_json, tab_order) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                new.pane_id,
                new.kind.as_str(),
                new.state_json,
                new.tab_order
            ],
        )?;
        let id = guard.last_insert_rowid();
        drop(guard);
        self.get(id)?.ok_or(StoreError::NotFound(id))
    }

    /// id 로 조회. 없으면 `Ok(None)`.
    pub fn get(&self, id: i64) -> Result<Option<SurfaceRow>, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let row = guard
            .query_row(
                "SELECT id, pane_id, kind, state_json, tab_order, created_at, updated_at \
                 FROM surfaces WHERE id = ?1",
                params![id],
                map_surface_row,
            )
            .optional()?;
        row.transpose()
    }

    /// pane ID 기준 surface 목록 (tab_order 오름차순).
    pub fn list_by_pane(&self, pane_id: i64) -> Result<Vec<SurfaceRow>, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let mut stmt = guard.prepare(
            "SELECT id, pane_id, kind, state_json, tab_order, created_at, updated_at \
             FROM surfaces WHERE pane_id = ?1 ORDER BY tab_order",
        )?;
        let rows = stmt
            .query_map(params![pane_id], map_surface_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().collect::<Result<Vec<_>, _>>()
    }

    /// kind 를 업데이트하고 갱신된 행을 반환한다.
    pub fn update_kind(&self, id: i64, kind: SurfaceKind) -> Result<SurfaceRow, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE surfaces SET kind = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![kind.as_str(), id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        drop(guard);
        self.get(id)?.ok_or(StoreError::NotFound(id))
    }

    /// tab_order 를 업데이트한다.
    pub fn update_tab_order(&self, id: i64, tab_order: i64) -> Result<(), StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE surfaces SET tab_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![tab_order, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        Ok(())
    }

    /// state_json 을 업데이트한다.
    pub fn update_state_json(&self, id: i64, state_json: Option<&str>) -> Result<(), StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE surfaces SET state_json = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![state_json, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        Ok(())
    }

    /// surface 를 삭제한다. 삭제된 경우 true.
    pub fn delete(&self, id: i64) -> Result<bool, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute("DELETE FROM surfaces WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }
}

/// surface 행을 러스트 타입으로 매핑하는 헬퍼.
fn map_surface_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<SurfaceRow, StoreError>> {
    let kind_str: String = row.get("kind")?;
    let kind = SurfaceKind::from_str(&kind_str)
        .map_err(|e| StoreError::Corrupt(format!("invalid kind: {e}")));
    let id: i64 = row.get("id")?;
    let pane_id: i64 = row.get("pane_id")?;
    let state_json: Option<String> = row.get("state_json")?;
    let tab_order: i64 = row.get("tab_order")?;
    let created_at: String = row.get("created_at")?;
    let updated_at: String = row.get("updated_at")?;
    Ok(kind.map(|k| SurfaceRow {
        id,
        pane_id,
        kind: k,
        state_json,
        tab_order,
        created_at,
        updated_at,
    }))
}

/// 기존 Store 에 `surfaces()` DAO 접근 포인트를 붙이는 헬퍼 trait.
pub trait SurfaceStoreExt {
    /// 새 SurfaceDao 인스턴스를 반환한다 (내부 Arc 클론만 수행).
    fn surfaces(&self) -> SurfaceDao;
}

impl SurfaceStoreExt for crate::Store {
    fn surfaces(&self) -> SurfaceDao {
        SurfaceDao::new(Arc::clone(&self.conn))
    }
}
