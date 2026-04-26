//! SPEC-V3-009 RG-SU-6 — Sprint Contract Revision 추출.
//!
//! 대상 패턴: `^## \d+\.\d+ Sprint Contract Revision` (REQ-SU-050).

use regex::Regex;
use tracing::warn;

/// Sprint Contract Revision 단일 항목 (REQ-SU-051).
#[derive(Debug, Clone, PartialEq)]
pub struct SprintContractRevision {
    /// section 번호 쌍 (예: §10.1 → (10, 1))
    pub section: (u32, u32),
    /// heading 제목 전체 텍스트
    pub title: String,
    /// ISO-8601 날짜 (heading 또는 첫 단락에서 추출, 없으면 None)
    pub date: Option<String>,
    /// heading 이후 본문 markdown (다음 같은 레벨 heading 전까지)
    pub body: String,
}

/// spec.md 본문에서 Sprint Contract Revision heading 을 모두 추출한다 (REQ-SU-050).
pub fn parse_sprint_contracts(text: &str) -> Vec<SprintContractRevision> {
    // lazy_static 대신 직접 생성 (호출 빈도 낮음)
    let heading_re = Regex::new(r"(?m)^## (\d+)\.(\d+) Sprint Contract Revision(.*)?$")
        .expect("sprint contract regex must be valid");
    let date_re = Regex::new(r"\b(\d{4}-\d{2}-\d{2})\b").expect("date regex must be valid");

    let mut revisions: Vec<SprintContractRevision> = Vec::new();
    let matches: Vec<_> = heading_re.find_iter(text).collect();

    for (i, m) in matches.iter().enumerate() {
        let full_heading = m.as_str();

        // section 번호 캡처
        let caps = match heading_re.captures(full_heading) {
            Some(c) => c,
            None => {
                warn!("Sprint Contract heading 파싱 실패 (graceful skip): {full_heading}");
                continue;
            }
        };

        let major: u32 = caps[1].parse().unwrap_or(0);
        let minor: u32 = caps[2].parse().unwrap_or(0);
        let extra_title = caps.get(3).map(|s| s.as_str().trim()).unwrap_or("");

        let title = format!(
            "## {major}.{minor} Sprint Contract Revision{}",
            if extra_title.is_empty() {
                String::new()
            } else {
                format!(" {extra_title}")
            }
        );

        // body: heading 다음부터 다음 heading 시작 전까지
        let body_start = m.end();
        let body_end = if i + 1 < matches.len() {
            matches[i + 1].start()
        } else {
            text.len()
        };
        let body = text[body_start..body_end].trim().to_string();

        // date 추출: heading extra_title 먼저, 없으면 body 첫 단락
        let date = if let Some(m) = date_re.find(extra_title) {
            Some(m.as_str().to_string())
        } else {
            let first_para = body.lines().take(5).collect::<Vec<_>>().join(" ");
            date_re.find(&first_para).map(|m| m.as_str().to_string())
        };

        revisions.push(SprintContractRevision {
            section: (major, minor),
            title,
            date,
            body,
        });
    }

    revisions
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
## 10. Sprint Contracts

Some intro.

## 10.1 Sprint Contract Revision 2026-04-20

First revision body.
Contains multiple lines.

## 10.2 Sprint Contract Revision

Second revision body.
Date: 2026-04-22 mentioned here.

## 11. Other section

Not a sprint contract.
"#;

    #[test]
    fn parse_two_sprint_contracts() {
        let revisions = parse_sprint_contracts(SAMPLE);
        assert_eq!(revisions.len(), 2, "2 개 Sprint Contract Revision 추출");
    }

    #[test]
    fn section_numbers_extracted() {
        let revisions = parse_sprint_contracts(SAMPLE);
        assert_eq!(revisions[0].section, (10, 1));
        assert_eq!(revisions[1].section, (10, 2));
    }

    #[test]
    fn date_from_heading_title() {
        let revisions = parse_sprint_contracts(SAMPLE);
        assert_eq!(revisions[0].date.as_deref(), Some("2026-04-20"));
    }

    #[test]
    fn date_from_body_when_not_in_title() {
        let revisions = parse_sprint_contracts(SAMPLE);
        // body 에 "2026-04-22" 포함
        assert_eq!(revisions[1].date.as_deref(), Some("2026-04-22"));
    }

    #[test]
    fn body_captured() {
        let revisions = parse_sprint_contracts(SAMPLE);
        assert!(
            revisions[0].body.contains("First revision body"),
            "body 포함 확인"
        );
    }

    #[test]
    fn no_sprint_contracts_returns_empty() {
        let text = "# Normal SPEC\n\n## 5. Requirements\n\nNo sprint contracts here.\n";
        let revisions = parse_sprint_contracts(text);
        assert!(revisions.is_empty());
    }
}
