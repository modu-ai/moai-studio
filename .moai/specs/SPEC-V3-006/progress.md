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

---

## MS-4 (2026-04-30 sess 8) — KaTeX/Mermaid placeholder enrichment

Branch: feature/SPEC-V3-006-ms4-katex-mermaid

### Implementation
- `markdown/math_unicode.rs` (신규) — best-effort LaTeX→Unicode preview converter (superscripts, subscripts, Greek, operators, set/relation symbols). 15 unit tests.
- `markdown/mermaid_meta.rs` (신규) — Mermaid diagram type detector (flowchart/sequence/class/state/er/journey/gantt/pie/mindmap/timeline/gitGraph). 13 unit tests.
- `markdown/mod.rs` — render_block: Math 블록은 `MATH · {unicode-preview}` 헤더 + 원문, Mermaid 블록은 `MERMAID ({type}) · C-7 pending` 헤더 + 원문 표시.

### Acceptance Criteria
| AC | 내용 | 상태 |
|----|------|------|
| AC-MV-15 | Math 블록 placeholder가 가능한 곳까지 Unicode preview 표시 (e.g., `E=mc^2` → `E=mc²`) | ✅ |
| AC-MV-16 | Mermaid 블록 placeholder가 diagram type 라벨 표시 | ✅ |
| AC-MV-17 | 변환 실패 시 원본 LaTeX 그대로 통과 (panic 없음) | ✅ |

### Test count
- 신규: 28 (math_unicode 15 + mermaid_meta 13)
- 전체 markdown 모듈 67 tests pass, clippy 0, fmt clean

### Deferred to V3-007 wry integration
- 실제 KaTeX SVG 렌더 (REQ-MV-010) — wry WebView 채택 후
- 실제 Mermaid 다이어그램 (REQ-MV-011) — C-7 Mermaid Renderer Surface

---

## MS-5 (2026-04-30 sess 8) — JavaScript / JSON tree-sitter 지원 추가

Branch: feature/SPEC-V3-006-ms5-langs

### Implementation
- `Cargo.toml` (moai-studio-ui): tree-sitter-javascript 0.23 + tree-sitter-json 0.24 추가
- `viewer/code/languages.rs`: `SupportedLang::JavaScript` / `SupportedLang::Json` enum variant 추가, detect_lang_from_extension 매핑 (js/jsx/mjs/cjs/json/jsonc/pyi 추가, case-insensitive)
- `viewer/code/highlight.rs`: `map_javascript_kind` (40+ keywords/operators) + `map_json_kind` (string/number/literal/structural punctuation) 추가

### Acceptance Criteria
| AC | 내용 | 상태 |
|----|------|------|
| AC-MV-18 | JS 확장자 감지 (js/jsx/mjs/cjs) | ✅ |
| AC-MV-19 | JSON 확장자 감지 (json/jsonc) | ✅ |
| AC-MV-20 | JS keyword/operator/constant/number highlight | ✅ |
| AC-MV-21 | JSON string/number/literal/punctuation highlight | ✅ |
| AC-MV-22 | 확장자 case-insensitive 처리 | ✅ |
| AC-MV-23 | Python `.pyi` alias 추가 | ✅ |

### Test count
- 신규: 11 (languages 6 + highlight 5)
- 전체 viewer::code 모듈 27 tests pass, clippy 0, fmt clean

### Deferred
- 실제 LSP diagnostics squiggly underlines (REQ-MV-040~046) — V3-006 MS-6 또는 후속 SPEC
- C/C++/Java/Kotlin grammar 추가 — v0.2.0+ 후보
