//! SPEC-V3-009 RG-SU-1.4 — pulldown-cmark 기반 spec.md EARS/AC 표 파싱.
//!
//! USER-DECISION-SU-A: pulldown-cmark v0.13 채택.
//!
//! ## 파싱 대상
//! - frontmatter YAML (--- ... ---) → id, version, status, milestones
//! - `### RG-{group}-{nnn}` heading 직후 표 → EARS 요구사항 (`| REQ ID | 패턴 | ... |`)
//! - `| AC ID | ...` 표 → AC (Acceptance Criteria)
//! - `^## \d+\.\d+ Sprint Contract Revision` heading → SprintContractRevision

mod ac;
pub mod ears;
mod frontmatter;
mod sprint_contract;

pub use ac::{AcRow, parse_ac_tables};
pub use ears::{Requirement, RequirementGroup};
pub use frontmatter::{SpecFrontmatter, parse_frontmatter};
pub use sprint_contract::{SprintContractRevision, parse_sprint_contracts};

/// spec.md 한 문서 파싱 결과.
#[derive(Debug, Clone)]
pub struct ParsedSpec {
    /// frontmatter YAML 파싱 결과. YAML 없거나 파싱 실패 시 default.
    pub frontmatter: SpecFrontmatter,
    /// EARS 요구사항 그룹 목록 (RG-SU-1.4).
    pub requirement_groups: Vec<RequirementGroup>,
    /// AC 표 행 목록 (RG-SU-2).
    pub ac_rows: Vec<AcRow>,
    /// Sprint Contract Revision 목록 (RG-SU-6).
    pub sprint_contracts: Vec<SprintContractRevision>,
}

/// spec.md 텍스트를 파싱하여 [`ParsedSpec`] 을 반환한다 (REQ-SU-004).
///
/// 파싱 실패 시 panic 하지 않고 graceful 결과를 반환한다 (REQ-SU-005).
pub fn parse_spec_md(text: &str) -> ParsedSpec {
    // 1. frontmatter 분리
    let (frontmatter_str, body) = split_frontmatter(text);
    let frontmatter = parse_frontmatter(frontmatter_str);

    // 2. body 에서 EARS 표 + AC 표 + Sprint Contract 파싱
    let requirement_groups = ears::parse_ears_tables(body);
    let ac_rows = parse_ac_tables(body);
    let sprint_contracts = parse_sprint_contracts(body);

    ParsedSpec {
        frontmatter,
        requirement_groups,
        ac_rows,
        sprint_contracts,
    }
}

/// spec.md 텍스트에서 `---\n...\n---` frontmatter 블록과 본문을 분리한다.
///
/// frontmatter 가 없으면 `(None, text)` 반환.
fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    let text = text.trim_start();
    if !text.starts_with("---") {
        return (None, text);
    }
    // `---\n` 이후부터 다음 `---` 찾기
    let after_open = match text.find('\n') {
        Some(i) => &text[i + 1..],
        None => return (None, text),
    };
    // 줄 시작에서만 `---` 인식
    let close = after_open
        .lines()
        .enumerate()
        .position(|(_, line)| line.trim() == "---");

    match close {
        Some(line_idx) => {
            let fm_bytes: usize = after_open.lines().take(line_idx).map(|l| l.len() + 1).sum();
            let fm = &after_open[..fm_bytes.saturating_sub(1)]; // trailing \n 제거
            let body_start = fm_bytes + 4; // "---\n" 길이
            let body = if body_start <= after_open.len() {
                &after_open[body_start..]
            } else {
                ""
            };
            (Some(fm), body)
        }
        None => (None, text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_with_yaml() {
        let text = "---\nid: SPEC-X\nstatus: draft\n---\n# Title\n";
        let (fm, body) = split_frontmatter(text);
        assert!(fm.is_some());
        assert!(fm.unwrap().contains("SPEC-X"));
        assert!(body.starts_with("# Title"));
    }

    #[test]
    fn split_frontmatter_without_yaml() {
        let text = "# No frontmatter\nSome content\n";
        let (fm, body) = split_frontmatter(text);
        assert!(fm.is_none());
        assert!(body.starts_with("# No frontmatter"));
    }

    #[test]
    fn parse_spec_md_does_not_panic_on_empty() {
        let result = parse_spec_md("");
        assert!(result.requirement_groups.is_empty());
        assert!(result.ac_rows.is_empty());
    }

    #[test]
    fn parse_spec_md_does_not_panic_on_malformed() {
        let text = "| broken | table | with no\ncontent\n";
        let _result = parse_spec_md(text); // should not panic
    }
}
