//! SPEC frontmatter YAML 파싱 (--- ... --- 블록).

use serde::Deserialize;
use tracing::warn;

/// spec.md frontmatter YAML 파싱 결과.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpecFrontmatter {
    /// SPEC ID (예: "SPEC-V3-009")
    pub id: Option<String>,
    /// 버전 문자열 (예: "1.0.0")
    pub version: Option<String>,
    /// SPEC 상태 (예: "draft", "approved")
    pub status: Option<String>,
    /// milestone 목록 (예: ["MS-1", "MS-2", "MS-3"])
    pub milestones: Vec<String>,
}

/// YAML serde 중간 타입.
#[derive(Debug, Deserialize)]
struct RawFrontmatter {
    pub id: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub milestones: Option<Vec<String>>,
}

/// frontmatter YAML 문자열을 [`SpecFrontmatter`] 로 파싱한다.
///
/// `None` 이거나 파싱 실패 시 default 반환 (panic 없음 — REQ-SU-005).
pub fn parse_frontmatter(yaml: Option<&str>) -> SpecFrontmatter {
    let yaml = match yaml {
        Some(y) if !y.trim().is_empty() => y,
        _ => return SpecFrontmatter::default(),
    };

    match serde_yaml::from_str::<RawFrontmatter>(yaml) {
        Ok(raw) => SpecFrontmatter {
            id: raw.id,
            version: raw.version,
            status: raw.status,
            milestones: raw.milestones.unwrap_or_default(),
        },
        Err(e) => {
            warn!("SPEC frontmatter YAML 파싱 실패 (graceful skip): {e}");
            SpecFrontmatter::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_frontmatter() {
        let yaml = "id: SPEC-V3-009\nversion: 1.0.0\nstatus: draft\nmilestones: [MS-1, MS-2, MS-3]";
        let fm = parse_frontmatter(Some(yaml));
        assert_eq!(fm.id.as_deref(), Some("SPEC-V3-009"));
        assert_eq!(fm.version.as_deref(), Some("1.0.0"));
        assert_eq!(fm.status.as_deref(), Some("draft"));
        assert_eq!(fm.milestones, vec!["MS-1", "MS-2", "MS-3"]);
    }

    #[test]
    fn parse_partial_frontmatter() {
        let yaml = "id: SPEC-V3-001\nstatus: approved";
        let fm = parse_frontmatter(Some(yaml));
        assert_eq!(fm.id.as_deref(), Some("SPEC-V3-001"));
        assert!(fm.version.is_none());
        assert_eq!(fm.milestones, Vec::<String>::new());
    }

    #[test]
    fn parse_none_frontmatter_returns_default() {
        let fm = parse_frontmatter(None);
        assert_eq!(fm, SpecFrontmatter::default());
    }

    #[test]
    fn parse_malformed_yaml_returns_default() {
        let yaml = "id: [broken yaml: {unclosed";
        let fm = parse_frontmatter(Some(yaml));
        // graceful fallback — panic 없음
        assert!(fm.id.is_none());
    }

    #[test]
    fn parse_empty_yaml_returns_default() {
        let fm = parse_frontmatter(Some(""));
        assert_eq!(fm, SpecFrontmatter::default());
    }
}
