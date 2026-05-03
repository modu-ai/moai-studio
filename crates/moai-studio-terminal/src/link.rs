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

use std::collections::HashSet;
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
    /// SPEC-ID pattern: SPEC-<AREA>-<NNN> (B-4 feature).
    SpecId(String),
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

/// Action dispatched when the user clicks a SPEC-ID span (B-4 feature).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenSpec {
    pub spec_id: String,
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

/// SPEC-ID pattern: SPEC-<AREA>-<NNN> (B-4 feature).
///
/// Matches patterns like SPEC-V3-001, SPEC-AUTH-012, SPEC-M1-001.
/// AREA is uppercase alphanumeric + hyphens, NNN is 1+ digits.
fn spec_id_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"SPEC-[A-Z0-9][A-Z0-9-]*-\d+").expect("spec_id_regex must be valid")
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

    // Pass 3: detect SPEC-IDs, skipping ranges already covered.
    for m in spec_id_regex().find_iter(text) {
        let start = m.start();
        let end = m.end();
        if overlaps_any(&spans, start, end) {
            continue;
        }
        spans.push(LinkSpan {
            kind: LinkKind::SpecId(m.as_str().to_owned()),
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
// Click resolution (B-1 feature)
// ============================================================

/// Action to dispatch when a link is clicked (B-1 feature).
///
/// SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-1: extended with `CopyUrl` variant for
/// the right-click / modifier-click "copy URL" path. The first three variants
/// are preserved at their original discriminants for backward compatibility
/// with existing match arms in `crates/moai-studio-ui/src/terminal/mod.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickAction {
    OpenCodeViewer(OpenCodeViewer),
    OpenUrl(OpenUrl),
    OpenSpec(OpenSpec),
    /// Copy the URL to the clipboard rather than opening it.
    /// SPEC-V0-2-0-OSC8-LIFECYCLE-001 REQ-OL-005.
    CopyUrl(OpenUrl),
}

/// Resolves a byte offset within `text` to a clickable link action (B-1).
///
/// Runs `detect_links()` on the text and returns the action for the first
/// span containing `byte_offset`. Returns `None` if no link spans cover
/// the offset.
pub fn resolve_click(text: &str, byte_offset: usize) -> Option<ClickAction> {
    let spans = detect_links(text);
    resolve_click_from_spans(&spans, byte_offset)
}

/// Resolves a click using pre-computed spans (avoids re-detecting).
pub fn resolve_click_from_spans(spans: &[LinkSpan], byte_offset: usize) -> Option<ClickAction> {
    for span in spans {
        if byte_offset >= span.start && byte_offset < span.end {
            return Some(match &span.kind {
                LinkKind::FilePath { path, line, col } => {
                    ClickAction::OpenCodeViewer(OpenCodeViewer {
                        path: path.clone(),
                        line: *line,
                        col: *col,
                    })
                }
                LinkKind::Url(url) => ClickAction::OpenUrl(OpenUrl { url: url.clone() }),
                LinkKind::Osc8(url) => ClickAction::OpenUrl(OpenUrl { url: url.clone() }),
                LinkKind::SpecId(id) => ClickAction::OpenSpec(OpenSpec {
                    spec_id: id.clone(),
                }),
            });
        }
    }
    None
}

/// Converts a cell column index to an approximate byte offset in a text line.
///
/// For ASCII text this is a direct mapping. For UTF-8, the offset is
/// the byte position of the `col`-th character boundary.
pub fn col_to_byte_offset(text: &str, col: usize) -> usize {
    text.char_indices()
        .nth(col)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

// ============================================================
// SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-1 — VisitedLinkRegistry + copy resolver
// ============================================================

/// In-memory set of URLs that the user has clicked-through.
///
/// @MX:NOTE: [AUTO] visited-link-registry
/// @MX:SPEC: SPEC-V0-2-0-OSC8-LIFECYCLE-001 REQ-OL-001
/// `mark_visited` is idempotent — a URL can be tracked at most once. The
/// renderer (separate PR) consults `is_visited` to apply a "visited" colour
/// override on URL / OSC 8 spans.
#[derive(Debug, Clone, Default)]
pub struct VisitedLinkRegistry {
    urls: HashSet<String>,
}

impl VisitedLinkRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark `url` as visited. Idempotent.
    /// REQ-OL-002.
    pub fn mark_visited(&mut self, url: impl Into<String>) {
        self.urls.insert(url.into());
    }

    /// True when `url` has been marked visited.
    /// REQ-OL-003.
    pub fn is_visited(&self, url: &str) -> bool {
        self.urls.contains(url)
    }

    /// Drop every entry — restore the registry to its default state.
    /// REQ-OL-004.
    pub fn clear(&mut self) {
        self.urls.clear();
    }

    /// Number of unique URLs tracked.
    pub fn count(&self) -> usize {
        self.urls.len()
    }

    /// True when no URL has been tracked.
    pub fn is_empty(&self) -> bool {
        self.urls.is_empty()
    }
}

/// Resolve a click intended for the **copy URL** path.
///
/// Returns `Some(ClickAction::CopyUrl(...))` when the click lands on a URL or
/// OSC 8 hyperlink span. Returns `None` for FilePath / SpecId spans (those
/// have no clipboard semantics in this SPEC scope) and for clicks outside any
/// span.
///
/// REQ-OL-006.
pub fn resolve_click_for_copy(text: &str, byte_offset: usize) -> Option<ClickAction> {
    let spans = detect_links(text);
    resolve_click_for_copy_from_spans(&spans, byte_offset)
}

/// Pre-computed-spans variant of `resolve_click_for_copy`.
/// REQ-OL-007.
pub fn resolve_click_for_copy_from_spans(
    spans: &[LinkSpan],
    byte_offset: usize,
) -> Option<ClickAction> {
    for span in spans {
        if byte_offset >= span.start && byte_offset < span.end {
            return match &span.kind {
                LinkKind::Url(url) | LinkKind::Osc8(url) => {
                    Some(ClickAction::CopyUrl(OpenUrl { url: url.clone() }))
                }
                LinkKind::FilePath { .. } | LinkKind::SpecId(_) => None,
            };
        }
    }
    None
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

    // ---- B-4: SPEC-ID detection ----

    #[test]
    fn test_spec_id_basic() {
        let spans = detect_links("Working on SPEC-V3-001 today");
        assert_eq!(spans.len(), 1);
        match &spans[0].kind {
            LinkKind::SpecId(id) => assert_eq!(id, "SPEC-V3-001"),
            other => panic!("expected SpecId, got {other:?}"),
        }
    }

    #[test]
    fn test_spec_id_with_area_hyphens() {
        let spans = detect_links("See SPEC-AUTH-DB-012 for details");
        assert_eq!(spans.len(), 1);
        match &spans[0].kind {
            LinkKind::SpecId(id) => assert_eq!(id, "SPEC-AUTH-DB-012"),
            other => panic!("expected SpecId, got {other:?}"),
        }
    }

    #[test]
    fn test_spec_id_not_matched_without_prefix() {
        let spans = detect_links("the V3-001 spec");
        let spec_spans: Vec<_> = spans
            .iter()
            .filter(|s| matches!(&s.kind, LinkKind::SpecId(_)))
            .collect();
        assert!(
            spec_spans.is_empty(),
            "should not match without SPEC- prefix"
        );
    }

    // ---- B-1: Click resolution ----

    #[test]
    fn test_resolve_click_on_url() {
        let text = "see https://example.com/foo for details";
        let offset = text.find("example").unwrap();
        let action = resolve_click(text, offset).expect("should resolve");
        match action {
            ClickAction::OpenUrl(OpenUrl { url }) => assert_eq!(url, "https://example.com/foo"),
            other => panic!("expected OpenUrl, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_click_on_file_path() {
        let text = s("error at src/main.rs:42:10");
        let offset = text.find("main").unwrap();
        let action = resolve_click(&text, offset).expect("should resolve");
        match action {
            ClickAction::OpenCodeViewer(OpenCodeViewer { path, line, col }) => {
                assert_eq!(path, PathBuf::from("src/main.rs"));
                assert_eq!(line, Some(42));
                assert_eq!(col, Some(10));
            }
            other => panic!("expected OpenCodeViewer, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_click_on_spec_id() {
        let text = "Working on SPEC-V3-001 today";
        let offset = text.find("V3").unwrap();
        let action = resolve_click(text, offset).expect("should resolve");
        match action {
            ClickAction::OpenSpec(OpenSpec { spec_id }) => {
                assert_eq!(spec_id, "SPEC-V3-001");
            }
            other => panic!("expected OpenSpec, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_click_no_link() {
        let text = "plain text no links";
        assert!(resolve_click(text, 5).is_none());
    }

    #[test]
    fn test_resolve_click_at_boundary_start() {
        let spans = detect_links("https://example.com");
        // Click at exact start offset
        let action = resolve_click_from_spans(&spans, 0);
        assert!(action.is_some(), "click at start should resolve");
    }

    #[test]
    fn test_resolve_click_at_boundary_end_excluded() {
        let spans = detect_links("https://example.com");
        let end = spans[0].end;
        // Click at end offset (exclusive) should NOT resolve
        let action = resolve_click_from_spans(&spans, end);
        assert!(
            action.is_none(),
            "click at end (exclusive) should not resolve"
        );
    }

    #[test]
    fn test_col_to_byte_offset_ascii() {
        assert_eq!(col_to_byte_offset("hello world", 0), 0);
        assert_eq!(col_to_byte_offset("hello world", 6), 6);
    }

    #[test]
    fn test_col_to_byte_offset_beyond_end() {
        assert_eq!(col_to_byte_offset("hi", 10), 2);
    }

    #[test]
    fn test_col_to_byte_offset_utf8() {
        let text = "한글test";
        // '한' is 3 bytes, '글' is 3 bytes. col=2 → byte offset 6
        assert_eq!(col_to_byte_offset(text, 2), 6);
    }

    // ── SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-1 — VisitedLinkRegistry + CopyUrl ──

    /// AC-OL-1 (REQ-OL-001): default registry is empty.
    #[test]
    fn visited_registry_default_is_empty() {
        let reg = VisitedLinkRegistry::default();
        assert_eq!(reg.count(), 0);
        assert!(reg.is_empty());
        assert!(!reg.is_visited("https://example.com"));
    }

    /// AC-OL-2 (REQ-OL-002): mark_visited is idempotent.
    #[test]
    fn visited_registry_mark_is_idempotent() {
        let mut reg = VisitedLinkRegistry::new();
        reg.mark_visited("https://a.test/");
        reg.mark_visited("https://a.test/");
        reg.mark_visited("https://a.test/");
        assert_eq!(reg.count(), 1);
        assert!(reg.is_visited("https://a.test/"));
    }

    /// AC-OL-3 (REQ-OL-003): is_visited true for marked, false otherwise.
    #[test]
    fn visited_registry_is_visited_distinguishes_entries() {
        let mut reg = VisitedLinkRegistry::new();
        reg.mark_visited("https://a.test/");
        reg.mark_visited("https://b.test/");
        assert!(reg.is_visited("https://a.test/"));
        assert!(reg.is_visited("https://b.test/"));
        assert!(!reg.is_visited("https://c.test/"));
        assert_eq!(reg.count(), 2);
    }

    /// AC-OL-4 (REQ-OL-004): clear() empties the registry.
    #[test]
    fn visited_registry_clear_empties_set() {
        let mut reg = VisitedLinkRegistry::new();
        reg.mark_visited("https://a.test/");
        reg.mark_visited("https://b.test/");
        reg.mark_visited("https://c.test/");
        assert_eq!(reg.count(), 3);
        reg.clear();
        assert_eq!(reg.count(), 0);
        assert!(reg.is_empty());
        assert!(!reg.is_visited("https://a.test/"));
    }

    /// AC-OL-5 (REQ-OL-005): ClickAction::CopyUrl variant matches exhaustively.
    #[test]
    fn click_action_copy_url_variant_matches_exhaustively() {
        let actions = vec![
            ClickAction::OpenCodeViewer(OpenCodeViewer {
                path: PathBuf::from("a.rs"),
                line: None,
                col: None,
            }),
            ClickAction::OpenUrl(OpenUrl {
                url: "https://x".to_string(),
            }),
            ClickAction::OpenSpec(OpenSpec {
                spec_id: "SPEC-X-1".to_string(),
            }),
            ClickAction::CopyUrl(OpenUrl {
                url: "https://y".to_string(),
            }),
        ];
        let mut seen_copy = false;
        for action in &actions {
            match action {
                ClickAction::OpenCodeViewer(_) => {}
                ClickAction::OpenUrl(_) => {}
                ClickAction::OpenSpec(_) => {}
                ClickAction::CopyUrl(OpenUrl { url }) => {
                    assert_eq!(url, "https://y");
                    seen_copy = true;
                }
            }
        }
        assert!(seen_copy, "CopyUrl arm must be reachable");
    }

    /// AC-OL-6 (REQ-OL-006): URL span resolves to CopyUrl on the copy path.
    #[test]
    fn resolve_click_for_copy_url_returns_copy_url() {
        let text = "see https://example.com/foo for more";
        let byte_offset = text.find("example").unwrap();
        let action = resolve_click_for_copy(text, byte_offset).expect("must resolve");
        match action {
            ClickAction::CopyUrl(OpenUrl { url }) => {
                assert_eq!(url, "https://example.com/foo");
            }
            other => panic!("expected CopyUrl, got {other:?}"),
        }
    }

    /// REQ-OL-006: OSC 8 span resolves to CopyUrl on the copy path.
    #[test]
    fn resolve_click_for_copy_osc8_returns_copy_url() {
        let text = "click here";
        let osc8 = vec![LinkSpan {
            kind: LinkKind::Osc8("https://osc.test/".to_string()),
            start: 0,
            end: text.len(),
        }];
        let action = resolve_click_for_copy_from_spans(&osc8, 5).expect("must resolve");
        match action {
            ClickAction::CopyUrl(OpenUrl { url }) => {
                assert_eq!(url, "https://osc.test/");
            }
            other => panic!("expected CopyUrl, got {other:?}"),
        }
    }

    /// AC-OL-7 (REQ-OL-006): FilePath span returns None on the copy path.
    #[test]
    fn resolve_click_for_copy_filepath_returns_none() {
        let text = "edit src/main.rs:10 now";
        let byte_offset = text.find("src").unwrap();
        // Sanity: regular resolve_click returns Some(OpenCodeViewer)
        assert!(matches!(
            resolve_click(text, byte_offset),
            Some(ClickAction::OpenCodeViewer(_))
        ));
        // But the copy path returns None.
        assert!(resolve_click_for_copy(text, byte_offset).is_none());
    }

    /// REQ-OL-006: SpecId span returns None on the copy path.
    #[test]
    fn resolve_click_for_copy_spec_id_returns_none() {
        let text = "see SPEC-V3-001 today";
        let byte_offset = text.find("V3").unwrap();
        assert!(matches!(
            resolve_click(text, byte_offset),
            Some(ClickAction::OpenSpec(_))
        ));
        assert!(resolve_click_for_copy(text, byte_offset).is_none());
    }

    /// REQ-OL-006: clicking outside any span returns None.
    #[test]
    fn resolve_click_for_copy_no_span_returns_none() {
        let text = "plain text with no links";
        assert!(resolve_click_for_copy(text, 5).is_none());
    }

    /// REQ-OL-007: pre-computed spans variant matches the auto-detected variant.
    #[test]
    fn resolve_click_for_copy_from_spans_matches_auto_variant() {
        let text = "open https://x.test/ now";
        let auto = resolve_click_for_copy(text, text.find("x.test").unwrap());
        let spans = detect_links(text);
        let manual = resolve_click_for_copy_from_spans(&spans, text.find("x.test").unwrap());
        assert_eq!(auto, manual);
    }
}
