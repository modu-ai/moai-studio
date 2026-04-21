//! Pane CRUD (SPEC-M2-001 RG-M2-1 산출물).
//!
//! NSSplitView binary tree 의 노드를 DB 에 영속한다.
//! split 필드는 `SplitKind` enum 으로 관리된다.

use std::str::FromStr;
use std::sync::Arc;

use rusqlite::{OptionalExtension, params};

use crate::{SharedConn, StoreError};

/// pane 의 분할 방향.
// @MX:NOTE: [AUTO] 'leaf' 는 실제 surface 를 담는 단말 노드; 'horizontal'/'vertical' 은 분할 노드
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitKind {
    /// 좌-우 분할
    Horizontal,
    /// 상-하 분할
    Vertical,
    /// 단말 노드 (surface 를 포함)
    Leaf,
}

impl SplitKind {
    /// DB 저장용 문자열로 변환.
    pub fn as_str(&self) -> &'static str {
        match self {
            SplitKind::Horizontal => "horizontal",
            SplitKind::Vertical => "vertical",
            SplitKind::Leaf => "leaf",
        }
    }
}

impl FromStr for SplitKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "horizontal" => Ok(SplitKind::Horizontal),
            "vertical" => Ok(SplitKind::Vertical),
            "leaf" => Ok(SplitKind::Leaf),
            other => Err(format!("알 수 없는 SplitKind: {other}")),
        }
    }
}

/// pane 한 행의 전체 상태.
// @MX:ANCHOR: [AUTO] Rust core 와 Swift FFI 사이의 pane 도메인 객체 (fan_in>=3)
// @MX:REASON: [AUTO] PaneDao::insert, list_by_workspace, FFI pane.rs 세 경로에서 사용
#[derive(Debug, Clone, PartialEq)]
pub struct PaneRow {
    /// SQLite rowid (AUTOINCREMENT)
    pub id: i64,
    /// 소속 워크스페이스 id
    pub workspace_id: i64,
    /// 부모 pane id. 루트 pane 은 None
    pub parent_id: Option<i64>,
    /// 분할 방향
    pub split: SplitKind,
    /// 분할 비율 (0.0 ~ 1.0)
    pub ratio: f64,
    /// 생성 시각 (ISO8601)
    pub created_at: String,
    /// 마지막 수정 시각 (ISO8601)
    pub updated_at: String,
}

/// 신규 삽입용 파라미터.
#[derive(Debug, Clone)]
pub struct NewPane {
    /// 소속 워크스페이스 id
    pub workspace_id: i64,
    /// 부모 pane id. 루트이면 None
    pub parent_id: Option<i64>,
    /// 분할 방향
    pub split: SplitKind,
    /// 분할 비율 (0.0 ~ 1.0)
    pub ratio: f64,
}

/// Pane CRUD 파사드.
pub struct PaneDao {
    conn: SharedConn,
}

impl PaneDao {
    pub(crate) fn new(conn: SharedConn) -> Self {
        Self { conn }
    }

    /// 새 pane 을 삽입하고 생성된 행을 반환한다.
    // @MX:ANCHOR: [AUTO] pane 생성 진입점 (fan_in>=3: T-033 DAO, T-035 FFI, T-037 통합테스트)
    // @MX:REASON: [AUTO] 워크스페이스별 binary tree 구성의 최초 노드 생성 API
    pub fn insert(&self, new: &NewPane) -> Result<PaneRow, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        guard.execute(
            "INSERT INTO panes (workspace_id, parent_id, split, ratio) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                new.workspace_id,
                new.parent_id,
                new.split.as_str(),
                new.ratio
            ],
        )?;
        let id = guard.last_insert_rowid();
        drop(guard);
        self.get(id)?.ok_or(StoreError::NotFound(id))
    }

    /// id 로 조회. 없으면 `Ok(None)`.
    pub fn get(&self, id: i64) -> Result<Option<PaneRow>, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let row = guard
            .query_row(
                "SELECT id, workspace_id, parent_id, split, ratio, created_at, updated_at \
                 FROM panes WHERE id = ?1",
                params![id],
                map_pane_row,
            )
            .optional()?;
        row.transpose()
    }

    /// 워크스페이스 ID 기준 전체 pane 목록.
    pub fn list_by_workspace(&self, workspace_id: i64) -> Result<Vec<PaneRow>, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let mut stmt = guard.prepare(
            "SELECT id, workspace_id, parent_id, split, ratio, created_at, updated_at \
             FROM panes WHERE workspace_id = ?1 ORDER BY id",
        )?;
        let rows = stmt
            .query_map(params![workspace_id], map_pane_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().collect::<Result<Vec<_>, _>>()
    }

    /// ratio 를 업데이트하고 갱신된 행을 반환한다.
    pub fn update_ratio(&self, id: i64, ratio: f64) -> Result<PaneRow, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute(
            "UPDATE panes SET ratio = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![ratio, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(id));
        }
        drop(guard);
        self.get(id)?.ok_or(StoreError::NotFound(id))
    }

    /// pane 을 삭제한다. 삭제된 경우 true.
    pub fn delete(&self, id: i64) -> Result<bool, StoreError> {
        let guard = self.conn.lock().map_err(|_| StoreError::PoisonedLock)?;
        let n = guard.execute("DELETE FROM panes WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }
}

/// pane 행을 러스트 타입으로 매핑하는 헬퍼.
fn map_pane_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<PaneRow, StoreError>> {
    let split_str: String = row.get("split")?;
    let split = SplitKind::from_str(&split_str)
        .map_err(|e| StoreError::Corrupt(format!("invalid split: {e}")));
    let id: i64 = row.get("id")?;
    let workspace_id: i64 = row.get("workspace_id")?;
    let parent_id: Option<i64> = row.get("parent_id")?;
    let ratio: f64 = row.get("ratio")?;
    let created_at: String = row.get("created_at")?;
    let updated_at: String = row.get("updated_at")?;
    Ok(split.map(|s| PaneRow {
        id,
        workspace_id,
        parent_id,
        split: s,
        ratio,
        created_at,
        updated_at,
    }))
}

/// 기존 Store 에 `panes()` DAO 접근 포인트를 붙이는 헬퍼 trait.
pub trait PaneStoreExt {
    /// 새 PaneDao 인스턴스를 반환한다 (내부 Arc 클론만 수행).
    fn panes(&self) -> PaneDao;
}

impl PaneStoreExt for crate::Store {
    fn panes(&self) -> PaneDao {
        PaneDao::new(Arc::clone(&self.conn))
    }
}
