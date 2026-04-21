-- V3: panes + surfaces 테이블 (SPEC-M2-001 RG-M2-1, RG-M2-2)

-- @MX:NOTE: [AUTO] pane binary tree 구조를 저장. split='leaf' 는 실제 surface 를 담는 단말 노드
CREATE TABLE IF NOT EXISTS panes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    parent_id INTEGER REFERENCES panes(id) ON DELETE CASCADE,
    split TEXT NOT NULL CHECK(split IN ('horizontal','vertical','leaf')),
    -- @MX:NOTE: [AUTO] ratio 기본값 0.5 = 균등 분할. CHECK 로 [0.0, 1.0] 범위 강제
    ratio REAL NOT NULL DEFAULT 0.5 CHECK(ratio >= 0.0 AND ratio <= 1.0),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_panes_workspace ON panes(workspace_id);
CREATE INDEX IF NOT EXISTS idx_panes_parent ON panes(parent_id);

-- @MX:NOTE: [AUTO] surface 는 leaf pane 의 탭 하나. kind 는 10종 중 하나
CREATE TABLE IF NOT EXISTS surfaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pane_id INTEGER NOT NULL REFERENCES panes(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('terminal','code','markdown','image','browser','filetree','agent_run','kanban','memory','instructions_graph')),
    state_json TEXT,
    tab_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_surfaces_pane ON surfaces(pane_id);
CREATE INDEX IF NOT EXISTS idx_surfaces_tab_order ON surfaces(pane_id, tab_order);
