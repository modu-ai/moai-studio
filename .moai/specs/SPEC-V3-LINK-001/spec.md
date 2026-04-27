# SPEC-V3-LINK-001 — Smart Link Detection

| Field | Value |
|-------|-------|
| **ID** | SPEC-V3-LINK-001 |
| **Title** | Smart Link Detection (path:line:col + URL + OSC 8 click handling) |
| **Status** | ready |
| **Tier** | Critical (MoAI USP) |
| **Covers** | Feature-audit B-2 (regex file path detection), B-3 (URL auto-detect), B-1 partial (OSC 8 precedence) |
| **Dependencies** | SPEC-V3-002 (terminal core) |
| **Owner** | manager-tdd |

## HISTORY

| Date | Version | Change |
|------|---------|--------|
| 2026-04-27 | 0.1.0 | Initial SPEC — B-2 + B-3 + OSC 8 precedence. First implementation (status was NONE). |

---

## 1. Problem Statement

moai-studio terminal output contains file paths (`src/main.rs:42:10`), URLs
(`https://example.com`), and OSC 8 hyperlinks. Currently all text is rendered
as plain characters — users cannot click to navigate.

This is the primary MoAI differentiation vs cmux:
a developer sees `src/main.rs:42` in terminal output, clicks, and the code
viewer opens at that exact location.

---

## 2. Scope

| In Scope | Out of Scope |
|----------|-------------|
| Regex-based file path detection (B-2) | B-4 SPEC-ID pattern detection |
| URL auto-detection (B-3) | B-5 @MX tag detection |
| OSC 8 sequence precedence (B-1 partial) | B-6 Mermaid code block detection |
| `LinkSpan` data model + `detect_links()` API | B-7 Hover preview popup |
| `OpenCodeViewer` stub action | Full GPUI click-wiring (deferred, see AC-LK-4/5) |
| Unit tests (≥ 12) covering all ACs | UI rendering of click targets |

---

## 3. Acceptance Criteria (EARS Format)

### AC-LK-1 — File Path Detection

**WHEN** terminal output contains text matching the file path pattern
`(?P<path>[\w./_-]+\.[a-zA-Z]{1,5})(?::(?P<line>\d+))?(?::(?P<col>\d+))?`,
**the system SHALL** emit a `LinkSpan { kind: FilePath { path, line, col }, start, end }`.

Constraints:
- `path` must contain at least one path separator (`/` or `\`) OR start with a recognized
  file prefix (`.`, `..`, letter-colon on Windows) OR be an explicit test double.
  Note: bare `foo.txt` (no separator) MAY match — configurable default = match.
- `line` and `col` are optional (`None` when absent).

### AC-LK-2 — URL Detection

**WHEN** terminal output contains text matching URL pattern
`https?://[^\s<>"{}|\\^`\[\]]+|file://[^\s]+`,
**the system SHALL** emit `LinkSpan { kind: Url(String), start, end }`.

Constraints:
- Trailing punctuation (`.`, `,`, `)`) MUST be stripped from matched URL.
- Query strings and fragments MUST be preserved.

### AC-LK-3 — OSC 8 Precedence

**WHEN** OSC 8 sequence metadata (`\x1b]8;params;uri\x07`) is present in the text,
**the system SHALL** emit `LinkSpan { kind: Osc8(String), start, end }` for the
visible text region, **taking precedence over** any regex matches that overlap
the same byte range.

Note: Full OSC 8 parsing requires libghostty-vt integration (out of scope for v0.1.2).
For this SPEC, `detect_links_with_osc8()` accepts pre-extracted OSC 8 spans as input
to demonstrate precedence logic.

### AC-LK-4 — File Path Click Action (PARTIAL)

**WHEN** user clicks a `FilePath` span,
**the application SHOULD** dispatch `OpenCodeViewer { path, line, col }`.

**Status: PARTIAL** — `OpenCodeViewer` struct is defined and logged. Full GPUI
click-wiring to TerminalSurface requires UI integration work deferred to a
follow-up SPEC (B-2 UI wiring).

### AC-LK-5 — URL Click Action (PARTIAL)

**WHEN** user clicks a `Url` span,
**the application SHOULD** open the URL via OS default browser.

**Status: PARTIAL** — `OpenUrl` struct is defined. GPUI `cx.open_url()` wiring
to TerminalSurface deferred to follow-up SPEC.

### AC-LK-6 — Performance

**The system SHALL** detect links in O(n) time per line where n = line length.
Regex patterns MUST be compiled once via `std::sync::OnceLock` and reused
across all calls.

### AC-LK-7 — ANSI Escape Exclusion

**The system SHALL NOT** match link patterns inside ANSI escape sequences.
The VT parser (libghostty-vt) strips ANSI escapes before text reaches
`detect_links()`, so this constraint is satisfied architecturally.

### NEGATIVE-AC-1 — Version Number False Positive Guard

Bare version numbers like `1.2.3` or `v0.1.0` **MUST NOT** match the file
path regex. The path segment must contain a letter before the dot or a
path separator.

### NEGATIVE-AC-2 — ANSI Sequence Body

Text like `\x1b[31mfoo\x1b[0m` passed raw to `detect_links()` **MUST NOT**
produce spans that overlap escape bytes. (Callers are expected to strip ANSI
before calling — this is a documentation constraint, not a runtime guard.)

---

## 4. Data Model

```rust
// crates/moai-studio-terminal/src/link.rs

pub enum LinkKind {
    FilePath {
        path: std::path::PathBuf,
        line: Option<u32>,
        col: Option<u32>,
    },
    Url(String),
    Osc8(String),
}

pub struct LinkSpan {
    pub kind: LinkKind,
    /// Byte offset of span start in the source string.
    pub start: usize,
    /// Byte offset of span end (exclusive) in the source string.
    pub end: usize,
}

/// Stub action — dispatched on FilePath span click.
/// Full GPUI wiring deferred (AC-LK-4 PARTIAL).
pub struct OpenCodeViewer {
    pub path: std::path::PathBuf,
    pub line: Option<u32>,
    pub col: Option<u32>,
}

/// Stub action — dispatched on URL span click.
/// Full GPUI wiring deferred (AC-LK-5 PARTIAL).
pub struct OpenUrl {
    pub url: String,
}
```

---

## 5. Algorithm

```
detect_links(text: &str) -> Vec<LinkSpan>:
  1. Initialize result vec
  2. Apply URL_REGEX (anchored alternation: https?://, file://)
     For each match → append Url span
  3. Apply PATH_REGEX (named groups: path, line, col)
     For each match:
       a. Check overlap with existing Url spans → skip if overlapping
       b. Validate: path contains '/' OR starts with './' OR explicit bare match
       c. Reject if path matches version-number pattern (\d+\.\d+\.\d+)
       d. Append FilePath span
  4. Sort result by start offset
  5. Return result

detect_links_with_osc8(text: &str, osc8_spans: &[LinkSpan]) -> Vec<LinkSpan>:
  1. Run detect_links(text) → regex_spans
  2. For each osc8_span, remove any regex_span that overlaps → (AC-LK-3)
  3. Merge osc8_spans + remaining regex_spans
  4. Sort by start offset
  5. Return merged
```

---

## 6. Implementation Files

| File | Action | Description |
|------|--------|-------------|
| `crates/moai-studio-terminal/src/link.rs` | CREATE | Core detection logic + data model |
| `crates/moai-studio-terminal/src/lib.rs` | MODIFY | `pub mod link;` |
| `crates/moai-studio-terminal/Cargo.toml` | MODIFY | Add `regex` dependency |

---

## 7. Non-Goals (v0.1.2)

- Full GPUI click handler wiring (AC-LK-4/5 PARTIAL)
- OSC 8 sequence parsing from raw VT stream (requires libghostty-vt integration)
- B-4 SPEC-ID pattern, B-5 @MX tag pattern, B-6 Mermaid detection
- B-7 Hover preview popup

---

## 8. v0.1.2 Status Summary

| AC | Status | Notes |
|----|--------|-------|
| AC-LK-1 | DONE | Regex path detection, line, col |
| AC-LK-2 | DONE | URL detection with trailing punctuation strip |
| AC-LK-3 | DONE | OSC 8 precedence via `detect_links_with_osc8()` |
| AC-LK-4 | PARTIAL | `OpenCodeViewer` struct defined; GPUI wiring deferred |
| AC-LK-5 | PARTIAL | `OpenUrl` struct defined; GPUI wiring deferred |
| AC-LK-6 | DONE | OnceLock regex compilation |
| AC-LK-7 | DONE | Architectural guarantee via VT parser |
| NEGATIVE-AC-1 | DONE | Version number guard in regex |
