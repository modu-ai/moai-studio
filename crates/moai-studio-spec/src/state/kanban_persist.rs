//! SPEC-V3-009 RG-SU-3 — `.kanban-stage` sidecar I/O (REQ-SU-021).
//!
//! 각 SPEC 디렉터리에 `.kanban-stage` 단일 라인 텍스트 파일을 읽고 쓴다.
//! 파일 없음 → `KanbanStage::Todo` fallback (REQ-SU-021).
//! 쓰기는 atomic rename (write .tmp → rename) — last-write-wins (R-SU-5 완화).

use std::io;
use std::path::Path;

use crate::state::kanban::KanbanStage;

/// `.kanban-stage` sidecar 파일에서 stage 를 읽는다.
///
/// - 파일이 없으면 `KanbanStage::Todo` 반환 (REQ-SU-021 fallback).
/// - 내용이 알 수 없는 문자열이면 `KanbanStage::Todo` fallback.
/// - I/O 오류는 graceful 처리: `KanbanStage::Todo` 반환.
///
/// # @MX:ANCHOR: [AUTO] read_stage
/// @MX:REASON: [AUTO] SPEC-V3-009 REQ-SU-021. sidecar 읽기 진입점.
///   fan_in >= 3: SpecIndex::scan, KanbanBoardView::new, scan_loads_kanban_stage_from_sidecar test.
pub fn read_stage(spec_dir: &Path) -> KanbanStage {
    let sidecar = spec_dir.join(".kanban-stage");
    match std::fs::read_to_string(&sidecar) {
        Ok(content) => KanbanStage::from_sidecar(content.trim()),
        Err(_) => KanbanStage::Todo,
    }
}

/// `.kanban-stage` sidecar 파일에 stage 를 쓴다.
///
/// atomic rename 패턴: `.kanban-stage.tmp` 에 먼저 쓴 후 rename.
/// last-write-wins 의미론 — 동시 쓰기 시 마지막 writer 가 우선 (R-SU-5 완화).
///
/// # @MX:ANCHOR: [AUTO] write_stage
/// @MX:REASON: [AUTO] SPEC-V3-009 REQ-SU-021. sidecar 쓰기 진입점.
///   fan_in >= 3: KanbanBoardView::handle_enter, write_then_read_roundtrip test, write_overwrites_existing test.
pub fn write_stage(spec_dir: &Path, stage: KanbanStage) -> io::Result<()> {
    let tmp_path = spec_dir.join(".kanban-stage.tmp");
    let final_path = spec_dir.join(".kanban-stage");
    // stage 라벨 + trailing newline
    let content = format!("{}\n", stage.to_sidecar());
    std::fs::write(&tmp_path, &content)?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(())
}

// ============================================================
// 단위 테스트 (RED phase 에서 먼저 작성)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// 테스트용 임시 spec 디렉터리 생성 헬퍼.
    fn tmp_spec_dir() -> TempDir {
        tempfile::tempdir().expect("tempdir 생성 실패")
    }

    // ── read_stage 테스트 ──────────────────────────────────────

    #[test]
    fn read_stage_returns_todo_when_file_missing() {
        // REQ-SU-021: 파일 없음 → Todo fallback
        let dir = tmp_spec_dir();
        let stage = read_stage(dir.path());
        assert_eq!(stage, KanbanStage::Todo, "파일 없음 → Todo fallback");
    }

    #[test]
    fn read_stage_returns_todo_when_malformed() {
        // REQ-SU-021: 인식 불가 내용 → Todo fallback
        let dir = tmp_spec_dir();
        fs::write(dir.path().join(".kanban-stage"), "GARBAGE_CONTENT").unwrap();
        let stage = read_stage(dir.path());
        assert_eq!(stage, KanbanStage::Todo, "알 수 없는 값 → Todo fallback");
    }

    #[test]
    fn read_stage_returns_todo_for_empty_file() {
        // 빈 파일도 malformed 로 처리 → Todo
        let dir = tmp_spec_dir();
        fs::write(dir.path().join(".kanban-stage"), "").unwrap();
        let stage = read_stage(dir.path());
        assert_eq!(stage, KanbanStage::Todo);
    }

    // ── write_then_read roundtrip ──────────────────────────────

    #[test]
    fn write_then_read_roundtrip_todo() {
        let dir = tmp_spec_dir();
        write_stage(dir.path(), KanbanStage::Todo).unwrap();
        assert_eq!(read_stage(dir.path()), KanbanStage::Todo);
    }

    #[test]
    fn write_then_read_roundtrip_in_progress() {
        let dir = tmp_spec_dir();
        write_stage(dir.path(), KanbanStage::InProgress).unwrap();
        assert_eq!(read_stage(dir.path()), KanbanStage::InProgress);
    }

    #[test]
    fn write_then_read_roundtrip_review() {
        let dir = tmp_spec_dir();
        write_stage(dir.path(), KanbanStage::Review).unwrap();
        assert_eq!(read_stage(dir.path()), KanbanStage::Review);
    }

    #[test]
    fn write_then_read_roundtrip_done() {
        let dir = tmp_spec_dir();
        write_stage(dir.path(), KanbanStage::Done).unwrap();
        assert_eq!(read_stage(dir.path()), KanbanStage::Done);
    }

    // ── last-write-wins ────────────────────────────────────────

    #[test]
    fn write_overwrites_existing() {
        // 두 번 쓰면 마지막 값이 남는다 (last-write-wins)
        let dir = tmp_spec_dir();
        write_stage(dir.path(), KanbanStage::InProgress).unwrap();
        write_stage(dir.path(), KanbanStage::Done).unwrap();
        assert_eq!(
            read_stage(dir.path()),
            KanbanStage::Done,
            "마지막 write 가 최종값"
        );
    }

    // ── atomic write 검증 ─────────────────────────────────────

    #[test]
    fn write_stage_leaves_no_tmp_file() {
        // atomic rename 후 .tmp 파일이 남지 않아야 한다
        let dir = tmp_spec_dir();
        write_stage(dir.path(), KanbanStage::Review).unwrap();
        let tmp = dir.path().join(".kanban-stage.tmp");
        assert!(!tmp.exists(), ".kanban-stage.tmp 파일이 남아 있으면 안 됨");
    }

    #[test]
    fn write_stage_file_content_has_trailing_newline() {
        // 파일 내용: "review\n"
        let dir = tmp_spec_dir();
        write_stage(dir.path(), KanbanStage::Review).unwrap();
        let content = fs::read_to_string(dir.path().join(".kanban-stage")).unwrap();
        assert!(
            content.ends_with('\n'),
            "trailing newline 이 있어야 함: {:?}",
            content
        );
        assert_eq!(content.trim(), "review");
    }
}
