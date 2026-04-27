//! Smart Link Detection -- SPEC-V3-LINK-001.
//!
//! Detects file paths (path:line:col), URLs (http/https/file), and OSC 8
//! hyperlinks in terminal output text. All regex patterns are compiled once
//! via OnceLock for O(n) per-line performance (AC-LK-6).
//!
//! @MX:ANCHOR: [AUTO] detect-links-api
//! @MX:REASON: Public API consumed by TerminalSurface (UI layer) and future
//!   click-handler wiring. fan_in >= 3 expected: TerminalSurface::render,
//!   integration tests, and OSC 8 merge path.

use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;

// ============================================================
// Data model
// ============================================================

/// Discriminates the semantic kind of a detected link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkKind {
    /// File path with optional line and column (AC-LK-1).
    FilePath {
        path: PathBuf,
        line: Option<u32>,
        col: Option<u32>,
    },
    /// HTTP/HTTPS/file URL (AC-LK-2).
    Url(String),
    /// OSC 8 hyperlink -- takes precedence over regex matches (AC-LK-3).
    Osc8(String),
}

/// A detected link span within a line of terminal output.
///
/// `start` and `end` are byte offsets into the source `&str` (end is exclusive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSpan {
    pub kind: LinkKind,
    /// Byte offset of span start in the source string.
    pub start: usize,
    /// Byte offset of span end (exclusive) in the source string.
    pub end: usize,
}

// ============================================================
// Stub actions (AC-LK-4 PARTIAL, AC-LK-5 PARTIAL)
// ============================================================

/// Action dispatched when the user clicks a FilePath span.
///
/// Full GPUI click-wiring is deferred (AC-LK-4 PARTIAL).
/// The struct is defined here so the type exists for future integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenCodeViewer {
    pub path: PathBuf,
    pub line: Option<u32>,
    pub col: Option<u32>,
}

/// Action dispatched when the user clicks a URL span.
///
/// Full GPUI cx.open_url() wiring is deferred (AC-LK-5 PARTIAL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenUrl {
    pub url: String,
}

// ============================================================
// Compiled regex patterns -- compiled once via OnceLock (AC-LK-6)
// ============================================================

/// URL pattern: https?:// or file:// followed by non-whitespace chars.
///
/// Trailing punctuation is stripped by `strip_trailing_punctuation()` after matching.
/// Uses raw string r#"..."# to avoid escaping conflicts in Rust 2024 edition.
fn url_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Matches https://, http://, or file:// followed by URL-safe characters.
        // Character class excludes whitespace and angle brackets; backtick excluded via \x60.
        Regex::new(r#"(?:https?://|file://)[^\s<>"\{\}\|\\\^\x60\x5b\x5d]+"#)
            .expect("url_regex must be valid")
    })
}

/// File path pattern with optional :line and :line:col suffixes.
///
/// Design (AC-LK-1, NEGATIVE-AC-1):
/// - Requires a dot followed by 1-5 alpha chars (file extension).
/// - The path segment allows word chars, dots, slashes, underscores, hyphens.
/// - Bare version numbers are rejected downstream by `is_version_number()`.
fn path_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?P<path>[\w./\\_-]+\.[a-zA-Z]{1,5})(?::(?P<line>\d+))?(?::(?P<col>\d+))?")
            .expect("path_regex must be valid")
    })
}

// ============================================================
// Detection logic
// ============================================================

/// Detects all link spans in `text` using regex-based matching.
///
/// Returns spans sorted by `start` offset. OSC 8 spans are NOT handled here;
/// use `detect_links_with_osc8` for that (AC-LK-3).
///
/// Complexity: O(n) per line -- regexes are compiled once (AC-LK-6).
pub fn detect_links(text: &str) -> Vec<LinkSpan> {
    let mut spans: Vec<LinkSpan> = Vec::new();

    // Pass 1: detect URLs (higher specificity -- checked first so path regex
    // can skip overlapping ranges).
    for m in url_regex().find_iter(text) {
        let raw = m.as_str();
        let stripped = strip_trailing_punctuation(raw);
        let end = m.start() + stripped.len();
        spans.push(LinkSpan {
            kind: LinkKind::Url(stripped.to_owned()),
            start: m.start(),
            end,
        });
    }

    // Pass 2: detect file paths, skipping ranges already covered by URL spans.
    for cap in path_regex().captures_iter(text) {
        let full_match = cap.get(0).expect("full match always present");
        let start = full_match.start();
        let end = full_match.end();

        // Skip if this range overlaps any already-recorded span.
        if overlaps_any(&spans, start, end) {
            continue;
        }

        let path_str = cap
            .name("path")
            .expect("named group 'path' always present")
            .as_str();

        // NEGATIVE-AC-1: reject bare version numbers (e.g., "1.2.3", "v0.1.0").
        if is_version_number(path_str) {
            continue;
        }

        let line: Option<u32> = cap.name("line").and_then(|m| m.as_str().parse().ok());
        let col: Option<u32> = cap.name("col").and_then(|m| m.as_str().parse().ok());

        spans.push(LinkSpan {
            kind: LinkKind::FilePath {
                path: PathBuf::from(path_str),
                line,
                col,
            },
            start,
            end,
        });
    }

    spans.sort_by_key(|s| s.start);
    spans
}

/// Merges pre-extracted OSC 8 spans with regex-detected spans.
///
/// OSC 8 spans take precedence: any regex span overlapping an OSC 8 span is
/// removed (AC-LK-3).
///
/// `osc8_spans` must contain only `LinkKind::Osc8` entries.
pub fn detect_links_with_osc8(text: &str, osc8_spans: &[LinkSpan]) -> Vec<LinkSpan> {
    let regex_spans = detect_links(text);

    // Filter out regex spans that overlap any OSC 8 span.
    let filtered: Vec<LinkSpan> = regex_spans
        .into_iter()
        .filter(|rs| {
            !osc8_spans
                .iter()
                .any(|os| ranges_overlap(rs.start, rs.end, os.start, os.end))
        })
        .collect();

    // Merge and sort.
    let mut result: Vec<LinkSpan> = osc8_spans.to_vec();
    result.extend(filtered);
    result.sort_by_key(|s| s.start);
    result
}

// ============================================================
// Helpers
// ============================================================

/// Returns true if [start, end) overlaps any span in `spans`.
fn overlaps_any(spans: &[LinkSpan], start: usize, end: usize) -> bool {
    spans
        .iter()
        .any(|s| ranges_overlap(start, end, s.start, s.end))
}

/// Returns true if [a, b) and [c, d) overlap.
fn ranges_overlap(a: usize, b: usize, c: usize, d: usize) -> bool {
    a < d && c < b
}

/// Strips trailing ASCII punctuation that commonly appears after a URL in prose.
fn strip_trailing_punctuation(s: &str) -> &str {
    s.trim_end_matches(['.', ',', ')', ']'])
}

/// Returns true if the string looks like a bare version number
/// (e.g., "1.2.3", "v0.1.0") that should NOT be reported as a FilePath.
///
/// Heuristic: no path separators, and all dot-separated parts are numeric
/// (with an optional leading 'v').
fn is_version_number(s: &str) -> bool {
    // Strings with a path separator are real file paths, not version numbers.
    if s.contains('/') || s.contains('\\') {
        return false;
    }
    let stripped = s.strip_prefix('v').unwrap_or(s);
    // All dot-separated parts must be purely numeric.
    !stripped.is_empty()
        && stripped
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

// ============================================================
// Tests -- RED-GREEN-REFACTOR (SPEC-V3-LINK-001)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to avoid Rust 2024 edition suffix collision in string literals
    // containing ".rs" -- use separate variable to build the string.
    fn s(v: &str) -> String {
        v.to_owned()
    }

    // ---- AC-LK-1: file path detection ----

    #[test]
    fn test_file_path_with_line_and_col() {
        let input = s("error in src/main.rs:42:10 here");
        let spans = detect_links(&input);
        assert_eq!(spans.len(), 1, "expected exactly one span");
        match &spans[0].kind {
            LinkKind::FilePath { path, line, col } => {
                assert_eq!(path, &PathBuf::from("src/main.rs"));
                assert_eq!(*line, Some(42));
                assert_eq!(*col, Some(10));
            }
            other => panic!("expected FilePath, got {other:?}"),
        }
    }

    #[test]
    fn test_file_path_with_line_only() {
        let input = s("crates/foo/lib.rs:100");
        let spans = detect_links(&input);
        assert_eq!(spans.len(), 1);
        match &spans[0].kind {
            LinkKind::FilePath { line, col, .. } => {
                assert_eq!(*line, Some(100));
                assert_eq!(*col, None);
            }
            other => panic!("expected FilePath, got {other:?}"),
        }
    }

    #[test]
    fn test_file_path_no_line_col() {
        let input = s("src/lib.rs");
        let spans = detect_links(&input);
        assert_eq!(spans.len(), 1);
        match &spans[0].kind {
            LinkKind::FilePath { line, col, .. } => {
                assert_eq!(*line, None);
                assert_eq!(*col, None);
            }
            other => panic!("expected FilePath, got {other:?}"),
        }
    }

    #[test]
    fn test_empty_input_returns_empty() {
        assert!(detect_links("").is_empty());
    }

    #[test]
    fn test_no_match_plain_text() {
        assert!(detect_links("hello world no links here").is_empty());
    }

    // ---- AC-LK-2: URL detection ----

    #[test]
    fn test_url_https_basic() {
        let spans = detect_links("see https://example.com/foo for details");
        assert_eq!(spans.len(), 1);
        match &spans[0].kind {
            LinkKind::Url(url) => assert_eq!(url, "https://example.com/foo"),
            other => panic!("expected Url, got {other:?}"),
        }
    }

    #[test]
    fn test_url_with_query_and_fragment() {
        let spans = detect_links("visit https://example.com/search?q=foo&bar=1#section");
        assert_eq!(spans.len(), 1);
        match &spans[0].kind {
            LinkKind::Url(url) => assert_eq!(url, "https://example.com/search?q=foo&bar=1#section"),
            other => panic!("expected Url, got {other:?}"),
        }
    }

    #[test]
    fn test_url_trailing_period_stripped() {
        // In prose: "Go to https://example.com." the period is not part of URL.
        let spans = detect_links("Go to https://example.com.");
        assert_eq!(spans.len(), 1);
        match &spans[0].kind {
            LinkKind::Url(url) => assert_eq!(url, "https://example.com"),
            other => panic!("expected Url, got {other:?}"),
        }
    }

    #[test]
    fn test_multiple_spans_in_one_line() {
        // Both a URL and a file path on the same line.
        let line = s("Open https://doc.rust-lang.org and src/lib.rs:10");
        let spans = detect_links(&line);
        assert!(
            spans.len() >= 2,
            "expected at least 2 spans, got {}",
            spans.len()
        );
        let has_url = spans.iter().any(|s| matches!(&s.kind, LinkKind::Url(_)));
        assert!(has_url, "expected at least one URL span");
        let has_path = spans
            .iter()
            .any(|s| matches!(&s.kind, LinkKind::FilePath { .. }));
        assert!(has_path, "expected at least one FilePath span");
    }

    // ---- NEGATIVE-AC-1: version number false positive guard ----

    #[test]
    fn test_version_number_not_matched_as_path() {
        // "1.2.3" must NOT be reported as a FilePath.
        let spans = detect_links("cargo: version 1.2.3 released");
        let file_spans: Vec<_> = spans
            .iter()
            .filter(|s| matches!(&s.kind, LinkKind::FilePath { .. }))
            .collect();
        assert!(
            file_spans.is_empty(),
            "1.2.3 must not be treated as a file path, got {file_spans:?}"
        );
    }

    #[test]
    fn test_v_prefixed_version_not_matched() {
        let spans = detect_links("release v0.1.0 is here");
        let file_spans: Vec<_> = spans
            .iter()
            .filter(|s| matches!(&s.kind, LinkKind::FilePath { .. }))
            .collect();
        assert!(
            file_spans.is_empty(),
            "v0.1.0 must not be treated as a file path, got {file_spans:?}"
        );
    }

    // ---- AC-LK-3: OSC 8 precedence ----

    #[test]
    fn test_osc8_takes_precedence_over_regex_url() {
        // OSC 8 covers the entire URL range.
        let text = "https://example.com";
        let osc8 = vec![LinkSpan {
            kind: LinkKind::Osc8("https://overridden.example".to_owned()),
            start: 0,
            end: text.len(),
        }];
        let spans = detect_links_with_osc8(text, &osc8);
        assert_eq!(spans.len(), 1);
        assert!(matches!(&spans[0].kind, LinkKind::Osc8(_)));
    }

    #[test]
    fn test_osc8_and_non_overlapping_regex_both_present() {
        // OSC 8 covers "https://example.com" at offset 0..19.
        // File path "src/lib.rs" at a later offset should still appear.
        let text = {
            let mut t = "https://example.com ".to_owned();
            t.push_str("src/lib.rs:5");
            t
        };
        let url_len = "https://example.com".len();
        let osc8 = vec![LinkSpan {
            kind: LinkKind::Osc8("https://osc8.example".to_owned()),
            start: 0,
            end: url_len,
        }];
        let spans = detect_links_with_osc8(&text, &osc8);
        let osc8_count = spans
            .iter()
            .filter(|s| matches!(&s.kind, LinkKind::Osc8(_)))
            .count();
        let path_count = spans
            .iter()
            .filter(|s| matches!(&s.kind, LinkKind::FilePath { .. }))
            .count();
        assert_eq!(osc8_count, 1, "OSC 8 span should be present");
        assert_eq!(
            path_count, 1,
            "non-overlapping FilePath span should be present"
        );
    }

    // ---- General edge cases ----

    #[test]
    fn test_spans_sorted_by_start_offset() {
        // Multiple paths at known positions -- result must be sorted.
        let text = {
            let mut t = "a/b.rs:1 and ".to_owned();
            t.push_str("c/d.rs:2");
            t
        };
        let spans = detect_links(&text);
        assert!(spans.len() >= 2, "expected >= 2 spans, got {:?}", spans);
        for window in spans.windows(2) {
            assert!(
                window[0].start <= window[1].start,
                "spans not sorted: {:?}",
                spans
            );
        }
    }

    #[test]
    fn test_stub_actions_defined() {
        // AC-LK-4/5 PARTIAL: confirm structs compile and hold expected fields.
        let path = PathBuf::from("src/main.rs");
        let _code_action = OpenCodeViewer {
            path: path.clone(),
            line: Some(42),
            col: Some(10),
        };
        let _url_action = OpenUrl {
            url: "https://example.com".to_owned(),
        };
        // Ensure equality comparison compiles (PartialEq derived).
        let a = OpenCodeViewer {
            path,
            line: Some(1),
            col: None,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
