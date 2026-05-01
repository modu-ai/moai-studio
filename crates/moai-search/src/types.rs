//! Core data types for the search engine.

use std::path::PathBuf;
use thiserror::Error;

// ---------------------------------------------------------------------------
// SearchHit
// ---------------------------------------------------------------------------

/// A single matched line within a file.
///
/// All fields are cheap to clone; preview is pre-truncated to 200 chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    /// Identifier of the workspace that owns the file.
    pub workspace_id: String,
    /// File path relative to the workspace root.
    pub rel_path: PathBuf,
    /// 1-based line number of the match.
    pub line: u32,
    /// 0-based column of the first matched character.
    pub col: u32,
    /// Preview of the matched line (at most 200 chars + ellipsis).
    pub preview: String,
    /// Byte offset of the match start within `preview`.
    pub match_start: u32,
    /// Byte offset of the match end (exclusive) within `preview`.
    pub match_end: u32,
}

// ---------------------------------------------------------------------------
// SearchOptions
// ---------------------------------------------------------------------------

/// Configuration for a single search session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOptions {
    /// The query string (literal or regex-meta containing).
    pub query: String,
    /// Whether matching is case-sensitive. Defaults to `false`.
    pub case_sensitive: bool,
    /// Maximum hits returned per file. Defaults to 50 (USER-DECISION-C (a)).
    pub max_per_file: u32,
    /// Maximum hits returned per workspace. Defaults to 200.
    pub max_per_workspace: u32,
    /// Maximum hits returned in total. Defaults to 1000.
    pub max_total: u32,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            case_sensitive: false,
            max_per_file: 50,
            max_per_workspace: 200,
            max_total: 1000,
        }
    }
}

// ---------------------------------------------------------------------------
// SearchError
// ---------------------------------------------------------------------------

/// Errors returned by the search engine.
#[derive(Debug, Error)]
pub enum SearchError {
    /// The root path could not be walked (I/O error).
    #[error("walk error: {0}")]
    Walk(#[from] ignore::Error),

    /// A regex pattern failed to compile.
    #[error("regex compile error: {0}")]
    RegexCompile(#[from] regex::Error),

    /// Generic I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Tests — T2 / T3
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// T2: SearchOptions::default() cap values and case_sensitive default.
    #[test]
    fn test_search_options_defaults() {
        let opts = SearchOptions::default();
        assert_eq!(
            opts.max_per_file, 50,
            "per-file cap must be 50 (USER-DECISION-C)"
        );
        assert_eq!(opts.max_per_workspace, 200, "per-workspace cap must be 200");
        assert_eq!(opts.max_total, 1000, "total cap must be 1000");
        assert!(!opts.case_sensitive, "default must be case-insensitive");
        assert!(opts.query.is_empty(), "default query must be empty");
    }

    /// T3: SearchHit fields are accessible and Clone/Debug work correctly.
    #[test]
    fn test_search_hit_fields_and_clone() {
        let hit = SearchHit {
            workspace_id: "ws-1".to_string(),
            rel_path: PathBuf::from("src/main.rs"),
            line: 42,
            col: 4,
            preview: "// TODO: implement".to_string(),
            match_start: 3,
            match_end: 7,
        };

        let cloned = hit.clone();
        assert_eq!(cloned.workspace_id, "ws-1");
        assert_eq!(cloned.rel_path, PathBuf::from("src/main.rs"));
        assert_eq!(cloned.line, 42);
        assert_eq!(cloned.col, 4);
        assert_eq!(cloned.match_start, 3);
        assert_eq!(cloned.match_end, 7);

        // Debug must not panic.
        let _debug = format!("{:?}", hit);
    }
}
