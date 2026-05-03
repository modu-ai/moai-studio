//! Result row rendering for the global search panel.
//!
//! SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2 T5 (REQ-GS-032).
//!
//! # Logic-level helpers
//!
//! `format_row_label` and `extract_highlight_span` are pure-logic functions
//! unit-testable without GPUI. The GPUI `render_result_row` function wraps
//! these into a 2-line GPUI element (Spike 2 pattern).
//!
//! # 2-line row layout
//!
//! ```text
//! Line 1: <workspace_name> / <rel_path>:<line>
//! Line 2: <preview text with match_start..match_end highlighted>
//! ```

use gpui::{IntoElement, ParentElement, Styled, div};
use moai_search::SearchHit;

use crate::design::tokens as tok;

// ---------------------------------------------------------------------------
// Logic helpers (no GPUI dependency)
// ---------------------------------------------------------------------------

/// Format the first line of a result row label.
///
/// Format: `<workspace_id> / <rel_path>:<line>`
///
/// `workspace_name` is passed separately because `SearchHit` stores only the
/// id; the name→id mapping lives in the MS-3 navigation layer.
pub fn format_row_label(workspace_name: &str, hit: &SearchHit) -> String {
    format!(
        "{} / {}:{}",
        workspace_name,
        hit.rel_path.display(),
        hit.line
    )
}

/// Extract the byte span `(start, end)` of the match within the preview string.
///
/// Returns `(match_start, match_end)` clamped to `preview.len()`.
pub fn extract_highlight_span(hit: &SearchHit) -> (usize, usize) {
    let len = hit.preview.len();
    let start = (hit.match_start as usize).min(len);
    let end = (hit.match_end as usize).min(len);
    (start, end)
}

// ---------------------------------------------------------------------------
// GPUI render
// ---------------------------------------------------------------------------

/// Render a single search-result row as a 2-line GPUI element.
///
/// Line 1: workspace label + rel_path:line (dimmed secondary text).
/// Line 2: preview text. The match substring at `match_start..match_end` is
///         rendered with `tok::ACCENT` color for basic highlight (N14: reuses
///         existing token, no new design token added).
///
/// The full match-highlight polish (MS-4) will refine this into a rich
/// inline-span rendering; this MS-2 version renders the preview in three
/// sequential spans: pre-match, match, post-match.
pub fn render_result_row(workspace_name: &str, hit: &SearchHit) -> impl IntoElement {
    let label = format_row_label(workspace_name, hit);
    let (start, end) = extract_highlight_span(hit);
    let preview = hit.preview.clone();

    // Split preview into three parts around the match span.
    let pre = preview[..start].to_string();
    let matched = preview[start..end].to_string();
    let post = preview[end..].to_string();

    use gpui::rgb;

    div()
        .flex()
        .flex_col()
        .px_2()
        .py_1()
        .gap_px()
        // Line 1: location label
        .child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_SECONDARY))
                .child(label),
        )
        // Line 2: preview with match highlight
        .child(
            div()
                .flex()
                .flex_row()
                .text_sm()
                .text_color(rgb(tok::FG_PRIMARY))
                .child(div().child(pre))
                .child(div().text_color(rgb(tok::ACCENT)).child(matched))
                .child(div().child(post)),
        )
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_hit(preview: &str, match_start: u32, match_end: u32) -> SearchHit {
        SearchHit {
            workspace_id: "ws-alpha".to_string(),
            rel_path: PathBuf::from("src/lib.rs"),
            line: 42,
            col: match_start,
            preview: preview.to_string(),
            match_start,
            match_end,
        }
    }

    // ── T5: result row layout ──

    /// AC-GS-8 (row layout): label includes workspace name, rel_path, and line.
    #[test]
    fn test_render_result_row_two_line_layout() {
        let hit = make_hit("use std::path::PathBuf;", 4, 7);
        let label = format_row_label("alpha-project", &hit);
        assert!(
            label.contains("alpha-project"),
            "label must include workspace name"
        );
        assert!(label.contains("src/lib.rs"), "label must include rel_path");
        assert!(label.contains("42"), "label must include line number");
        assert_eq!(label, "alpha-project / src/lib.rs:42");
    }

    /// AC-GS-8 (row layout): highlight span is extracted correctly.
    #[test]
    fn test_render_result_row_match_highlight_span() {
        // preview: "use std::path::PathBuf;"
        // match: "std" at bytes 4..7
        let hit = make_hit("use std::path::PathBuf;", 4, 7);
        let (start, end) = extract_highlight_span(&hit);
        assert_eq!(start, 4);
        assert_eq!(end, 7);
        let preview = &hit.preview;
        assert_eq!(
            &preview[start..end],
            "std",
            "highlighted region must be 'std'"
        );
    }

    /// Clamping: match_end beyond preview length is clamped.
    #[test]
    fn test_extract_highlight_span_clamps_to_preview_len() {
        let hit = make_hit("short", 3, 100); // match_end way beyond len
        let (start, end) = extract_highlight_span(&hit);
        assert!(
            end <= hit.preview.len(),
            "end must not exceed preview length"
        );
        assert!(start <= end, "start must not exceed end");
    }

    /// format_row_label handles path with subdirectories.
    #[test]
    fn test_format_row_label_nested_path() {
        let hit = SearchHit {
            workspace_id: "ws-1".to_string(),
            rel_path: PathBuf::from("crates/moai-search/src/walker.rs"),
            line: 7,
            col: 0,
            preview: "pub fn walk_workspace".to_string(),
            match_start: 0,
            match_end: 3,
        };
        let label = format_row_label("moai-studio", &hit);
        assert_eq!(label, "moai-studio / crates/moai-search/src/walker.rs:7");
    }
}
