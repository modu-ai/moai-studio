//! SPEC-V3-009 RG-SU-2 — AC (Acceptance Criteria) 표 파싱.
//!
//! 대상 패턴:
//! ```markdown
//! | AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 | RG 매핑 |
//! |------|--------------|----------|----------|---------|
//! | AC-SU-1 | ... | ... | ... | RG-SU-1 |
//! ```
//!
//! header 첫 셀이 "AC ID" 또는 "AC" 로 시작하면 AC 표로 인식.
//! 셀 수 4 또는 5 모두 허용 (RG 매핑 컬럼은 optional — plan.md §4.2).

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use tracing::warn;

/// AC 표 단일 행.
#[derive(Debug, Clone, PartialEq)]
pub struct AcRow {
    /// AC ID (예: "AC-SU-1")
    pub id: String,
    /// 검증 시나리오 텍스트
    pub scenario: String,
    /// 통과 조건 텍스트
    pub pass_condition: String,
    /// 검증 수단 텍스트 (optional)
    pub verification: Option<String>,
    /// RG 매핑 (optional)
    pub rg_mapping: Option<String>,
}

/// spec.md 본문에서 모든 AC 표를 파싱한다 (REQ-SU-004).
///
/// 파싱 실패 행은 warn 후 graceful skip (REQ-SU-005).
///
/// pulldown-cmark 표 이벤트 순서:
///   Start(Table) → Start(TableHead) → Start(TableCell)/Text/End(TableCell)...
///   End(TableHead) → Start(TableRow) → Start(TableCell)/Text/End(TableCell)...
///   End(TableRow)... → End(Table)
/// 주의: TableHead 내부에 TableRow 이벤트가 없다.
pub fn parse_ac_tables(text: &str) -> Vec<AcRow> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(text, opts);

    let mut rows: Vec<AcRow> = Vec::new();
    let mut in_table = false;
    let mut is_ac_table = false;
    let mut in_table_head = false;
    let mut header_cells: Vec<String> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();

    // 컬럼 인덱스
    let mut id_col: Option<usize> = None;
    let mut scenario_col: Option<usize> = None;
    let mut pass_col: Option<usize> = None;
    let mut verify_col: Option<usize> = None;
    let mut rg_col: Option<usize> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Table(_)) => {
                in_table = true;
                is_ac_table = false;
                header_cells.clear();
                id_col = None;
                scenario_col = None;
                pass_col = None;
                verify_col = None;
                rg_col = None;
            }
            Event::End(TagEnd::Table) => {
                in_table = false;
                is_ac_table = false;
            }
            Event::Start(Tag::TableHead) => {
                in_table_head = true;
                header_cells.clear();
            }
            Event::End(TagEnd::TableHead) => {
                in_table_head = false;
                if in_table {
                    let first = header_cells
                        .first()
                        .map(|s| s.to_uppercase())
                        .unwrap_or_default();
                    // "AC ID" 또는 "AC" 로 시작 → AC 표
                    if first.starts_with("AC") {
                        is_ac_table = true;
                        id_col = Some(0);
                        scenario_col = header_cells.iter().position(|h| {
                            let u = h.to_uppercase();
                            u.contains("시나리오") || u.contains("SCENARIO")
                        });
                        pass_col = header_cells.iter().position(|h| {
                            let u = h.to_uppercase();
                            u.contains("통과") || u.contains("PASS")
                        });
                        verify_col = header_cells.iter().position(|h| {
                            let u = h.to_uppercase();
                            u.contains("검증") || u.contains("VERIF")
                        });
                        rg_col = header_cells.iter().position(|h| {
                            let u = h.to_uppercase();
                            u.contains("RG") || u.contains("매핑") || u.contains("MAPPING")
                        });
                    }
                }
            }
            Event::Start(Tag::TableRow) if !in_table_head => {
                current_row.clear();
                current_cell.clear();
            }
            Event::End(TagEnd::TableRow) if !in_table_head && is_ac_table => {
                let row = &current_row;
                let get = |idx: Option<usize>| -> Option<&str> {
                    idx.and_then(|i| row.get(i)).map(|s| s.trim())
                };
                match get(id_col) {
                    // AC ID pattern: "AC-{group}-{nnn}".
                    Some(id) if id.starts_with("AC-") => {
                        let ac_row = AcRow {
                            id: id.to_string(),
                            scenario: get(scenario_col).map(|s| s.to_string()).unwrap_or_default(),
                            pass_condition: get(pass_col)
                                .map(|s| s.to_string())
                                .unwrap_or_default(),
                            verification: get(verify_col).map(|s| s.to_string()),
                            rg_mapping: get(rg_col).map(|s| s.to_string()),
                        };
                        rows.push(ac_row);
                    }
                    Some(_) => {
                        // ID column present but cell does not look like an AC ID — skip silently.
                    }
                    None => {
                        warn!(
                            "AC row parse failed (graceful skip): cells={:?}",
                            current_row
                        );
                    }
                }
                current_row.clear();
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
            Event::Text(t) | Event::Code(t) if in_table => {
                current_cell.push_str(&t);
            }
            Event::SoftBreak | Event::HardBreak if in_table => {
                current_cell.push(' ');
            }
            _ => {}
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_AC: &str = r#"
## 6. Acceptance Criteria

| AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 | RG 매핑 |
|------|--------------|----------|----------|---------|
| AC-SU-1 | SPEC 디렉터리가 카드로 표시 | 카드 ID 일치 | unit | RG-SU-1 |
| AC-SU-2 | EARS 표가 파싱됨 | RG 개수 = 12 | unit | RG-SU-1 |
| AC-SU-3 | AC 상태 분류 | 5 상태 모두 등장 | unit | RG-SU-2 |
"#;

    #[test]
    fn parse_three_ac_rows() {
        let rows = parse_ac_tables(SAMPLE_AC);
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn ac_id_extracted() {
        let rows = parse_ac_tables(SAMPLE_AC);
        assert_eq!(rows[0].id, "AC-SU-1");
        assert_eq!(rows[1].id, "AC-SU-2");
    }

    #[test]
    fn rg_mapping_extracted() {
        let rows = parse_ac_tables(SAMPLE_AC);
        assert_eq!(rows[0].rg_mapping.as_deref(), Some("RG-SU-1"));
    }

    #[test]
    fn graceful_on_empty_text() {
        let rows = parse_ac_tables("");
        assert!(rows.is_empty());
    }

    #[test]
    fn four_column_ac_table() {
        // RG 매핑 컬럼 없는 4-컬럼 표도 파싱 가능
        let text = "| AC ID | 시나리오 | 통과 조건 | 검증 수단 |\n|---|---|---|---|\n| AC-X-1 | test | pass | unit |\n";
        let rows = parse_ac_tables(text);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "AC-X-1");
        assert!(rows[0].rg_mapping.is_none());
    }
}
