//! SPEC-V3-009 RG-SU-1 — SpecId, SpecFileKind, SpecRecord.
//!
//! REQ-SU-001: `Vec<SpecRecord>` 생성 시 사용하는 SPEC 단위 자료구조.
//! REQ-SU-002: canonical 파일 존재 여부를 `SpecRecord.files` 로 기록.
//! REQ-SU-014: `SpecRecord::ac_summary()` 노출.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::parser::{AcRow, ParsedSpec, RequirementGroup, SprintContractRevision};
use crate::state::{AcRecord, AcSummary, KanbanStage};

/// SPEC ID 래퍼 (예: "SPEC-V3-009") (REQ-SU-001).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpecId(pub String);

impl SpecId {
    /// 새 SpecId 생성.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 내부 문자열 참조.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SpecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SPEC 디렉터리 내 canonical 파일 종류 (REQ-SU-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecFileKind {
    Spec,
    Plan,
    Research,
    Acceptance,
    Contract,
    Progress,
    Tasks,
}

impl SpecFileKind {
    /// 파일 이름 문자열.
    pub fn filename(&self) -> &'static str {
        match self {
            SpecFileKind::Spec => "spec.md",
            SpecFileKind::Plan => "plan.md",
            SpecFileKind::Research => "research.md",
            SpecFileKind::Acceptance => "acceptance.md",
            SpecFileKind::Contract => "contract.md",
            SpecFileKind::Progress => "progress.md",
            SpecFileKind::Tasks => "tasks.md",
        }
    }

    /// 모든 canonical 파일 목록.
    pub fn all() -> &'static [SpecFileKind] {
        &[
            SpecFileKind::Spec,
            SpecFileKind::Plan,
            SpecFileKind::Research,
            SpecFileKind::Acceptance,
            SpecFileKind::Contract,
            SpecFileKind::Progress,
            SpecFileKind::Tasks,
        ]
    }
}

/// 단일 SPEC 디렉터리의 파싱+상태 기록 (REQ-SU-001~REQ-SU-014).
///
/// # @MX:ANCHOR: [AUTO] SpecRecord
/// @MX:REASON: [AUTO] SPEC-V3-009 §12 외부 인터페이스 고정 구조체.
///   fan_in >= 3: SpecIndex::scan, spec_ui::list_view, spec_ui::detail_view.
#[derive(Debug, Clone)]
pub struct SpecRecord {
    /// SPEC ID
    pub id: SpecId,
    /// SPEC 제목 (spec.md 첫 H1 또는 frontmatter 에서 추출)
    pub title: String,
    /// SPEC 디렉터리 절대 경로
    pub dir_path: PathBuf,
    /// canonical 파일 존재 여부 맵 (REQ-SU-002)
    pub files: HashMap<SpecFileKind, Option<PathBuf>>,
    /// EARS 요구사항 그룹 목록 (spec.md 파싱 결과)
    pub requirement_groups: Vec<RequirementGroup>,
    /// AC 표 행 목록 (spec.md 파싱 결과)
    pub ac_rows: Vec<AcRow>,
    /// AC 상태 기록 (progress.md 파싱 결과, default: Pending)
    pub ac_records: Vec<AcRecord>,
    /// Sprint Contract Revision 목록
    pub sprint_contract_revisions: Vec<SprintContractRevision>,
    /// Kanban stage (`.kanban-stage` sidecar 파싱)
    pub kanban_stage: KanbanStage,
    /// spec.md 상태 문자열 (frontmatter.status)
    pub spec_status: Option<String>,
}

impl SpecRecord {
    /// 신규 SpecRecord 생성 (최소 필드만).
    pub fn new(id: SpecId, title: String, dir_path: PathBuf) -> Self {
        Self {
            id,
            title,
            dir_path,
            files: HashMap::new(),
            requirement_groups: Vec::new(),
            ac_rows: Vec::new(),
            ac_records: Vec::new(),
            sprint_contract_revisions: Vec::new(),
            kanban_stage: KanbanStage::Todo,
            spec_status: None,
        }
    }

    /// `ParsedSpec` 내용을 SpecRecord 에 반영한다.
    ///
    /// spec.md 파싱 결과로 requirement_groups, ac_rows, sprint_contract_revisions,
    /// spec_status, title 을 갱신한다.
    pub fn apply_parsed_spec(&mut self, parsed: &ParsedSpec) {
        self.requirement_groups = parsed.requirement_groups.clone();
        self.ac_rows = parsed.ac_rows.clone();
        self.sprint_contract_revisions = parsed.sprint_contracts.clone();
        self.spec_status = parsed.frontmatter.status.clone();

        // frontmatter id 가 있으면 title 보정
        if let Some(id_str) = &parsed.frontmatter.id {
            // title 이 단순 ID 만인 경우 spec 첫 H1 으로 보정은 별도 — 여기선 유지
            let _ = id_str; // 현재는 id 로 title override 안 함
        }
    }

    /// progress.md 파싱 결과로 ac_records 를 갱신한다.
    pub fn apply_ac_states(&mut self, records: Vec<AcRecord>) {
        self.ac_records = records;
    }

    /// AC 요약을 계산하여 반환한다 (REQ-SU-014).
    pub fn ac_summary(&self) -> AcSummary {
        if !self.ac_records.is_empty() {
            // progress.md 기반 상태
            AcSummary::from_records(&self.ac_records)
        } else {
            // spec.md 의 AC 표에서 count (모두 Pending)
            let pending_records: Vec<AcRecord> =
                self.ac_rows.iter().map(|r| AcRecord::new(&r.id)).collect();
            AcSummary::from_records(&pending_records)
        }
    }

    /// REQ 총 개수 (모든 RG 의 합계).
    pub fn req_count(&self) -> usize {
        self.requirement_groups
            .iter()
            .map(|g| g.requirements.len())
            .sum()
    }

    /// RG 개수.
    pub fn rg_count(&self) -> usize {
        self.requirement_groups.len()
    }

    /// AC 총 개수 (spec.md 표 기준).
    pub fn ac_count(&self) -> usize {
        self.ac_rows.len()
    }

    /// 지정 파일 kind 가 존재하는지 확인한다.
    pub fn has_file(&self, kind: SpecFileKind) -> bool {
        self.files.get(&kind).and_then(|v| v.as_ref()).is_some()
    }

    /// 지정 파일 kind 의 경로 반환 (없으면 None).
    pub fn file_path(&self, kind: SpecFileKind) -> Option<&PathBuf> {
        self.files.get(&kind).and_then(|v| v.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::AcRow;

    fn make_record(id: &str) -> SpecRecord {
        SpecRecord::new(
            SpecId::new(id),
            format!("{id} 제목"),
            PathBuf::from(format!("/tmp/specs/{id}")),
        )
    }

    #[test]
    fn spec_id_new_and_display() {
        let id = SpecId::new("SPEC-V3-009");
        assert_eq!(id.as_str(), "SPEC-V3-009");
        assert_eq!(id.to_string(), "SPEC-V3-009");
    }

    #[test]
    fn spec_file_kind_filename() {
        assert_eq!(SpecFileKind::Spec.filename(), "spec.md");
        assert_eq!(SpecFileKind::Progress.filename(), "progress.md");
        assert_eq!(SpecFileKind::Acceptance.filename(), "acceptance.md");
    }

    #[test]
    fn spec_file_kind_all_has_seven_entries() {
        assert_eq!(SpecFileKind::all().len(), 7);
    }

    #[test]
    fn spec_record_new_defaults() {
        let r = make_record("SPEC-V3-001");
        assert_eq!(r.id.as_str(), "SPEC-V3-001");
        assert_eq!(r.kanban_stage, KanbanStage::Todo);
        assert!(r.requirement_groups.is_empty());
        assert!(r.ac_rows.is_empty());
        assert!(r.ac_records.is_empty());
    }

    #[test]
    fn ac_summary_with_no_ac_records_uses_ac_rows() {
        let mut r = make_record("SPEC-V3-002");
        r.ac_rows = vec![
            AcRow {
                id: "AC-P-1".to_string(),
                scenario: String::new(),
                pass_condition: String::new(),
                verification: None,
                rg_mapping: None,
            },
            AcRow {
                id: "AC-P-2".to_string(),
                scenario: String::new(),
                pass_condition: String::new(),
                verification: None,
                rg_mapping: None,
            },
        ];
        let s = r.ac_summary();
        assert_eq!(s.pending, 2);
        assert_eq!(s.full, 0);
    }

    #[test]
    fn ac_summary_with_ac_records() {
        let mut r = make_record("SPEC-V3-003");
        r.ac_records = vec![
            AcRecord::new("AC-P-1").with_label("PASS"),
            AcRecord::new("AC-P-2").with_label("FAIL"),
            AcRecord::new("AC-P-3"),
        ];
        let s = r.ac_summary();
        assert_eq!(s.full, 1);
        assert_eq!(s.fail, 1);
        assert_eq!(s.pending, 1);
    }

    #[test]
    fn req_count_sums_across_groups() {
        use crate::parser::ears::{Requirement, RequirementGroup};
        let mut r = make_record("SPEC-V3-004");
        r.requirement_groups = vec![
            RequirementGroup {
                id: "RG-1".to_string(),
                title: "G1".to_string(),
                requirements: vec![
                    Requirement {
                        id: "REQ-1".to_string(),
                        pattern: String::new(),
                        korean: String::new(),
                        english: None,
                    },
                    Requirement {
                        id: "REQ-2".to_string(),
                        pattern: String::new(),
                        korean: String::new(),
                        english: None,
                    },
                ],
            },
            RequirementGroup {
                id: "RG-2".to_string(),
                title: "G2".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-3".to_string(),
                    pattern: String::new(),
                    korean: String::new(),
                    english: None,
                }],
            },
        ];
        assert_eq!(r.req_count(), 3);
        assert_eq!(r.rg_count(), 2);
    }

    #[test]
    fn has_file_true_when_path_present() {
        let mut r = make_record("SPEC-V3-005");
        r.files
            .insert(SpecFileKind::Spec, Some(PathBuf::from("/tmp/spec.md")));
        r.files.insert(SpecFileKind::Acceptance, None);
        assert!(r.has_file(SpecFileKind::Spec));
        assert!(!r.has_file(SpecFileKind::Acceptance));
        assert!(!r.has_file(SpecFileKind::Progress));
    }

    #[test]
    fn has_file_false_when_none() {
        let mut r = make_record("SPEC-V3-006");
        r.files.insert(SpecFileKind::Acceptance, None);
        assert!(
            !r.has_file(SpecFileKind::Acceptance),
            "None 은 missing 으로 처리"
        );
    }
}
