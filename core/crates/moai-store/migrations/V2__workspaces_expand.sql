-- V2: workspaces 테이블 확장 (SPEC-M1-001 RG-M1-4)
--
-- 추가 컬럼: name, project_path, worktree_path, status (6-state), spec_id,
--           claude_session_id, updated_at. 기존 v1 컬럼(id, working_dir, state,
--           created_at)은 호환성을 위해 유지한다.

ALTER TABLE workspaces ADD COLUMN name TEXT NOT NULL DEFAULT '';
ALTER TABLE workspaces ADD COLUMN project_path TEXT NOT NULL DEFAULT '';
ALTER TABLE workspaces ADD COLUMN worktree_path TEXT;
ALTER TABLE workspaces ADD COLUMN status TEXT NOT NULL DEFAULT 'Created';
ALTER TABLE workspaces ADD COLUMN spec_id TEXT;
ALTER TABLE workspaces ADD COLUMN claude_session_id TEXT;
ALTER TABLE workspaces ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));

CREATE INDEX IF NOT EXISTS idx_workspaces_status ON workspaces(status);
CREATE INDEX IF NOT EXISTS idx_workspaces_name ON workspaces(name);
