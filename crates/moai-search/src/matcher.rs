//! Query matcher: literal substring or regex.
//!
//! Auto-detects regex meta characters in the query. Falls back to literal on
//! regex compile failure to avoid silently dropping results.

use regex::Regex;

/// Regex meta characters that trigger automatic regex mode.
///
/// If the query string contains any of these, `Matcher::from_query` attempts
/// to compile a `Regex`. On compile failure the matcher degrades to literal.
const REGEX_META_CHARS: &[char] = &[
    '*', '+', '?', '(', ')', '[', ']', '\\', '|', '{', '}', '^', '$',
];

// @MX:ANCHOR: [AUTO] Matcher — core pattern matching abstraction; fan_in >= 3
//             (walk_workspace, tests, future MS-2 UI query binding).
// @MX:REASON: Every file line is evaluated through this matcher during search.

/// Compiled query matcher used by workers during line scanning.
#[derive(Debug)]
pub enum Matcher {
    /// Case-folded literal substring search.
    Literal {
        /// Lowercased needle when `case_sensitive` is false; original otherwise.
        needle: String,
        case_sensitive: bool,
    },
    /// Compiled regular expression.
    Regex(Regex),
}

impl Matcher {
    /// Builds a `Matcher` from a raw query string and case-sensitivity flag.
    ///
    /// Decision tree:
    /// 1. If the query contains a regex meta character → try `Regex::new`.
    /// 2. On compile failure → fall back to literal (log via `tracing::warn`).
    /// 3. Otherwise → literal.
    pub fn from_query(query: &str, case_sensitive: bool) -> Self {
        let has_meta = query.chars().any(|c| REGEX_META_CHARS.contains(&c));

        if has_meta {
            let pattern = if case_sensitive {
                query.to_owned()
            } else {
                format!("(?i){query}")
            };
            match Regex::new(&pattern) {
                Ok(re) => return Self::Regex(re),
                Err(err) => {
                    tracing::warn!(
                        query,
                        %err,
                        "regex compile failed — falling back to literal match"
                    );
                }
            }
        }

        // Literal path.
        let needle = if case_sensitive {
            query.to_owned()
        } else {
            query.to_lowercase()
        };
        Self::Literal {
            needle,
            case_sensitive,
        }
    }

    /// Returns `true` if `haystack` contains a match.
    pub fn is_match(&self, haystack: &str) -> bool {
        match self {
            Self::Literal {
                needle,
                case_sensitive,
            } => {
                if *case_sensitive {
                    haystack.contains(needle.as_str())
                } else {
                    haystack.to_lowercase().contains(needle.as_str())
                }
            }
            Self::Regex(re) => re.is_match(haystack),
        }
    }

    /// Returns the byte range `[start, end)` of the first match within `haystack`,
    /// or `None` if there is no match.
    pub fn find(&self, haystack: &str) -> Option<(usize, usize)> {
        match self {
            Self::Literal {
                needle,
                case_sensitive,
            } => {
                let (search_in, search_for) = if *case_sensitive {
                    (haystack.to_owned(), needle.clone())
                } else {
                    (haystack.to_lowercase(), needle.clone())
                };
                search_in
                    .find(search_for.as_str())
                    .map(|start| (start, start + needle.len()))
            }
            Self::Regex(re) => re.find(haystack).map(|m| (m.start(), m.end())),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — T5 / T6
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- T5: Literal matching -----------------------------------------------

    /// T5a: Literal matcher finds a plain substring.
    #[test]
    fn test_literal_substring_match() {
        let m = Matcher::from_query("use", false);
        assert!(m.is_match("use std::io;"), "should match 'use' in haystack");
        assert!(!m.is_match("fn main() {}"), "should not match when absent");
    }

    /// T5b: Default (case_sensitive=false) ignores case differences.
    #[test]
    fn test_literal_case_insensitive() {
        let m = Matcher::from_query("TODO", false);
        assert!(
            m.is_match("// todo: fix this"),
            "uppercase query matches lowercase text"
        );
        assert!(
            m.is_match("// TODO: fix this"),
            "uppercase query matches uppercase text"
        );

        let m_cs = Matcher::from_query("TODO", true);
        assert!(
            m_cs.is_match("// TODO: fix this"),
            "case-sensitive matches same case"
        );
        assert!(
            !m_cs.is_match("// todo: fix this"),
            "case-sensitive must not match different case"
        );
    }

    // ---- T6: Regex auto-detect + fallback -----------------------------------

    /// T6a: A query with regex meta characters selects `Matcher::Regex`.
    #[test]
    fn test_regex_auto_detect() {
        let m = Matcher::from_query("fn.*main", false);
        assert!(
            matches!(m, Matcher::Regex(_)),
            "query with '.' and '*' must produce Regex variant"
        );
        assert!(m.is_match("fn main() {}"), "regex should match 'fn main()'");
        assert!(
            !m.is_match("struct Foo {}"),
            "regex should not match unrelated line"
        );
    }

    /// T6b: An invalid regex falls back to literal matching instead of panicking.
    #[test]
    fn test_regex_compile_failure_fallback_to_literal() {
        // Unbalanced `[` is an invalid regex.
        let m = Matcher::from_query("[invalid", false);
        assert!(
            matches!(m, Matcher::Literal { .. }),
            "invalid regex must degrade to Literal variant"
        );
        // Should still match the raw string literally.
        assert!(
            m.is_match("[invalid syntax here"),
            "literal fallback must work as substring"
        );
    }
}
