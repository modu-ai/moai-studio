//! SPEC-V3-009 RG-SU-1 — SpecIndex: `.moai/specs/` 스캔 + SPEC 목록 관리.
//!
//! REQ-SU-001: active workspace 의 `.moai/specs/SPEC-*/` 를 1-depth 스캔하여
//!             `Vec<SpecRecord>` 를 생성한다.
//! REQ-SU-002: canonical 파일 존재 여부를 `SpecRecord.files` 에 기록한다.
//! REQ-SU-005: canonical 파일이 없는 SPEC 도 panic 없이 처리한다.

use std::path::{Path, PathBuf};

use tracing::{info, warn};

use crate::parser::parse_spec_md;
use crate::state::ac_state::parse_ac_states_from_progress;
use crate::state::kanban::KanbanStage;
use crate::state::spec_record::{SpecFileKind, SpecId, SpecRecord};

/// `.moai/specs/` 디렉터리의 모든 SPEC 을 관리하는 인덱스.
///
/// # @MX:ANCHOR: [AUTO] SpecIndex
/// @MX:REASON: [AUTO] SPEC-V3-009 RG-SU-1 진입점.
///   fan_in >= 3: SpecWatcher::scan, spec_ui::list_view, integration tests.
#[derive(Debug, Default)]
pub struct SpecIndex {
    /// 스캔된 SPEC 목록 (ID 순 정렬)
    pub records: Vec<SpecRecord>,
    /// 스캔한 베이스 디렉터리 (`.moai/specs/`)
    pub specs_dir: Option<PathBuf>,
}

impl SpecIndex {
    /// 신규 SpecIndex 생성 (비어 있음).
    pub fn new() -> Self {
        Self::default()
    }

    /// `.moai/specs/` 디렉터리를 1-depth 스캔하여 SpecRecord 목록을 갱신한다.
    ///
    /// `specs_dir` 가 존재하지 않으면 빈 목록으로 graceful 반환 (REQ-SU-005).
    pub fn scan(&mut self, specs_dir: &Path) {
        self.specs_dir = Some(specs_dir.to_path_buf());

        let dir_entries = match std::fs::read_dir(specs_dir) {
            Ok(e) => e,
            Err(e) => {
                warn!("`.moai/specs/` 스캔 실패 (graceful skip): {e}");
                self.records.clear();
                return;
            }
        };

        let mut records: Vec<SpecRecord> = Vec::new();

        for entry in dir_entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            // SPEC-{area}-{nnn} 패턴만 인식
            if !dir_name.starts_with("SPEC-") {
                continue;
            }

            let spec_id = SpecId::new(&dir_name);
            let mut record = SpecRecord::new(spec_id, dir_name.clone(), path.clone());

            // canonical 파일 존재 여부 기록 (REQ-SU-002)
            for &kind in SpecFileKind::all() {
                let file_path = path.join(kind.filename());
                let value = if file_path.exists() {
                    Some(file_path)
                } else {
                    None
                };
                record.files.insert(kind, value);
            }

            // spec.md 파싱 (REQ-SU-004)
            if let Some(spec_path) = record.file_path(SpecFileKind::Spec).cloned() {
                match std::fs::read_to_string(&spec_path) {
                    Ok(content) => {
                        let parsed = parse_spec_md(&content);
                        // title 추출: frontmatter id 우선, 없으면 첫 H1 heading
                        let title = parsed
                            .frontmatter
                            .id
                            .clone()
                            .unwrap_or_else(|| dir_name.clone());
                        record.title = title;
                        record.apply_parsed_spec(&parsed);
                    }
                    Err(e) => {
                        warn!("spec.md 읽기 실패 (graceful skip): {spec_path:?}: {e}");
                    }
                }
            }

            // progress.md AC 상태 파싱 (REQ-SU-011)
            if let Some(progress_path) = record.file_path(SpecFileKind::Progress).cloned() {
                match std::fs::read_to_string(&progress_path) {
                    Ok(content) => {
                        let ac_records = parse_ac_states_from_progress(&content);
                        record.apply_ac_states(ac_records);
                    }
                    Err(e) => {
                        warn!("progress.md 읽기 실패 (graceful skip): {progress_path:?}: {e}");
                    }
                }
            }

            // .kanban-stage sidecar 읽기 (REQ-SU-021)
            let sidecar_path = path.join(".kanban-stage");
            if sidecar_path.exists() {
                match std::fs::read_to_string(&sidecar_path) {
                    Ok(content) => {
                        record.kanban_stage = KanbanStage::from_sidecar(content.trim());
                    }
                    Err(e) => {
                        warn!(".kanban-stage 읽기 실패 (graceful skip): {sidecar_path:?}: {e}");
                    }
                }
            }

            records.push(record);
            info!("SPEC 스캔 완료: {dir_name}");
        }

        // ID 순 정렬 (일관된 표시 순서)
        records.sort_by(|a, b| a.id.cmp(&b.id));
        self.records = records;
    }

    /// SPEC ID 로 record 를 찾는다.
    pub fn find(&self, id: &SpecId) -> Option<&SpecRecord> {
        self.records.iter().find(|r| &r.id == id)
    }

    /// SPEC ID 로 record 를 가변 참조로 찾는다.
    pub fn find_mut(&mut self, id: &SpecId) -> Option<&mut SpecRecord> {
        self.records.iter_mut().find(|r| &r.id == id)
    }

    /// 단일 SPEC 을 재스캔한다 (REQ-SU-003 debounce 이후 호출).
    ///
    /// 지정 SPEC 의 spec.md 와 progress.md 를 다시 읽어 records 를 갱신한다.
    /// panic 없이 graceful 처리한다 (REQ-SU-005).
    pub fn rescan_one(&mut self, id: &SpecId) {
        let record = match self.records.iter_mut().find(|r| &r.id == id) {
            Some(r) => r,
            None => {
                warn!("rescan_one: SPEC ID {id} 를 인덱스에서 찾지 못함 (graceful skip)");
                return;
            }
        };

        // spec.md 재파싱
        if let Some(spec_path) = record.file_path(SpecFileKind::Spec).cloned() {
            match std::fs::read_to_string(&spec_path) {
                Ok(content) => {
                    let parsed = parse_spec_md(&content);
                    record.apply_parsed_spec(&parsed);
                }
                Err(e) => {
                    warn!("spec.md 재읽기 실패 (graceful skip): {spec_path:?}: {e}");
                }
            }
        }

        // progress.md 재파싱
        if let Some(progress_path) = record.file_path(SpecFileKind::Progress).cloned() {
            match std::fs::read_to_string(&progress_path) {
                Ok(content) => {
                    let ac_records = parse_ac_states_from_progress(&content);
                    record.apply_ac_states(ac_records);
                }
                Err(e) => {
                    warn!("progress.md 재읽기 실패 (graceful skip): {progress_path:?}: {e}");
                }
            }
        }
    }

    /// 전체 SPEC 수.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 비어 있는지 여부.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// 테스트용 `.moai/specs/` 구조 생성 헬퍼.
    fn make_specs_dir() -> TempDir {
        tempfile::tempdir().expect("tempdir 생성 실패")
    }

    fn make_spec_dir(parent: &Path, spec_id: &str) -> PathBuf {
        let dir = parent.join(spec_id);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scan_empty_dir_returns_empty() {
        let tmp = make_specs_dir();
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        assert!(index.is_empty());
    }

    #[test]
    fn scan_single_spec_dir_creates_record() {
        let tmp = make_specs_dir();
        let _spec_dir = make_spec_dir(tmp.path(), "SPEC-V3-009");
        // spec.md 없어도 graceful
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        assert_eq!(index.len(), 1);
        assert_eq!(index.records[0].id.as_str(), "SPEC-V3-009");
    }

    #[test]
    fn scan_non_spec_dir_ignored() {
        let tmp = make_specs_dir();
        // SPEC- prefix 없는 디렉터리
        make_spec_dir(tmp.path(), "not-a-spec");
        make_spec_dir(tmp.path(), "SPEC-V3-001");
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        assert_eq!(index.len(), 1, "SPEC- prefix 없는 디렉터리는 무시");
    }

    #[test]
    fn scan_records_canonical_files_existence() {
        let tmp = make_specs_dir();
        let spec_dir = make_spec_dir(tmp.path(), "SPEC-V3-010");
        // spec.md 와 progress.md 만 생성
        fs::write(spec_dir.join("spec.md"), "---\nid: SPEC-V3-010\n---\n# T").unwrap();
        fs::write(spec_dir.join("progress.md"), "AC-SU-1: PASS\n").unwrap();

        let mut index = SpecIndex::new();
        index.scan(tmp.path());

        let record = index
            .records
            .iter()
            .find(|r| r.id.as_str() == "SPEC-V3-010")
            .unwrap();
        assert!(record.has_file(SpecFileKind::Spec));
        assert!(record.has_file(SpecFileKind::Progress));
        assert!(
            !record.has_file(SpecFileKind::Acceptance),
            "acceptance.md 없음"
        );
        assert!(!record.has_file(SpecFileKind::Plan), "plan.md 없음");
    }

    #[test]
    fn scan_parses_spec_md_frontmatter() {
        let tmp = make_specs_dir();
        let spec_dir = make_spec_dir(tmp.path(), "SPEC-V3-011");
        fs::write(
            spec_dir.join("spec.md"),
            "---\nid: SPEC-V3-011\nstatus: approved\n---\n# My Spec\n",
        )
        .unwrap();
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        let r = &index.records[0];
        assert_eq!(r.spec_status.as_deref(), Some("approved"));
    }

    #[test]
    fn scan_reads_progress_ac_states() {
        let tmp = make_specs_dir();
        let spec_dir = make_spec_dir(tmp.path(), "SPEC-V3-012");
        fs::write(spec_dir.join("spec.md"), "---\nid: SPEC-V3-012\n---\n").unwrap();
        fs::write(spec_dir.join("progress.md"), "AC-P-1: PASS\nAC-P-2: FAIL\n").unwrap();
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        let r = &index.records[0];
        assert_eq!(r.ac_records.len(), 2);
        let s = r.ac_summary();
        assert_eq!(s.full, 1);
        assert_eq!(s.fail, 1);
    }

    #[test]
    fn scan_reads_kanban_stage_sidecar() {
        let tmp = make_specs_dir();
        let spec_dir = make_spec_dir(tmp.path(), "SPEC-V3-013");
        fs::write(spec_dir.join(".kanban-stage"), "in-progress").unwrap();
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        let r = &index.records[0];
        assert_eq!(r.kanban_stage, KanbanStage::InProgress);
    }

    #[test]
    fn scan_missing_kanban_stage_defaults_to_todo() {
        let tmp = make_specs_dir();
        make_spec_dir(tmp.path(), "SPEC-V3-014");
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        let r = &index.records[0];
        assert_eq!(r.kanban_stage, KanbanStage::Todo);
    }

    #[test]
    fn scan_sorts_records_by_id() {
        let tmp = make_specs_dir();
        make_spec_dir(tmp.path(), "SPEC-V3-003");
        make_spec_dir(tmp.path(), "SPEC-V3-001");
        make_spec_dir(tmp.path(), "SPEC-V3-002");
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        let ids: Vec<_> = index.records.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["SPEC-V3-001", "SPEC-V3-002", "SPEC-V3-003"]);
    }

    #[test]
    fn find_returns_correct_record() {
        let tmp = make_specs_dir();
        make_spec_dir(tmp.path(), "SPEC-V3-020");
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        let id = SpecId::new("SPEC-V3-020");
        assert!(index.find(&id).is_some());
    }

    #[test]
    fn find_returns_none_for_unknown_id() {
        let index = SpecIndex::new();
        let id = SpecId::new("SPEC-UNKNOWN");
        assert!(index.find(&id).is_none());
    }

    #[test]
    fn rescan_one_updates_spec_content() {
        let tmp = make_specs_dir();
        let spec_dir = make_spec_dir(tmp.path(), "SPEC-V3-030");
        // 초기 spec.md
        fs::write(
            spec_dir.join("spec.md"),
            "---\nid: SPEC-V3-030\nstatus: draft\n---\n",
        )
        .unwrap();
        let mut index = SpecIndex::new();
        index.scan(tmp.path());
        assert_eq!(index.records[0].spec_status.as_deref(), Some("draft"));

        // spec.md 변경
        fs::write(
            spec_dir.join("spec.md"),
            "---\nid: SPEC-V3-030\nstatus: approved\n---\n",
        )
        .unwrap();
        // files 맵에 path 가 있어야 rescan_one 이 동작함
        index.records[0]
            .files
            .insert(SpecFileKind::Spec, Some(spec_dir.join("spec.md")));
        let id = SpecId::new("SPEC-V3-030");
        index.rescan_one(&id);
        assert_eq!(index.records[0].spec_status.as_deref(), Some("approved"));
    }

    #[test]
    fn scan_nonexistent_specs_dir_graceful() {
        let mut index = SpecIndex::new();
        index.scan(Path::new("/nonexistent/path/specs"));
        assert!(index.is_empty(), "존재하지 않는 경로는 graceful empty");
    }
}
