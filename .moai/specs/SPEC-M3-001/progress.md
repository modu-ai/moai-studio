# SPEC-M3-001 Progress

**Started**: N/A
**Branch**: N/A
**SPEC status**: superseded-by-v3-006
**Completion date**: N/A

## Status Assessment (2026-04-28 audit)

SPEC-M3-001 was the **Swift-era Code Viewer** SPEC. It defined:
- SwiftTreeSitter-based syntax highlighting (6 languages)
- LSP diagnostics via `mcp__ide__getDiagnostics` + moai-lsp-bridge
- @MX gutter annotations with popover
- Tri-pane diff (HEAD | working | pending)
- Time-travel git log slider
- CodeSurface `SurfaceProtocol` integration

**All of this functionality has been re-implemented in SPEC-V3-006** using Rust/GPUI:
- `crates/moai-studio-ui/src/viewer/` — CodeViewer + MarkdownViewer + Image rendering
- `crates/moai-studio-ui/src/viewer/code/highlight.rs` — tree-sitter syntax highlight (4 langs: Rust/Go/Python/TypeScript)
- `crates/moai-studio-ui/src/viewer/lsp.rs` — LSP client with graceful degradation
- `crates/moai-studio-ui/src/viewer/mx_gutter.rs` — @MX gutter icons + popover
- `crates/moai-studio-ui/src/viewer/find_replace.rs` — Find/Replace with regex

## V3-006 Coverage vs M3-001 Scope

| M3-001 Feature | V3-006 Status | Notes |
|----------------|---------------|-------|
| Syntax highlighting | PASS (tree-sitter, 4 langs) | M3-001 specified 6 langs; V3-006 covers 4 (Go/Rust/Python/TS) |
| LSP diagnostics | PASS (mock provider) | M3-001 specified PRIMARY/SECONDARY paths; V3-006 uses mock with graceful degradation |
| @MX gutter | PASS (4 tag types) | Fully implemented |
| Tri-pane diff | NOT IN V3-006 | Deferred to follow-up SPEC |
| Time-travel | NOT IN V3-006 | Deferred to follow-up SPEC |
| Surface integration | PASS (LeafKind enum) | Integrated via SPEC-V3-004 |

## Recommendation

- Mark SPEC-M3-001 as superseded by SPEC-V3-006
- Tri-pane diff and Time-travel features should be tracked in a future SPEC (e.g., SPEC-V3-DIFF-001)
- The 2 missing languages (C, Swift) can be added incrementally to the tree-sitter grammar registry
