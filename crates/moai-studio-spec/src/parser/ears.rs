//! SPEC-V3-009 RG-SU-1.4 — EARS 요구사항 표 파싱.
//!
//! 대상 패턴:
//! ```markdown
//! ### RG-{group}-{nnn} — {설명}
//!
//! | REQ ID | 패턴 | 요구사항 | 영문 보조 |
//! |--------|------|---------|-----------|
//! | REQ-P-001 | Ubiquitous | ... | ... |
//! ```

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use tracing::warn;

/// 단일 요구사항 행.
#[derive(Debug, Clone, PartialEq)]
pub struct Requirement {
    /// REQ ID (예: "REQ-P-001")
    pub id: String,
    /// EARS 패턴 (예: "Ubiquitous", "Event-Driven")
    pub pattern: String,
    /// 요구사항 한국어 본문
    pub korean: String,
    /// 영문 보조 텍스트 (optional)
    pub english: Option<String>,
}

/// EARS 요구사항 그룹 (RG-* 단위).
#[derive(Debug, Clone, PartialEq)]
pub struct RequirementGroup {
    /// 그룹 ID (예: "RG-P-1")
    pub id: String,
    /// 그룹 제목 (heading 텍스트)
    pub title: String,
    /// 이 그룹의 요구사항 목록
    pub requirements: Vec<Requirement>,
}

/// spec.md 본문에서 모든 EARS 요구사항 표를 파싱한다 (REQ-SU-004).
///
/// `### RG-*` heading 직후 첫 번째 표의 `REQ ID` 컬럼 헤더를 인식한다.
/// 파싱 실패 행은 warn 후 graceful skip (REQ-SU-005).
pub fn parse_ears_tables(text: &str) -> Vec<RequirementGroup> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(text, opts);

    let mut groups: Vec<RequirementGroup> = Vec::new();
    let mut current_rg: Option<(String, String)> = None; // (id, title)

    // 상태 머신
    let mut in_table = false;
    let mut is_req_table = false;
    let mut in_table_head = false;
    let mut header_cells: Vec<String> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell: String = String::new();
    let mut req_col: Option<usize> = None;
    let mut pattern_col: Option<usize> = None;
    let mut korean_col: Option<usize> = None;
    let mut english_col: Option<usize> = None;
    let mut current_reqs: Vec<Requirement> = Vec::new();
    let mut in_heading = false;
    let mut heading_level: Option<HeadingLevel> = None;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            // ── Heading 시작 ──
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_level = Some(level);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading {
                    let ht = heading_text.trim().to_string();
                    // `### RG-{id}` 또는 `### RG-{id} — {title}` 인식
                    if matches!(heading_level, Some(HeadingLevel::H3))
                        && ht.to_uppercase().contains("RG-")
                    {
                        // 이전 그룹 완료
                        if let Some((id, title)) = current_rg.take() {
                            groups.push(RequirementGroup {
                                id,
                                title,
                                requirements: std::mem::take(&mut current_reqs),
                            });
                        }
                        // 새 그룹 시작
                        let (rg_id, rg_title) = extract_rg_id_title(&ht);
                        current_rg = Some((rg_id, rg_title));
                    }
                    in_heading = false;
                    heading_level = None;
                }
            }

            // ── Table ──
            Event::Start(Tag::Table(_)) => {
                in_table = true;
                is_req_table = false;
                header_cells.clear();
                req_col = None;
                pattern_col = None;
                korean_col = None;
                english_col = None;
            }
            Event::End(TagEnd::Table) => {
                in_table = false;
                is_req_table = false;
            }
            Event::Start(Tag::TableHead) => {
                in_table_head = true;
                header_cells.clear();
            }
            Event::End(TagEnd::TableHead) => {
                in_table_head = false;
                if in_table && current_rg.is_some() {
                    // header 분석: 첫 셀이 "REQ ID" 혹은 "REQ" 로 시작하면 요구사항 표
                    let first_cell = header_cells
                        .first()
                        .map(|s| s.to_uppercase())
                        .unwrap_or_default();
                    if first_cell.starts_with("REQ") {
                        is_req_table = true;
                        req_col = header_cells
                            .iter()
                            .position(|h| h.to_uppercase().starts_with("REQ"));
                        pattern_col = header_cells.iter().position(|h| {
                            h.to_uppercase().contains("패턴") || h.to_uppercase() == "PATTERN"
                        });
                        korean_col = header_cells.iter().position(|h| {
                            let u = h.to_uppercase();
                            u.contains("한국어") || u.contains("요구사항") || u.contains("KOREAN")
                        });
                        english_col = header_cells.iter().position(|h| {
                            let u = h.to_uppercase();
                            u.contains("영문") || u.contains("ENGLISH")
                        });
                    }
                }
            }
            Event::Start(Tag::TableRow) => {
                if !in_table_head {
                    current_row.clear();
                    current_cell.clear();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if !in_table_head && is_req_table {
                    let row = &current_row;
                    let get = |idx: Option<usize>| -> Option<&str> {
                        idx.and_then(|i| row.get(i)).map(|s| s.trim())
                    };
                    if let Some(id) = get(req_col) {
                        if !id.is_empty() && id.starts_with("REQ") {
                            let req = Requirement {
                                id: id.to_string(),
                                pattern: get(pattern_col)
                                    .map(|s| s.to_string())
                                    .unwrap_or_default(),
                                korean: get(korean_col).map(|s| s.to_string()).unwrap_or_default(),
                                english: get(english_col).map(|s| s.to_string()),
                            };
                            current_reqs.push(req);
                        }
                    } else {
                        warn!(
                            "EARS 표 행 파싱 실패 (graceful skip): cells={:?}",
                            current_row
                        );
                    }
                    current_row.clear();
                }
            }
            Event::Start(Tag::TableCell) => {
                current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                let cell_text = current_cell.trim().to_string();
                if in_table_head {
                    // TableHead 내 셀은 header_cells 에 직접 추가
                    header_cells.push(cell_text);
                } else {
                    current_row.push(cell_text);
                }
                current_cell.clear();
            }

            // ── 텍스트 수집 ──
            Event::Text(t) | Event::Code(t) => {
                if in_heading {
                    heading_text.push_str(&t);
                } else if in_table {
                    current_cell.push_str(&t);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_table {
                    current_cell.push(' ');
                }
            }
            _ => {}
        }
    }

    // 마지막 그룹 완료
    if let Some((id, title)) = current_rg.take() {
        groups.push(RequirementGroup {
            id,
            title,
            requirements: current_reqs,
        });
    }

    groups
}

/// `"RG-P-1 — 설명 텍스트"` 형태 heading 에서 (id, title) 을 추출한다.
fn extract_rg_id_title(heading: &str) -> (String, String) {
    // `RG-` 로 시작하는 토큰 찾기
    let upper = heading.to_uppercase();
    let rg_start = match upper.find("RG-") {
        Some(i) => i,
        None => return (heading.to_string(), heading.to_string()),
    };

    let rest = &heading[rg_start..];
    // SPEC RG id 는 `RG-P-1`, `RG-SU-1` 형태이므로 공백 전까지
    let id_end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());

    let id = rest[..id_end].to_string();
    let title = heading.trim().to_string();
    (id, title)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_EARS: &str = r#"
### RG-P-1 — Pane 자료구조

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|---------|-----------|
| REQ-P-001 | Ubiquitous | 시스템은 이진 트리 구조를 제공해야 한다. | The system shall provide binary tree. |
| REQ-P-002 | Event-Driven | 사용자가 split 을 요청하면... | When user requests split... |

### RG-P-2 — Pane 크기

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|---------|-----------|
| REQ-P-010 | Ubiquitous | 시스템은 최소 크기를 강제해야 한다. | The system shall enforce minimum size. |
"#;

    #[test]
    fn parse_two_rg_groups() {
        let groups = parse_ears_tables(SAMPLE_EARS);
        assert_eq!(groups.len(), 2, "2 개 RG 그룹 파싱");
        assert_eq!(groups[0].requirements.len(), 2);
        assert_eq!(groups[1].requirements.len(), 1);
    }

    #[test]
    fn req_id_and_pattern_extracted() {
        let groups = parse_ears_tables(SAMPLE_EARS);
        let req = &groups[0].requirements[0];
        assert_eq!(req.id, "REQ-P-001");
        assert_eq!(req.pattern, "Ubiquitous");
    }

    #[test]
    fn graceful_on_malformed_table() {
        let text = "### RG-X-1 — Test\n\n| broken | |\n| no req id | val |\n";
        let groups = parse_ears_tables(text);
        // RG 그룹은 생성되지만 requirements 는 비어 있음 (panic 없음)
        assert!(groups.iter().all(|g| g.id.contains("RG")));
    }

    #[test]
    fn extract_rg_id_title_basic() {
        let (id, _title) = extract_rg_id_title("RG-SU-1 — SPEC document watch + parse");
        assert_eq!(id, "RG-SU-1");
    }
}
