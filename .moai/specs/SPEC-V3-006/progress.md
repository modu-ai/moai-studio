# SPEC-V3-006 Progress

**Started**: 2026-04-25
**Branch**: feature/SPEC-V3-005-file-explorer (MS-1/MS-2 wiring), feature/SPEC-V3-006-ms3-find-lsp-mx (MS-3a), feature/SPEC-V3-005-file-explorer (remaining PRs)
**SPEC status**: implemented
**Completion date**: 2026-04-27

## Implementation Timeline

- 2026-04-25 `8e1d3e9` PR #9: SPEC-V3-006 spec/plan/research documents created (planning phase)
- 2026-04-25 `ed45ce9` PR #12: feat(viewer): SPEC-V3-006 MS-1 — Markdown/Code Viewer foundation (510 tests, 46 new)
- 2026-04-25 `da8e92d` PR #15: feat(viewer): SPEC-V3-006 MS-2 — tree-sitter syntax + 4 langs + markdown fenced (541 tests, 19 new)
- 2026-04-25 `5764ee5` PR #10: feat(explorer): SPEC-V3-005 MS-1 — File Explorer foundation (dependency for V3-006 wiring; see V3-005 progress)
- 2026-04-26 `0293f20` PR #16: feat(wiring): V3-005/006 end-to-end — explorer click to viewer mount
- 2026-04-26 `5649385` PR #28: feat(viewer): SPEC-V3-006 MS-3a Find/Replace + LSP Hover + MX Popover (72 new tests)
- 2026-04-26 `f764e2b` PR #36: feat(viewer): SPEC-V3-006 MS-3b subset Regex + LSP graceful degradation
- 2026-04-27 `f80f5d3` feat(viewer): C-2 Markdown surface EARS clause detection + lang-hint plumbing (note: duplicate commit message exists on branch-only `856be25`)
- 2026-04-27 `82a11b2` PR #60: feat(viewer,git): SPEC-V3-006/V3-008/DIST-001 final integration

## Milestone Status

- [x] MS-1: MarkdownViewer + CodeViewer foundation + LeafKind enum + VirtualScroll + OpenFileEvent routing — PR #12
- [x] MS-2: tree-sitter syntax highlight (4 langs: Rust/Go/Python/TypeScript) + fenced code block in markdown — PR #15
- [x] MS-3a: Find/Replace palette + LSP Hover (mock provider) + @MX Popover (in-memory scanner) — PR #28
- [x] MS-3b: Regex mode for Find/Replace + LSP graceful degradation — PR #36
- [x] EARS clause detection + lang-hint plumbing — post-MS-3 enhancement
- [ ] MS-3 (partial): KaTeX/Mermaid WebView rendering deferred (text fallback active), 100MB perf benchmark deferred

## Key Files Changed

### New Files

- `crates/moai-studio-ui/src/viewer/mod.rs`: LeafKind enum (Empty/Terminal/Markdown/Code) + impl Render + route_by_extension + OpenFileEvent handler
- `crates/moai-studio-ui/src/viewer/markdown/mod.rs`: MarkdownViewer struct + Render impl
- `crates/moai-studio-ui/src/viewer/markdown/parser.rs`: pulldown-cmark Event to GPUI element conversion
- `crates/moai-studio-ui/src/viewer/code/mod.rs`: CodeViewer struct + Render impl
- `crates/moai-studio-ui/src/viewer/code/highlight.rs`: tree-sitter wrapper, capture to color mapping
- `crates/moai-studio-ui/src/viewer/code/languages.rs`: Grammar registry (4 langs)
- `crates/moai-studio-ui/src/viewer/scroll.rs`: VirtualScroll data structure
- `crates/moai-studio-ui/src/viewer/find_replace.rs`: Find/Replace palette with regex, case-sensitive, whole-word
- `crates/moai-studio-ui/src/viewer/lsp.rs`: LSP client (mock diagnostic provider + graceful fallback)
- `crates/moai-studio-ui/src/viewer/mx_gutter.rs`: @MX gutter icons + popover (in-memory scanner)
- `crates/moai-studio-ui/src/viewer/image.rs`: Image rendering support

### Modified Files

- `crates/moai-studio-ui/Cargo.toml`: Added pulldown-cmark, tree-sitter, tree-sitter-rust/go/python/typescript, regex dependencies
- `crates/moai-studio-ui/src/lib.rs`: RootView handle_open_file method, viewer module registration
- `crates/moai-studio-ui/src/panes/tree.rs`: LeafKind integration

## Acceptance Criteria Status

| AC ID | Status | Notes |
|-------|--------|-------|
| AC-MV-1 | PASS | OpenFileEvent → MarkdownViewer entity mount + GFM rendering |
| AC-MV-2 | PASS | OpenFileEvent → CodeViewer entity mount + tree-sitter highlight |
| AC-MV-3 | PASS | 4 grammar bundle (USER-DECISION option a) |
| AC-MV-4 | PASS | LSP diagnostic squiggly underline (mock provider) |
| AC-MV-5 | PASS | LSP graceful degradation when binary unavailable |
| AC-MV-6 | PASS | @MX gutter icons (gold/orange/blue/gray) + popover |
| AC-MV-7 | DEFERRED | KaTeX/Mermaid — text fallback active, WebView deferred to follow-up |
| AC-MV-8 | DEFERRED | 100MB virtual scroll perf benchmark deferred |
| AC-MV-9 | PASS | terminal/panes/tabs core 0 byte change |
| AC-MV-10 | PASS | SPEC identifier link click → OpenFileEvent routing |
| AC-MV-11 | PASS | Binary file rejection with status message |
| AC-MV-12 | PASS | CodeViewer drop → LSP server cleanup, no zombie processes |

## Test Coverage

- MS-1: 510 workspace tests (46 new)
- MS-2: 541 workspace tests (19 new)
- MS-3a: 72 new tests
- MS-3b: additional regex + graceful degradation tests
- Inline tests in: scroll.rs, mx_gutter.rs, markdown/mod.rs, viewer/mod.rs, find_replace.rs, lsp.rs, code/highlight.rs, code/languages.rs, code/mod.rs, markdown/parser.rs
- clippy 0, fmt PASS at each milestone

## Known Limitations

- KaTeX math rendering: text fallback only (mono-font + "math render disabled" banner)
- Mermaid diagram rendering: text fallback only
- 100MB file virtual scroll performance benchmark not yet run
- LSP integration uses mock diagnostic provider; real LSP server spawn deferred
- Image rendering added but basic (no complex layout)

## USER-DECISION Resolutions

- katex-mermaid-rendering-strategy-v3-006: (c) Text fallback for MS-1/MS-2, WebView deferred
- tree-sitter-language-priority-v3-006: (a) 4 langs — Rust + Go + Python + TypeScript
- lsp-server-binary-discovery-v3-006: (i) Graceful degradation with banner
