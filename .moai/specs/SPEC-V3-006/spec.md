---
id: SPEC-V3-006
version: 1.0.0
status: draft
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002, SPEC-V3-003, SPEC-V3-004]
parallel_with: [SPEC-V3-005]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, surface, viewer, markdown, code, tree-sitter, lsp, mx-tag]
revision: v1.0.0 (initial draft, 4-surface 비전의 viewer 두 개 통합 SPEC)
---

# SPEC-V3-006: Markdown / Code Viewer Surface — EARS SPEC 마크다운 (KaTeX/Mermaid) + 코드 뷰어 (tree-sitter + LSP 진단 + @MX gutter)

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. moai-studio 4-surface 비전 (Terminal / FileTree / Markdown / Code) 의 viewer 두 surface 통합 SPEC. SPEC-V3-004 의 `render_pane_tree<L>` generic 위에 `LeafKind` enum 으로 4 surface 다형성을 도입. KaTeX/Mermaid 렌더 전략 + tree-sitter 언어 priority + LSP server binary discovery 의 3 USER-DECISION 게이트. v2 SPEC-M3-001 의 SwiftTreeSitter 결정을 Rust `tree-sitter` crate 로 등가 매핑. |

---

## 1. 개요

### 1.1 목적

moai-studio 의 4-surface 비전 (Terminal / FileExplorer / **MarkdownViewer** / **CodeViewer**) 중 두 viewer surface 를 GPUI 0.2.2 위에서 구현한다.

- **MarkdownViewer**: EARS SPEC / docs 의 마크다운 본문을 CommonMark + GFM 으로 렌더하고, KaTeX 수식 (`$$ ... $$`) 과 Mermaid 다이어그램 (`` ```mermaid ... ``` ``) 을 시각적으로 표시한다.
- **CodeViewer**: 소스 코드를 tree-sitter syntax highlight 로 그리고, LSP 진단 (squiggly underline + hover tooltip) 과 @MX 거터 (4 종 태그 색상 아이콘 + popover) 를 표시한다.

본 SPEC 은 SPEC-V3-005 (File Explorer Surface, 병행) 가 발행하는 `OpenFileEvent` 의 **수신자** 다. 사용자가 파일 트리에서 파일을 더블클릭하면 본 SPEC 의 viewer entity 가 활성 탭의 leaf 로 마운트된다.

### 1.2 SPEC-V3-004 entity 패턴 의존

SPEC-V3-004 가 확정한 다음 패턴을 본 SPEC 은 그대로 활용한다:

- `RootView.tab_container: Option<Entity<TabContainer>>`
- `TabContainer` 의 활성 탭 `pane_tree: PaneTree<L>` 의 leaf payload generic `L: Render + 'static`
- `panes::render::render_pane_tree<L>` 재귀 변환

본 SPEC 은 `L` 의 구체 타입을 `LeafKind` enum 으로 인스턴스화한다 — SPEC-V3-004 의 공개 API 는 변경하지 않는다 (RG-MV-7).

```text
pub enum LeafKind {
    Empty,
    Terminal(Entity<TerminalSurface>),
    Markdown(Entity<MarkdownViewer>),
    Code(Entity<CodeViewer>),
}

impl gpui::Render for LeafKind { ... }
```

### 1.3 v2 SPEC-M3-001 cross-reference

레거시 v2 (Swift Shell, deprecated) 의 `SPEC-M3-001` 은 `SwiftTreeSitter` + LSP + @MX 거터 + tri-pane diff + time-travel 을 단일 SPEC 으로 정의했다. 본 SPEC 은 그 중 다음 하위 집합만 v3 (Rust + GPUI) 로 이관한다:

| v2 SPEC-M3-001 | v3 SPEC-V3-006 매핑 |
|----------------|---------------------|
| RG-M3-1 SwiftTreeSitter (6 lang) | RG-MV-3 tree-sitter Rust binding (default 4 lang, USER-DECISION) |
| RG-M3-2 LSP 진단 (sourcekit-lsp 등) | RG-MV-4 LSP 진단 (`async-lsp` + `lsp-types`) |
| RG-M3-3 @MX 거터 (4 tag) | RG-MV-5 @MX gutter (4 tag, in-memory scan) |
| RG-M3-4 Tri-pane Diff | ❌ 본 SPEC 외 (E1) |
| RG-M3-5 Time-travel | ❌ 본 SPEC 외 (E2) |
| (없음) Markdown 렌더 | RG-MV-1 / RG-MV-2 신규 |

이관 / 비이관 결정의 상세 근거는 `.moai/specs/SPEC-V3-006/research.md` §1.4 참조.

### 1.4 근거 문서

- `.moai/specs/SPEC-V3-006/research.md` — 코드베이스 분석, 라이브러리 비교, AC 의 EARS 매핑 근거.
- `.moai/specs/SPEC-V3-004/spec.md` §5 (RG-R-2) — `render_pane_tree<L>` generic.
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core 무변경 carry.
- `.moai/specs/SPEC-M3-001/spec.md` (v2 legacy) — @MX 거터, LSP 진단의 정책 reference.
- `.claude/rules/moai/core/lsp-client.md` (SPEC-LSP-CORE-002) — LSP client 정책 (powernap = Go context, Rust 환경에서는 등가 매핑).
- `.claude/rules/moai/core/moai-constitution.md` "MX Tag Quality Gates" — 4 종 태그의 의미와 시각 정책.

---

## 2. 배경 및 동기

본 섹션의 상세 분석은 `.moai/specs/SPEC-V3-006/research.md` §1 ~ §5 참조. SPEC 독자가 요구사항 진입 전에 알아야 할 최소 맥락만 요약한다.

- **4-surface 비전** (research §1.1): moai-studio 는 단일 GPUI shell 위에서 Terminal / FileExplorer / MarkdownViewer / CodeViewer 4 surface 를 일관 렌더한다. 본 SPEC 은 viewer 두 개 (Markdown + Code) 를 한 묶음으로 처리하여 leaf-payload 패턴 / 파일 로딩 / 가상 스크롤의 공통 인프라를 한 번에 정의한다.
- **SPEC-V3-005 와의 분담** (research §1.2): SPEC-V3-005 가 `OpenFileEvent` 발행 책임, 본 SPEC 이 수신 및 viewer entity 마운트 책임. event 의 canonical 정의는 SPEC-V3-005 spec.md 에 위치.
- **SPEC-V3-004 escape hatch 의 leaf payload** (research §1.3): SPEC-V3-004 의 leaf payload 는 placeholder 또는 단일 `TerminalSurface` 였다. 본 SPEC 이 `LeafKind` enum 으로 4 surface 다형성을 추가한다 — SPEC-V3-004 의 generic L 자리에 들어가므로 SPEC-V3-004 의 공개 API 변경 없음.
- **v2 SPEC-M3-001 carry** (research §1.4): SwiftTreeSitter (Swift binding) → `tree-sitter` Rust crate 등가 매핑. @MX 거터 / LSP 진단 정책은 그대로 carry, tri-pane diff / time-travel 은 본 SPEC 범위 외.
- **3 USER-DECISION 게이트** (research §6.2): KaTeX/Mermaid 전략, tree-sitter 언어 priority, LSP server discovery 정책. 각 게이트의 default 가 명확하여 implementation 차단 가능성 낮음.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `MarkdownViewer` 가 GPUI `Entity` 로 생성 가능하며 `impl Render for MarkdownViewer` 가 CommonMark + GFM 본문을 그린다.
- G2. `CodeViewer` 가 GPUI `Entity` 로 생성 가능하며 `impl Render for CodeViewer` 가 tree-sitter highlight + LSP 진단 squiggly + @MX gutter 를 그린다.
- G3. `LeafKind` enum (`Empty | Terminal | Markdown | Code`) 이 SPEC-V3-004 의 `render_pane_tree<L>` generic 을 그대로 사용하여 다형 leaf 를 가능하게 한다.
- G4. `OpenFileEvent { path, surface_hint }` 수신 시 본 SPEC 의 라우터가 적절한 viewer entity 를 생성하여 활성 탭의 last_focused_pane 위치에 마운트한다.
- G5. KaTeX 수식과 Mermaid 다이어그램이 USER-DECISION 결과 (default = MS-3 시점 WebView) 채택 후 렌더된다.
- G6. tree-sitter 4 lang (USER-DECISION default = rust/go/python/typescript) syntax highlight 가 활성 코드 파일에 적용된다.
- G7. LSP 진단이 활성 코드 파일에 inline squiggly + hover tooltip 으로 표시된다.
- G8. @MX 4 종 태그 (ANCHOR/WARN/NOTE/TODO) 가 좌측 거터에 색상 아이콘으로 표시되고, 클릭 시 popover 가 노출된다.
- G9. 100 MB 코드 파일을 열어도 첫 화면 paint < 200 ms, 스크롤 시 60 fps 를 유지한다 (가상 스크롤).
- G10. SPEC-V3-002 / SPEC-V3-003 / SPEC-V3-004 의 logic 공개 API 는 변경하지 않는다 (RG-MV-7 carry).

### 3.2 비목표 (Non-Goals)

- N1. **Tri-pane Diff** (v2 SPEC-M3-001 RG-M3-4) — 별도 SPEC.
- N2. **Time-travel viewer** (v2 SPEC-M3-001 RG-M3-5) — 별도 SPEC.
- N3. **편집 (Editor) 기능** — 본 SPEC 은 read-only viewer. 텍스트 편집, 자동 완성, 코드 액션, 리팩터링은 별도 SPEC.
- N4. **@MX 태그 SQLite cache / fan_in 정적 분석** — v1.0.0 은 in-memory line-based scan, fan_in = N/A.
- N5. **Markdown 본문 내 임베디드 이미지 (jpg/png) 렌더** — 별도 SPEC. v1.0.0 은 텍스트 + alt 만.
- N6. **다중 인코딩 (UTF-16 / Shift-JIS / EUC-KR 등)** — UTF-8 + lossy fallback only.
- N7. **다중 LSP server 동시 실행 (한 파일 = 다중 server)** — 1 파일 = 1 language server.
- N8. **Markdown 본문 link 의 외부 URL 클릭 → 시스템 브라우저 open** — v1.0.0 은 링크 표시만, 클릭 동작은 별도 SPEC. SPEC 식별자 link (예: `SPEC-V3-006`) 는 동일 surface 내 navigate.
- N9. **PDF / docx / 그 외 binary viewer** — 별도 SPEC.
- N10. **Windows 빌드** — SPEC-V3-002/003/004 와 동일 carry.
- N11. **VT escape sequence / ANSI 코드 처리 in CodeViewer** — terminal surface 의 책임.
- N12. **LSP 의 외 기능 (completion, hover, goto-def, code action)** — v1.0.0 은 진단 only. 그 외는 별도 SPEC.

---

## 4. 사용자 스토리

- **US-MV1**: 사용자가 FileExplorer 에서 `.md` 파일을 더블클릭하면, 활성 탭의 last_focused_pane 이 MarkdownViewer 로 교체되어 CommonMark + GFM 본문이 가시된다 → `OpenFileEvent { path, hint: Some(Markdown) }` → `MarkdownViewer::open(path)` → `LeafKind::Markdown(...)` 마운트.
- **US-MV2**: 사용자가 EARS SPEC 마크다운을 열었을 때 본문의 `$$ E = mc^2 $$` 수식이 KaTeX 로 렌더되고, ` ```mermaid graph TD; A-->B; ``` ` 블록이 Mermaid 다이어그램으로 렌더된다 (USER-DECISION 결과 채택 후) → `pulldown-cmark` 의 math/code-block event → KaTeX/Mermaid renderer → GPUI element.
- **US-MV3**: 사용자가 FileExplorer 에서 `.rs` / `.go` / `.py` / `.ts` 파일을 더블클릭하면, 활성 탭의 leaf 가 CodeViewer 로 교체되어 tree-sitter syntax highlight 가 적용된다 → `OpenFileEvent { path, hint: Some(Code) }` → `CodeViewer::open(path)` → highlight queries 로 token 색상 표시.
- **US-MV4**: CodeViewer 가 활성화되면 해당 언어의 LSP server (rust-analyzer / gopls 등) 가 spawn 되고, 진단이 emit 되면 본문에 squiggly underline 이 가시된다. 사용자가 squiggly 위에 hover 하면 진단 메시지 tooltip 가시.
- **US-MV5**: 코드 파일에 `// @MX:ANCHOR ...`, `// @MX:WARN [REASON: ...] ...`, `// @MX:NOTE ...`, `// @MX:TODO ...` 가 있으면 좌측 거터에 ★/⚠/ℹ/☐ 아이콘이 색상 (gold/orange/blue/gray) 으로 표시. 사용자가 아이콘 클릭 시 popover 에 본문 + REASON link + fan_in (N/A) + SPEC link.
- **US-MV6**: 사용자가 100 MB 코드 파일을 열어도 첫 화면이 < 200 ms 안에 그려지고, 스크롤 시 60 fps 유지. 보이지 않는 라인은 element tree 에 포함되지 않는다 (가상 스크롤).
- **US-MV7**: 사용자가 마크다운 본문의 `SPEC-V3-006` 같은 SPEC 식별자 링크를 클릭하면, 활성 탭의 leaf 가 해당 SPEC 의 spec.md 를 여는 새 MarkdownViewer 로 교체된다.

---

## 5. 기능 요구사항 (EARS)

### RG-MV-1 — Markdown CommonMark + GFM 렌더

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-MV-001 | Ubiquitous | 시스템은 `MarkdownViewer` 가 GPUI `cx.new(\|cx\| MarkdownViewer::open(path, cx))` 로 `Entity<MarkdownViewer>` 로 생성될 수 있도록 한다. | The system **shall** allow `MarkdownViewer` to be instantiated as `Entity<MarkdownViewer>` via `cx.new`. |
| REQ-MV-002 | Ubiquitous | 시스템은 `pulldown-cmark` crate 의 `Parser::new_ext` 를 사용하여 마크다운 본문을 파싱한다. 다음 옵션이 활성화된다: `ENABLE_TABLES`, `ENABLE_STRIKETHROUGH`, `ENABLE_TASKLISTS`, `ENABLE_FOOTNOTES`, `ENABLE_HEADING_ATTRIBUTES`, `ENABLE_MATH` (수식 event 분리). | The system **shall** parse markdown via `pulldown-cmark::Parser::new_ext` with GFM + math options enabled. |
| REQ-MV-003 | Ubiquitous | 시스템은 `pulldown-cmark::Event` 스트림을 GPUI element 로 변환하는 `markdown::render_events` 함수를 제공한다. 변환은 streaming (lazy) 이어야 하며 가상 스크롤 (RG-MV-6) 과 결합 가능하다. | The system **shall** provide `markdown::render_events` converting `Event` stream to GPUI elements lazily. |
| REQ-MV-004 | Event-Driven | `MarkdownViewer::open(path)` 가 호출되면, 시스템은 `tokio::fs::read` 로 비동기 파일 read 를 시작하고 viewer state 는 `Loading` 으로 시작한다. read 완료 후 `Ready(source)` 또는 `Error(e)` 로 전이한다. | When `MarkdownViewer::open` is called, the system **shall** asynchronously read the file via `tokio::fs::read`, transitioning state Loading → Ready/Error. |
| REQ-MV-005 | Unwanted | 시스템은 200 MB 를 초과하는 파일을 마크다운으로 열려는 시도를 거부하고 `ViewerError::TooLarge` 를 표시한다. | The system **shall not** open files larger than 200 MB; it must return `ViewerError::TooLarge`. |
| REQ-MV-006 | State-Driven | `MarkdownViewer.state == Loading` 인 동안, 시스템은 spinner element 를 렌더한다. `Ready` 동안에는 본문, `Error(e)` 동안에는 error message 를 렌더한다. | While `state == Loading`, the system **shall** render a spinner; while `Ready`, the body; while `Error`, the message. |
| REQ-MV-007 | Event-Driven | 사용자가 마크다운 본문의 SPEC 식별자 패턴 (정규식 `SPEC-[A-Z0-9]+-[0-9]+`) 링크를 클릭하면, 시스템은 `OpenFileEvent { path: ".moai/specs/{ID}/spec.md", hint: Markdown }` 를 발행하여 본 SPEC 의 라우터 (REQ-MV-080) 로 전달한다. | When the user clicks a SPEC ID link, the system **shall** emit an `OpenFileEvent` for the corresponding spec.md. |

### RG-MV-2 — KaTeX 수식 + Mermaid 다이어그램

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-MV-010 | Optional | **Where** USER-DECISION 결과가 (a) WebView 채택 또는 (c → MS-3 a) 인 경우, 시스템은 `pulldown-cmark` 의 `Event::DisplayMath` / `Event::InlineMath` 본문을 KaTeX 로 렌더된 SVG / HTML 로 표시한다. | Where the USER-DECISION result selects WebView, the system **shall** render math via KaTeX. |
| REQ-MV-011 | Optional | **Where** USER-DECISION 결과가 (a) WebView 채택 인 경우, 시스템은 `lang == "mermaid"` 인 fenced code block 본문을 Mermaid 다이어그램 (SVG) 으로 렌더한다. | Where WebView is selected, the system **shall** render fenced `mermaid` blocks as SVG. |
| REQ-MV-012 | State-Driven | USER-DECISION 결과가 (c) text fallback 인 동안, 시스템은 수식과 mermaid 블록을 mono-font 코드 블록으로 표시한다. 사용자가 본문에서 그 사실을 식별 가능하도록 우상단에 "math/diagram render disabled" 배너 1 회 표시. | While the result is text fallback, the system **shall** show math/mermaid as mono-font blocks with a one-time banner. |
| REQ-MV-013 | Unwanted | 시스템은 KaTeX / Mermaid 렌더 실패 (parse error, JS exception 등) 시 panic 하지 않는다. 실패는 본문 자리에 error 박스로 인라인 표시되며 viewer 의 다른 부분 렌더는 영향받지 않는다. | The system **shall not** panic on KaTeX/Mermaid failures; errors must be inlined as error boxes. |

### RG-MV-3 — tree-sitter Syntax Highlighting

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-MV-020 | Ubiquitous | 시스템은 `CodeViewer` 가 GPUI `cx.new(\|cx\| CodeViewer::open(path, cx))` 로 `Entity<CodeViewer>` 로 생성될 수 있도록 한다. | The system **shall** allow `CodeViewer` instantiation via `cx.new`. |
| REQ-MV-021 | Ubiquitous | 시스템은 `tree-sitter` Rust binding (`tree-sitter` crate) 을 사용하여 코드 파싱을 수행한다. 자체 lexer / parser 구현은 금지한다. | The system **shall** parse code via the `tree-sitter` crate; custom lexers are prohibited. |
| REQ-MV-022 | Ubiquitous | 시스템은 USER-DECISION 결과 (default = 4 lang) 에 따라 `tree-sitter-rust`, `tree-sitter-go`, `tree-sitter-python`, `tree-sitter-typescript` 4 개 grammar 를 번들에 포함한다. (b) 채택 시 + `tree-sitter-c`, `tree-sitter-cpp`, `tree-sitter-javascript`, `tree-sitter-json` 4 개 추가. | The system **shall** bundle 4 (default) or 8 (option b) tree-sitter grammars based on USER-DECISION. |
| REQ-MV-023 | Event-Driven | `CodeViewer::open(path)` 가 호출되면, 시스템은 파일 확장자로 grammar 를 결정하고, 결정 불가 시 highlight 없이 plain text 로 렌더한다. | When `CodeViewer::open` is called, the system **shall** select grammar by extension; unknown extensions render as plain text. |
| REQ-MV-024 | Ubiquitous | 시스템은 각 grammar 의 `queries/highlights.scm` 를 번들에 포함하고, capture name (`@function`, `@string`, `@comment`, `@keyword`, `@type`, `@variable` 등) 에 색상을 매핑한다. | The system **shall** bundle each grammar's `highlights.scm` and map capture names to colors. |
| REQ-MV-025 | State-Driven | 활성 코드 파일의 grammar 가 결정된 동안, 시스템은 화면에 보이는 라인의 토큰만 highlight 한다 (RG-MV-6 가상 스크롤과 결합). | While the active grammar is set, the system **shall** highlight only visible lines' tokens. |
| REQ-MV-031 | If/Then | **If** tree-sitter incremental reparsing 이 실패하면 (예: edit 영역 invalid), **then** 시스템은 전체 파일 full reparse 로 fallback 하고 `tracing::warn!` 1 건을 기록한다 (v2 SPEC-M3-001 RG-M3-1 carry). | If incremental reparse fails, the system **shall** fall back to full reparse with a `tracing::warn!`. |

### RG-MV-4 — LSP 진단 inline overlay

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-MV-040 | Ubiquitous | 시스템은 `async-lsp` + `lsp-types` Rust crate 를 사용하여 LSP server 와 통신한다. `.claude/rules/moai/core/lsp-client.md` 의 powernap (Go) 결정과는 별개의 Rust 등가 매핑이다. | The system **shall** communicate with LSP servers via `async-lsp + lsp-types`; powernap (Go) does not apply to this Rust crate context. |
| REQ-MV-041 | Event-Driven | `CodeViewer` 가 활성화되고 grammar 가 결정되면, 시스템은 해당 언어의 LSP server 를 `tokio::process::Command` 로 spawn 하여 `initialize` → `initialized` 핸드셰이크를 수행한다. | When `CodeViewer` activates with a known grammar, the system **shall** spawn the corresponding LSP server. |
| REQ-MV-042 | Ubiquitous | 시스템은 다음 server binary 를 default 로 시도한다: `.rs` → `rust-analyzer`, `.go` → `gopls`, `.py` → `pyright` (또는 `pylsp`), `.ts/.tsx` → `typescript-language-server`. | The system **shall** attempt these defaults: rust-analyzer / gopls / pyright / typescript-language-server. |
| REQ-MV-043 | If/Then | **If** LSP server binary 가 `$PATH` 에 없으면, **then** 시스템은 (USER-DECISION 결과 i = graceful degradation) 에 따라 syntax highlight 만으로 진행하고 viewer 우상단에 "LSP unavailable: {server}" 배너 1 회 표시한다. | If the LSP server binary is missing, the system **shall** continue with syntax highlight only and show a one-time banner. |
| REQ-MV-044 | Event-Driven | LSP server 가 `textDocument/publishDiagnostics` notification 을 보내면, 시스템은 진단 cache 를 갱신하고 `cx.notify()` 로 viewer 재렌더를 트리거한다. | When `publishDiagnostics` arrives, the system **shall** update the cache and call `cx.notify()`. |
| REQ-MV-045 | State-Driven | 진단이 활성 파일에 존재하는 동안, 시스템은 진단 위치 (line, column range) 의 텍스트 아래에 squiggly underline 등가물을 그린다. severity 에 따라 색상: error=red, warning=orange, information=blue, hint=gray. | While diagnostics exist, the system **shall** draw severity-colored squiggly underlines. |
| REQ-MV-046 | Event-Driven | 사용자가 squiggly underline 위에 마우스를 hover 하면, 시스템은 진단 메시지 + severity + source 를 tooltip 으로 표시한다. | When the user hovers, the system **shall** show a diagnostic tooltip. |
| REQ-MV-047 | Event-Driven | `CodeViewer` entity 가 drop 되거나 viewer 가 close 되면, 시스템은 `shutdown` + `exit` notification 을 보내고 LSP server 자식 프로세스를 종료한다. zombie process 발생을 허용하지 않는다. | When `CodeViewer` drops, the system **shall** send `shutdown`/`exit` and kill the child process. |

### RG-MV-5 — @MX gutter (4 종 태그)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-MV-050 | Ubiquitous | 시스템은 `viewer::code::mx_scan::scan_mx_tags(source: &str) -> Vec<MxTag>` 함수를 제공한다. 라인 단위 정규식으로 `@MX:NOTE`, `@MX:WARN`, `@MX:ANCHOR`, `@MX:TODO` 4 종을 추출한다. | The system **shall** provide `scan_mx_tags` extracting 4 MX tag kinds via line-based regex. |
| REQ-MV-051 | Ubiquitous | 시스템은 `@MX:WARN` 태그의 다음 라인 또는 같은 라인에 `[REASON: ...]` 또는 `@MX:REASON: ...` sub-line 이 있으면 그것을 `MxTag.reason: Some(String)` 으로 결합한다. | The system **shall** attach `REASON` sub-lines to `@MX:WARN` tags. |
| REQ-MV-052 | Event-Driven | `CodeViewer` 가 활성화되면, 시스템은 활성 파일에 대해 `scan_mx_tags` 를 1 회 실행하고 그 결과를 viewer state 에 저장한다. (SQLite cache 미사용 — N4) | When `CodeViewer` activates, the system **shall** scan tags once and cache in viewer state. |
| REQ-MV-053 | State-Driven | 코드 본문이 가시되는 동안, 시스템은 좌측 폭 20 px 의 거터 영역에 라인별 MX 태그 아이콘을 표시한다. 매핑: ANCHOR → "★" (#d4a017 gold), WARN → "⚠" (#ff8c1a orange), NOTE → "ℹ" (#4080d0 blue), TODO → "☐" (#888888 gray). | While code body is visible, the system **shall** render gutter icons per line with the specified colors. |
| REQ-MV-054 | Event-Driven | 사용자가 거터 아이콘을 클릭하면, 시스템은 popover 를 표시한다. popover 는 (a) 태그 본문, (b) WARN 의 경우 REASON (없으면 "REASON required" 경고), (c) ANCHOR 의 경우 fan_in 카운트 ("N/A" — 정적 분석 미지원), (d) SPEC ID 가 본문에 있으면 "Jump to SPEC" 링크. | When the user clicks an icon, the system **shall** show a popover with body, REASON, fan_in, and SPEC link. |
| REQ-MV-055 | Optional | **Where** WARN 태그에 REASON 이 누락된 경우, 시스템은 거터 아이콘에 시각적 강조 (예: 1Hz blink 또는 outline) 를 적용한다. | Where WARN lacks REASON, the system **shall** apply visual emphasis to the icon. |
| REQ-MV-056 | Event-Driven | popover 의 "Jump to SPEC" 링크를 클릭하면, 시스템은 `OpenFileEvent { path: ".moai/specs/{ID}/spec.md", hint: Markdown }` 를 발행한다. | When the user clicks "Jump to SPEC", the system **shall** emit an `OpenFileEvent`. |

### RG-MV-6 — 가상 스크롤 (대용량 파일)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-MV-060 | Ubiquitous | 시스템은 `viewer::scroll::VirtualScroll { line_count, line_height_px, viewport_top_px, viewport_height_px }` 자료구조와 `visible_range() -> Range<usize>` 함수를 제공한다. | The system **shall** provide `VirtualScroll` with `visible_range()`. |
| REQ-MV-061 | State-Driven | viewer 가 활성 (focused) 인 동안, 시스템은 `visible_range` 안의 라인만 GPUI element 로 마운트한다. 그 외 라인은 element tree 에 포함되지 않는다. | While viewer is focused, the system **shall** mount only `visible_range` lines as elements. |
| REQ-MV-062 | Event-Driven | 사용자가 마우스 휠 또는 키보드 PageUp/PageDown 으로 스크롤하면, 시스템은 `viewport_top_px` 를 갱신하고 `cx.notify()` 로 재렌더한다. | When the user scrolls, the system **shall** update `viewport_top_px` and call `cx.notify()`. |
| REQ-MV-063 | Ubiquitous | 시스템은 100 MB 코드 파일에 대해 (a) 첫 화면 paint < 200 ms, (b) 스크롤 시 60 fps 유지를 보장한다. | The system **shall** ensure 100MB file: first paint < 200ms, scroll at 60fps. |

### RG-MV-7 — SPEC-V3-002 / V3-003 / V3-004 무변경 (carry)

| REQ ID | 패턴 | 요구사항 (한국어) |
|--------|------|-------------------|
| REQ-MV-070 | Ubiquitous | 시스템은 `crates/moai-studio-terminal/**` 의 어떤 파일도 수정하지 않는다 (SPEC-V3-002 RG-V3-002-1 carry). |
| REQ-MV-071 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/{tabs, panes, terminal}/**` 의 공개 API 를 변경하지 않는다 (SPEC-V3-003/004 carry). 본 SPEC 은 leaf payload generic 자리에 `LeafKind` enum 을 인스턴스화할 뿐이다. |
| REQ-MV-072 | Ubiquitous | 시스템은 `crates/moai-studio-workspace/src/persistence.rs` 의 `moai-studio/panes-v1` schema 를 변경하지 않는다 (SPEC-V3-003 MS-3 carry). |
| REQ-MV-073 | Ubiquitous | 시스템은 SPEC-V3-004 의 `panes::render::render_pane_tree<L>` generic 시그니처와 `Render for TabContainer` 를 변경하지 않는다. 본 SPEC 은 `L = LeafKind` 인스턴스화 + `LeafKind: Render` 구현 추가만 담당한다. |

### RG-MV-8 — OpenFileEvent 라우팅

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-MV-080 | Ubiquitous | 시스템은 `RootView::handle_open_file(ev: OpenFileEvent, cx)` 메서드를 제공한다. SPEC-V3-005 의 OpenFileEvent canonical 정의를 import 하여 사용한다. | The system **shall** provide `RootView::handle_open_file` consuming SPEC-V3-005's `OpenFileEvent`. |
| REQ-MV-081 | Event-Driven | `OpenFileEvent.surface_hint == Some(Markdown)` 또는 (`hint == None` 이고 확장자가 `.md` / `.markdown`) 일 때, 시스템은 `MarkdownViewer::open(path)` 으로 entity 를 생성하고 `LeafKind::Markdown(...)` 으로 활성 탭 leaf 에 마운트한다. | When hint is Markdown or extension matches, the system **shall** mount a `MarkdownViewer` leaf. |
| REQ-MV-082 | Event-Driven | `OpenFileEvent.surface_hint == Some(Code)` 또는 (`hint == None` 이고 확장자가 `.rs`/`.go`/`.py`/`.ts`/`.tsx`/`.json`/`.toml`/`.txt`) 일 때, 시스템은 `CodeViewer::open(path)` 으로 entity 를 생성하고 `LeafKind::Code(...)` 으로 마운트한다. | When hint is Code or extension matches, the system **shall** mount a `CodeViewer` leaf. |
| REQ-MV-083 | Unwanted | 시스템은 binary 파일 (시그니처 검출: PNG/JPEG/PDF magic bytes, NUL byte 다수) 에 대해 viewer 를 열지 않는다. 거부 시 status bar 에 1 회 안내 메시지. | The system **shall not** open binary files; it must show a status message instead. |
| REQ-MV-084 | Event-Driven | `OpenFileEvent` 수신 시 활성 탭의 `last_focused_pane` 이 `LeafKind::Empty` 이면 그 자리에 in-place 교체. 아니면 활성 탭의 가장 최근 focused leaf 자리를 교체한다 (split 추가 없음, v1.0.0 정책). | When event arrives, the system **shall** replace the focused leaf in-place. |

---

## 6. 비기능 요구사항

### 6.1 성능

- NFR-MV-1. 일반 (≤ 1 MB) 파일의 viewer open → 첫 paint ≤ 100 ms.
- NFR-MV-2. 100 MB 파일의 viewer open → 첫 paint ≤ 200 ms (RG-MV-6 가상 스크롤 기반).
- NFR-MV-3. 가상 스크롤 시 60 fps 유지 — 단일 frame 예산 ≤ 16 ms.
- NFR-MV-4. tree-sitter incremental reparse (1 라인 편집) ≤ 5 ms (1 MB 파일 기준).
- NFR-MV-5. LSP server cold start ≤ 3 s (rust-analyzer 기준), warm hit ≤ 200 ms.

### 6.2 안정성

- NFR-MV-6. Render::render 는 어떤 viewer state 에서도 panic 하지 않는다 (REQ-MV-013, REQ-MV-005, ViewerError 보호).
- NFR-MV-7. LSP server zombie process 를 발생시키지 않는다 (REQ-MV-047).
- NFR-MV-8. CodeViewer 비활성 (탭 비활성) 동안 tree-sitter 파서 스레드를 일시정지한다 (v2 SPEC-M3-001 carry, 메모리 누수 방지).
- NFR-MV-9. 5 분 idle 후 메모리 사용량 증가 ≤ 5 MB (SPEC-V3-004 NFR-R-5 carry).

### 6.3 접근성

- NFR-MV-10. CodeViewer 의 진단 squiggly 는 색상 외에 형태 (점선 / 물결) 로도 severity 식별 가능해야 한다 (color-blind 대응).
- NFR-MV-11. @MX 거터 아이콘은 색상 외에 글자 (★/⚠/ℹ/☐) 로도 식별 가능하다.
- NFR-MV-12. 키보드 navigation 가능: Tab 으로 viewer focus, 화살표 키로 line scroll, PageUp/Down, Home/End.

### 6.4 호환성

- NFR-MV-13. macOS 14+ / Ubuntu 22.04+ 양쪽에서 동일한 viewer 동작 보장 (SPEC-V3-002/003/004 carry).
- NFR-MV-14. UTF-8 + lossy fallback 만 지원 (N6).

---

## 7. 아키텍처

### 7.1 모듈 트리

```
crates/moai-studio-ui/src/
├── lib.rs                        # SPEC-V3-001 ~ V3-004 carry, handle_open_file 추가만
├── tabs/, panes/, terminal/      # SPEC-V3-002~004 carry, 무변경
├── viewer/                       # 본 SPEC 신규
│   ├── mod.rs                    # LeafKind enum + impl Render + 라우터 (route_by_extension)
│   ├── markdown/
│   │   ├── mod.rs                # MarkdownViewer struct + impl Render
│   │   ├── parser.rs             # pulldown-cmark wrapper (Event → IntoElement)
│   │   ├── katex.rs              # 수식 렌더 (USER-DECISION 결과 분기)
│   │   ├── mermaid.rs            # 다이어그램 렌더 (USER-DECISION 결과 분기)
│   │   └── tests.rs
│   ├── code/
│   │   ├── mod.rs                # CodeViewer struct + impl Render
│   │   ├── highlight.rs          # tree-sitter wrapper, capture → color
│   │   ├── languages.rs          # grammar registry (4 or 8 lang)
│   │   ├── gutter.rs             # @MX gutter element
│   │   ├── mx_scan.rs            # scan_mx_tags 함수
│   │   └── tests.rs
│   ├── diagnostics.rs            # 진단 cache + squiggly render helper
│   └── scroll.rs                 # VirtualScroll
└── lsp/                          # 본 SPEC 신규 (MS-3)
    ├── mod.rs                    # LspClient 추상 (async-lsp wrapper)
    ├── server_registry.rs        # rust-analyzer / gopls / pyright / tsserver config
    └── tests.rs
```

### 7.2 LeafKind dispatch 데이터 흐름

```
SPEC-V3-005 emits OpenFileEvent
   │
   ▼ RootView::handle_open_file(ev, cx)
   │
   ▼ route_by_extension(&ev.path) or ev.surface_hint
   │
   ▼ match surface:
       Markdown → MarkdownViewer::open(path, cx) → Entity<MarkdownViewer>
                  → LeafKind::Markdown(entity)
       Code     → CodeViewer::open(path, cx)     → Entity<CodeViewer>
                  → LeafKind::Code(entity)
       Terminal → (SPEC-V3-002 carry) Entity<TerminalSurface>
                  → LeafKind::Terminal(entity)
   │
   ▼ tab_container.update(cx, |tc, cx| {
        let leaf_id = tc.active_tab().last_focused_pane;
        tc.active_tab_mut().pane_tree.set_leaf_payload(leaf_id, leaf_kind);
        cx.notify();
     });
   │
   ▼ SPEC-V3-004 의 render_pane_tree<LeafKind> 가 호출되어 element tree 갱신.
```

### 7.3 CodeViewer 내부 데이터 흐름

```
CodeViewer::open(path)
   │
   ▼ 1. async file read (tokio::fs::read)
   │
   ▼ 2. ext → grammar (tree-sitter language)
   │
   ▼ 3. parser.parse(source, None) → Tree
   │
   ▼ 4. mx_scan(source) → Vec<MxTag>
   │
   ▼ 5. lsp_client.spawn_for_language(lang) → 진단 subscribe
   │
   ▼ 6. impl Render:
        ┌─────────────────────────────────────────────────────┐
        │ flex_row (gutter | code-body)                       │
        │ ├── gutter: render_mx_gutter(&tags, visible_range)  │
        │ └── code-body:                                       │
        │     for line_no in visible_range:                    │
        │       render_line(line_no, tokens, diagnostics)      │
        │         ├── tree-sitter highlight tokens             │
        │         └── if diag exists: squiggly underline       │
        └─────────────────────────────────────────────────────┘
```

### 7.4 MarkdownViewer 내부 데이터 흐름

```
MarkdownViewer::open(path)
   │
   ▼ 1. async file read
   │
   ▼ 2. pulldown-cmark Parser::new_ext(source, opts)
   │
   ▼ 3. impl Render:
        ┌─────────────────────────────────────────────────────┐
        │ for event in parser:                                 │
        │   match event:                                       │
        │     Start(Heading(level)) → text_size_for_level()    │
        │     Text(s) → text_element                           │
        │     Code(s) → inline code                            │
        │     Start(CodeBlock(Fenced(lang))):                  │
        │       if lang == "mermaid": mermaid_render(body)     │
        │       else: tree_sitter_highlight(body, lang)        │
        │     DisplayMath(s) | InlineMath(s) → katex_render(s) │
        │     Start(Link(_, dest, _)):                         │
        │       if SPEC pattern: register click → OpenFileEvent│
        │     ...                                               │
        └─────────────────────────────────────────────────────┘
```

---

## 8. Milestone

본 SPEC 은 3 milestone 으로 분할한다. milestone 간 regression gate 는 SPEC-V3-004 carry.

### MS-1: Markdown CommonMark + 기본 코드 블록 (RG-MV-1, RG-MV-7, RG-MV-8 부분, RG-MV-6 기본)

- **범위**: `viewer::markdown::*` 모듈 신규, `LeafKind` enum 신규, `RootView::handle_open_file` 메서드 신규, mock OpenFileEvent unit test, `pulldown-cmark` GFM 파싱, 기본 코드 블록은 mono-font (highlight 미적용), `VirtualScroll` 자료구조, `viewer::scroll` 모듈.
- **포함 요구사항**: REQ-MV-001 ~ MV-006, REQ-MV-060 ~ MV-061 (basic), REQ-MV-070 ~ MV-073, REQ-MV-080 ~ MV-084.
- **시연 가능 상태**: mock OpenFileEvent 또는 임시 디버그 hook 으로 `.md` 파일 viewer 가시. EARS SPEC 마크다운 본문이 GFM (table, strikethrough, tasklist) 으로 렌더. 수식 / mermaid 는 텍스트 fallback (배너).
- **USER-DECISION**: (없음 — 모두 MS-2 / MS-3 진입 시점으로 deferral)

### MS-2: tree-sitter integration + syntax highlight (RG-MV-3, RG-MV-1 의 코드 블록 highlight 통합)

- **범위**: `viewer::code::{mod, highlight, languages}` 모듈 신규, `CodeViewer` entity, 4 (또는 8) lang grammar 번들, capture → color 매핑, fenced code block 안의 코드도 highlight 적용 (markdown viewer 와 통합).
- **포함 요구사항**: REQ-MV-020 ~ MV-025, REQ-MV-031.
- **시연 가능 상태**: `.rs` / `.go` / `.py` / `.ts` 파일을 viewer 로 열 때 토큰 색상 적용. 마크다운 본문의 ` ```rust ``` ` 블록도 동일 highlight.
- **USER-DECISION (2 개, MS-2 진입 직전)**:
  - katex-mermaid-rendering-strategy-v3-006 (default = c, MS-3 시점 a 채택)
  - tree-sitter-language-priority-v3-006 (default = a, 4 lang)

### MS-3: LSP 진단 + @MX gutter + KaTeX/Mermaid (RG-MV-2, RG-MV-4, RG-MV-5)

- **범위**: `viewer::code::{gutter, mx_scan}` 신규, `viewer::diagnostics` 신규, `lsp::*` 모듈 신규, `viewer::markdown::{katex, mermaid}` 의 USER-DECISION 결과 채택 (default = WebView), `wry` crate 의존성 추가 (a 채택 시).
- **포함 요구사항**: REQ-MV-010 ~ MV-013, REQ-MV-040 ~ MV-047, REQ-MV-050 ~ MV-056, REQ-MV-062 ~ MV-063 (NFR 검증).
- **시연 가능 상태**: §1.5 의 7 가지 사용자 가시 동작 모두 PASS. 100MB 파일 가상 스크롤 60 fps. LSP 진단 inline + popover. @MX gutter 클릭 → popover.
- **USER-DECISION (1 개, MS-3 진입 직전)**:
  - lsp-server-binary-discovery-v3-006 (default = i, graceful degradation)

---

## 9. 파일 레이아웃 (canonical)

### 9.1 신규

- `crates/moai-studio-ui/src/viewer/mod.rs` — LeafKind enum, route_by_extension, impl Render for LeafKind.
- `crates/moai-studio-ui/src/viewer/markdown/mod.rs` — MarkdownViewer struct + impl Render.
- `crates/moai-studio-ui/src/viewer/markdown/parser.rs` — pulldown-cmark Event → IntoElement 변환.
- `crates/moai-studio-ui/src/viewer/markdown/katex.rs` — KaTeX 수식 렌더 (USER-DECISION 결과 분기).
- `crates/moai-studio-ui/src/viewer/markdown/mermaid.rs` — Mermaid 다이어그램 렌더.
- `crates/moai-studio-ui/src/viewer/code/mod.rs` — CodeViewer struct + impl Render.
- `crates/moai-studio-ui/src/viewer/code/highlight.rs` — tree-sitter wrapper, capture → color.
- `crates/moai-studio-ui/src/viewer/code/languages.rs` — grammar registry (4 / 8 lang).
- `crates/moai-studio-ui/src/viewer/code/gutter.rs` — @MX gutter element rendering.
- `crates/moai-studio-ui/src/viewer/code/mx_scan.rs` — scan_mx_tags 함수.
- `crates/moai-studio-ui/src/viewer/diagnostics.rs` — 진단 cache + squiggly render.
- `crates/moai-studio-ui/src/viewer/scroll.rs` — VirtualScroll.
- `crates/moai-studio-ui/src/lsp/mod.rs` — LspClient 추상 (async-lsp wrapper).
- `crates/moai-studio-ui/src/lsp/server_registry.rs` — server config.
- `crates/moai-studio-ui/tests/integration_viewer.rs` — viewer integration 테스트 (USER-DECISION SPEC-V3-004 결과에 따라 fallback).

### 9.2 수정

- `crates/moai-studio-ui/src/lib.rs` — `RootView::handle_open_file(ev, cx)` 메서드 추가, `viewer` 와 `lsp` 모듈 re-export. RootView 구조체 수정 없음 (tab_container 필드 그대로).
- `crates/moai-studio-ui/Cargo.toml` — 신규 의존성: `pulldown-cmark`, `tree-sitter`, `tree-sitter-rust`/`-go`/`-python`/`-typescript` (USER-DECISION 결과), `async-lsp`, `lsp-types`, `regex`, `tokio` (workspace 이미), `wry` (USER-DECISION (a) 채택 시).

### 9.3 변경 금지 (FROZEN — REQ-MV-070 ~ MV-073)

- `crates/moai-studio-terminal/**` 전체.
- `crates/moai-studio-ui/src/{tabs, panes, terminal}/**` 의 공개 API.
- `crates/moai-studio-ui/src/lib.rs` 의 SPEC-V3-004 가 정의한 RootView 구조 (tab_container 필드, Render impl, key dispatch). 본 SPEC 은 handle_open_file 메서드 추가만.
- `crates/moai-studio-workspace/src/persistence.rs` 의 `moai-studio/panes-v1` schema.
- `crates/moai-studio-ui/src/panes/render.rs` 의 `render_pane_tree<L>` 시그니처 (SPEC-V3-004 carry).

---

## 10. Acceptance Criteria

| AC ID | Requirement Group | Milestone | Given | When | Then | 검증 수단 |
|-------|-------------------|-----------|-------|------|------|-----------|
| AC-MV-1 | RG-MV-1 (REQ-MV-001 ~ MV-006), RG-MV-8 (REQ-MV-080 ~ MV-081) | MS-1 | 활성 탭이 단일 leaf 인 RootView, mock `OpenFileEvent { path: ".moai/specs/SPEC-V3-006/spec.md", hint: Some(Markdown) }` 발행. | RootView::handle_open_file 호출 | 활성 탭의 last_focused_pane 이 `LeafKind::Markdown(Entity<MarkdownViewer>)` 로 교체된다. MarkdownViewer state 가 Loading → Ready 로 전이하고 GFM 본문 (heading, table, strikethrough) 이 element tree 에 포함된다. panic 없음. | unit test (mock cx + leaf state 검증) + USER-DECISION SPEC-V3-004 결과 (a) 시 integration_viewer.rs 의 TestAppContext |
| AC-MV-2 | RG-MV-3 (REQ-MV-020 ~ MV-024), RG-MV-8 (REQ-MV-080, MV-082) | MS-2 | mock `OpenFileEvent { path: "src/lib.rs", hint: Some(Code) }` (실제 본 레포의 Rust 파일). | RootView::handle_open_file 호출 | 활성 탭이 `LeafKind::Code(Entity<CodeViewer>)` 로 교체. CodeViewer 가 `tree-sitter-rust` grammar 를 선택, highlights.scm capture (function/string/comment) 토큰이 색상별로 element 에 포함된다. | unit test (highlight result → token color list 검증) + integration |
| AC-MV-3 | RG-MV-3 USER-DECISION 게이트 (default 4 lang) | MS-2 | `Cargo.toml` 에 USER-DECISION 결과 (default = a) 의 4 grammar (`tree-sitter-rust`/`-go`/`-python`/`-typescript`) 가 의존성으로 선언됨. | `cargo build -p moai-studio-ui` 실행 | 빌드 통과. 4 grammar 가 번들에 포함되고 `viewer::code::languages::SUPPORTED_LANGUAGES` 가 정확히 4 entry 반환. (b) 채택 시 8 entry. | cargo build + unit test |
| AC-MV-4 | RG-MV-4 (REQ-MV-040 ~ MV-046) | MS-3 | rust-analyzer 가 `$PATH` 에 있는 환경 + 의도적 syntax error 가 있는 `.rs` 파일 (예: `fn foo() { x }` 에서 `x` undefined) viewer 로 open. | LSP server 가 `publishDiagnostics` 발행 | CodeViewer 본문의 해당 라인/컬럼에 squiggly underline 이 가시. severity 가 error 인 경우 빨간색. 사용자가 그 위치에 hover 하면 진단 메시지 tooltip. CodeViewer entity drop 시 LSP server 자식 프로세스가 종료된다 (zombie 없음). | integration test (rust-analyzer 가용 CI) + manual smoke + ps 명령으로 zombie 확인 |
| AC-MV-5 | RG-MV-4 USER-DECISION 게이트 (graceful degradation) | MS-3 | rust-analyzer 가 `$PATH` 에 **없는** 환경 + `.rs` 파일 viewer open. | CodeViewer init | LSP server spawn 실패 → syntax highlight 만 활성, viewer 우상단에 "LSP unavailable: rust-analyzer" 배너 1 회 표시, panic 없음, error 로그 1 건. | unit test (LSP spawn mock failure) + manual env without rust-analyzer |
| AC-MV-6 | RG-MV-5 (REQ-MV-050 ~ MV-056) | MS-3 | 본 레포의 `crates/moai-studio-ui/src/lib.rs` (SPEC-V3-004 가 ANCHOR 태그 추가한 파일). 본문에 `@MX:ANCHOR root-view-tab-container-binding` (RootView.tab_container 위 — SPEC-V3-004 plan §3.1) 이 있다. | viewer 가 해당 파일 open + 라인 79 (또는 ANCHOR 라인) 가 viewport 안 | 라인 79 거터에 ★ (gold #d4a017) 아이콘 가시. 클릭 시 popover 노출 — body, fan_in: "N/A", SPEC link "SPEC-V3-004". WARN 태그가 있다면 REASON 검출. | integration test + manual smoke |
| AC-MV-7 | RG-MV-2 (REQ-MV-010 ~ MV-013) USER-DECISION 게이트 | MS-3 (USER-DECISION 결과 = a 채택 시) | EARS SPEC 마크다운 본문에 `$$ E = mc^2 $$` 와 ` ```mermaid graph TD; A-->B; ``` ` 블록 포함. WebView 채택. | MarkdownViewer render | KaTeX 수식이 SVG / element 로 렌더되어 가시. Mermaid 다이어그램이 SVG 로 렌더되어 가시. (c) 채택 (deferred) 시: mono-font + "math/diagram render disabled" 배너 1 회 + AC PASS (text fallback 명시). | integration test (WebView headless) + manual smoke |
| AC-MV-8 | RG-MV-6 (REQ-MV-060 ~ MV-063) | MS-3 | 100 MB 합성 텍스트 파일 (라인 길이 80, 약 1.25M lines) viewer 로 open. | viewer 활성화 + 첫 paint 시간 측정 | 첫 paint ≤ 200 ms. 사용자가 PageDown 50 회 빠르게 누를 때 평균 frame time ≤ 16 ms (60 fps). element tree 의 line element 수는 viewport 안 라인 (≤ 100) 만 존재. | bench (criterion) + integration test |
| AC-MV-9 | RG-MV-7 (REQ-MV-070 ~ MV-073) | 전체 | 본 SPEC 모든 milestone 완료 후 | `cargo test -p moai-studio-terminal --all-targets` + `cargo test -p moai-studio-workspace --all-targets` + `cargo test -p moai-studio-ui --lib panes::` + `cargo test -p moai-studio-ui --lib tabs::` + `cargo test -p moai-studio-ui --lib terminal::` + `cargo test -p moai-studio-ui --test integration_render` (SPEC-V3-004 carry) | SPEC-V3-002/003/004 의 모든 기존 tests GREEN, 0 regression. | CI gate / cargo test |
| AC-MV-10 | RG-MV-1 (REQ-MV-007), RG-MV-5 (REQ-MV-056) | MS-3 | MarkdownViewer 가 `.moai/specs/SPEC-V3-006/spec.md` 를 표시 중. 본문에 `SPEC-V3-004` 텍스트가 link 로 detected. | 사용자가 그 link 클릭 | `OpenFileEvent { path: ".moai/specs/SPEC-V3-004/spec.md", hint: Markdown }` 발행되어 본 SPEC 의 라우터로 진입, 활성 leaf 가 새 MarkdownViewer 로 교체. | integration test + manual smoke |
| AC-MV-11 | RG-MV-8 (REQ-MV-083) | MS-1 | binary 파일 (예: PNG) 의 `OpenFileEvent` 발행. | RootView::handle_open_file 호출 | viewer 가 마운트되지 않고 status bar 또는 toast 에 "Cannot open binary file: {path}" 메시지 1 회 표시. activetab leaf 변경 없음. | unit test (mock binary signature) |
| AC-MV-12 | NFR-MV-7, NFR-MV-8, NFR-MV-9 | MS-3 | CodeViewer 가 활성화된 viewer 를 close. | viewer 가 dropped 후 5 초 대기. | (a) `ps` 또는 `lsof` 결과에 spawn 한 LSP server child process 없음. (b) tree-sitter 파서 스레드 종료 또는 sleep. (c) 메모리 RSS 증가 ≤ 5 MB. | integration + ps + memory profile |

---

## 11. 의존성 및 제약

### 11.1 외부 의존성

| Crate | 버전 (제안) | 상태 | 비고 |
|-------|-------------|------|------|
| `pulldown-cmark` | ^0.13 (또는 0.10+) | 신규 | Markdown 파싱 (RG-MV-1) |
| `tree-sitter` | ^0.25 (또는 0.22+) | 신규 | Syntax 파싱 (RG-MV-3) |
| `tree-sitter-rust` | matching | 신규 (USER-DECISION default) | Rust grammar |
| `tree-sitter-go` | matching | 신규 (default) | Go grammar |
| `tree-sitter-python` | matching | 신규 (default) | Python grammar |
| `tree-sitter-typescript` | matching | 신규 (default) | TS/TSX grammar |
| `tree-sitter-c` / `-cpp` / `-javascript` / `-json` | matching | 조건부 (USER-DECISION (b)) | 8-lang priority 채택 시 |
| `async-lsp` | ^0.2 | 신규 | LSP client (RG-MV-4) |
| `lsp-types` | ^0.97 | 신규 | LSP request/response 타입 |
| `regex` | ^1 | 신규 | @MX scan, link 패턴 |
| `wry` | ^0.45 | 조건부 (USER-DECISION (a)) | WebView (KaTeX/Mermaid 채택 시) |
| `tokio` | workspace | 이미 | async file I/O, LSP transport |
| `gpui` | 0.2.2 | 이미 (SPEC-V3-001~004 carry) | 변경 없음 |

### 11.2 USER-DECISION 게이트 (3 개)

#### Gate 1: katex-mermaid-rendering-strategy-v3-006 (MS-2 진입 직전)

질문: "KaTeX 수식과 Mermaid 다이어그램의 렌더링 전략을 어떻게 결정하시겠습니까?"

- Option (a) MS-1 부터 즉시 WebView (`wry`) — 비용 선지급 (gtk + webkit2gtk Linux 의존성, ~200 LOC setup), 가치 즉시 (MS-1 부터 수식/다이어그램 가시).
- Option (b) Native Rust 렌더 — 비용 매우 큼 (KaTeX Rust port 부재), **권장하지 않음**.
- Option (c) **(권장)** MS-1/MS-2 텍스트 fallback + MS-3 시점 (a) 채택 — MS-1/MS-2 가 빠르게 PASS, MS-3 시점에 WebView 도입.

Default: option (c). 비채택 시 progress.md 에 명시.

#### Gate 2: tree-sitter-language-priority-v3-006 (MS-2 진입 직전)

질문: "tree-sitter 번들에 포함할 언어 grammar 우선순위를 결정하시겠습니까?"

- Option (a) **(권장)** 4 lang: Rust + Go + Python + TypeScript. 비용 4 grammar + 4 queries. moai-studio 자체 + 사용자 주류 언어 충분.
- Option (b) 8 lang: 위 4 + C + C++ + JavaScript + JSON. 비용 8 grammar, 빌드 시간 증가 (CI 영향 유의).
- Option (c) 6 lang: 4 + C + JSON. 비용 6 grammar, 중간.

Default: option (a). 비채택 시 progress.md 에 명시.

#### Gate 3: lsp-server-binary-discovery-v3-006 (MS-3 진입 직전)

질문: "LSP server binary 가 `$PATH` 에 없을 때 viewer 의 동작을 결정하시겠습니까?"

- Option (i) **(권장)** Graceful degradation — syntax highlight 만 활성, 배너 1 회 표시. 사용자 환경 다양성 존중.
- Option (ii) Fail-fast — 진단 의존이 핵심이라면 popup 으로 install 안내.

Default: option (i). 비채택 시 progress.md 에 명시.

### 11.3 내부 의존성

- `crates/moai-studio-ui` — 본 SPEC 의 모든 신규 코드는 이 crate 내부.
- `crates/moai-studio-terminal` (SPEC-V3-002 완료) — 무변경 carry.
- `crates/moai-studio-workspace` (SPEC-V3-003 MS-3 완료) — 무변경 carry.
- SPEC-V3-005 (병행 SPEC) — `OpenFileEvent` struct 의 canonical 정의를 import. 본 SPEC 의 MS-1 시점에는 mock event 로 unit test, MS-2 시점에 SPEC-V3-005 import.

### 11.4 시스템/도구 제약

- Rust stable 1.93+ (SPEC-V3-002 carry).
- macOS 14+ / Ubuntu 22.04+. Windows 는 본 SPEC 범위 밖 (N10).
- LSP server binary (rust-analyzer, gopls, pyright/pylsp, typescript-language-server) — 각 언어 SDK 설치 필요. 부재 시 graceful degradation (USER-DECISION i).
- `wry` 채택 시: macOS 의 `WKWebView` framework, Linux 의 `webkit2gtk-4.1-dev` 패키지 필요.
- 기존 `mlugg/setup-zig@v2` CI 스텝 유지 (Terminal Core 링크).

### 11.5 Git / Branch 제약

- 본 SPEC 의 plan 단계 산출 (research / spec / plan) 은 `feature/SPEC-V3-004-render` 브랜치의 단일 commit.
- implementation 단계는 별도 SPEC run 단계에서 새 브랜치 (예: `feature/SPEC-V3-006-viewer`) 에서 진행.
- `main` 직접 커밋 금지 (CLAUDE.local.md §1).
- 각 MS 는 squash 머지를 위한 단일 또는 그룹 커밋으로 분리.

---

## 12. 위험 및 완화

상세 분석은 `.moai/specs/SPEC-V3-006/research.md` §8 참조.

| ID | 위험 | 영향 | 완화 전략 | research 참조 |
|----|------|------|-----------|---------------|
| R-MV1 | KaTeX/Mermaid 렌더링 전략 결정 지연 | MS-2 일정 지연 | USER-DECISION default (c) → MS-1/MS-2 PASS 가능. MS-3 시점 a 채택. | research §2.2, §6.2 |
| R-MV2 | tree-sitter Rust binding 의 C lib 빌드 시간 (CI) | CI 빌드 시간 증가 | Spike 2 (MS-2 진입 시 ≤ 2h). Cargo cache 활용. nvim-treesitter prod 검증. | research §2.3, §9 |
| R-MV3 | async-lsp + tokio 결합 LSP server lifecycle 복잡성 | LSP server stale / leak | shutdown notification + child kill. Drop on viewer close (REQ-MV-047). | research §2.4 |
| R-MV4 | WebView (wry) Linux 빌드 의존성 (gtk + webkit2gtk) | Linux CI 셋업 부담 | USER-DECISION (a) 채택 시 docker / setup-script 검증. (c) 시 미적용. | research §2.2 |
| R-MV5 | tree-sitter incremental reparse 실패 fallback | 큰 파일 재파싱 시간 비례 | REQ-MV-031 fallback 정책. v2 SPEC-M3-001 carry. | research §2.3.4 |
| R-MV6 | LSP server binary 부재 (사용자 환경 다양) | 다른 동작 환경 | USER-DECISION (i) graceful degradation, 배너 안내. | research §2.4.4, §6.2 |
| R-MV7 | @MX gutter popover GPUI API 부재 | UX 저하 | Spike 3 (MS-3 진입 시 ≤ 2h). 부재 시 inline expand fallback. | research §5.3, §9 |
| R-MV8 | 100MB 가상 스크롤 GPUI 0.2.2 native 부재 | NFR-MV-2/3 미달 | Spike 4 (uniform_list / virtualized API 검증). 부재 시 자체 viewport 계산. | research §2.5, §9 |
| R-MV9 | SPEC-V3-005 미완 시 file-open 트리거 부재 | e2e 검증 차단 | mock event 로 unit test (MS-1), 통합 e2e 는 SPEC-V3-005 PASS 후 합의된 시점. | research §11 |
| R-MV10 | LeafKind enum 도입이 SPEC-V3-004 generic L 의 placeholder 와 호환 깨뜨림 | SPEC-V3-004 regression | LeafKind::Empty 가 placeholder 역할 + impl Render. SPEC-V3-004 무변경. AC-MV-9 보증. | research §3.2 |
| R-MV11 | KaTeX/Mermaid WebView 에서 보안 위험 (외부 JS 실행) | XSS / RCE | WebView 의 sandbox 기본 활성, file:// scheme + CSP, 외부 네트워크 허용 안 함. | research §2.2 |
| R-MV12 | LSP server 의 stdout/stderr 가 UI freeze 유발 | viewer responsiveness 저하 | tokio non-blocking I/O + bounded channel buffer. backpressure 정책. | research §2.4 |

---

## 13. 참조 문서

### 13.1 본 레포 내

- `.moai/specs/SPEC-V3-006/research.md` — 본 SPEC 의 코드베이스 분석 + 라이브러리 비교.
- `.moai/specs/SPEC-V3-006/plan.md` — Milestone × Task 표.
- `.moai/specs/SPEC-V3-005/spec.md` (병행) — `OpenFileEvent` 의 canonical 정의 (예정).
- `.moai/specs/SPEC-V3-004/spec.md` §5 (RG-R-2) — `render_pane_tree<L>` generic. §9 — RootView 구조.
- `.moai/specs/SPEC-V3-003/spec.md` — Pane/Tab logic, last_focused_pane.
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core 무변경 원칙.
- `.moai/specs/SPEC-V3-001/spec.md` — RootView scaffold, design tokens.
- `.moai/specs/SPEC-M3-001/spec.md` (v2 legacy) — SwiftTreeSitter / LSP 진단 / @MX 거터 정책의 reference. v2 → v3 등가 매핑.
- `.claude/rules/moai/core/lsp-client.md` (SPEC-LSP-CORE-002) — LSP client 정책 (powernap = Go context, Rust 환경에서는 등가 매핑).
- `.claude/rules/moai/core/moai-constitution.md` "MX Tag Quality Gates" — 4 종 태그 정의.

### 13.2 외부 참조

- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) — Markdown 파서.
- [tree-sitter](https://tree-sitter.github.io/tree-sitter/) — Incremental parser library.
- [async-lsp](https://github.com/oxalica/async-lsp) — Rust LSP client.
- [lsp-types](https://github.com/gluon-lang/lsp-types) — LSP type definitions.
- [KaTeX](https://katex.org/) — Math rendering library.
- [Mermaid](https://mermaid.js.org/) — Diagram rendering library.
- [wry](https://github.com/tauri-apps/wry) — Rust WebView.
- [Helix LSP integration](https://github.com/helix-editor/helix/tree/master/helix-lsp) — async-lsp 채택 reference.
- [Zed markdown crate](https://github.com/zed-industries/zed/tree/main/crates/markdown) — pulldown-cmark + GPUI 통합 reference.

---

## 14. Exclusions

본 SPEC 이 명시적으로 **다루지 않는** 항목 (별도 SPEC 으로 분리):

- E1. **Tri-pane Diff** (v2 SPEC-M3-001 RG-M3-4) — git diff visualization, side-by-side / inline diff.
- E2. **Time-travel viewer** (v2 SPEC-M3-001 RG-M3-5) — git history slider.
- E3. **편집 기능** — 텍스트 편집, 자동 완성, code action, 리팩터링. 본 SPEC 은 read-only viewer.
- E4. **LSP 의 진단 외 기능** — completion, hover (diagnostic 외), goto-def, find-references, formatting, code lens.
- E5. **@MX 태그 SQLite cache** — v2 SPEC-M3-001 의 `mx_tags` 테이블 / fan_in 정적 분석 / cross-file index.
- E6. **다중 인코딩 (UTF-16 / Shift-JIS / EUC-KR)** — UTF-8 + lossy 만.
- E7. **Markdown 본문 임베디드 이미지 (jpg/png) 렌더** — 텍스트 + alt 만.
- E8. **PDF / docx / 그 외 binary viewer** — 별도 SPEC.
- E9. **Markdown link 의 외부 URL → 시스템 브라우저 open** — v1.0.0 은 SPEC ID link 만 처리.
- E10. **Windows 빌드** — SPEC-V3-002/003/004 carry.
- E11. **CodeViewer 의 split / multi-cursor** — 단일 viewer instance.
- E12. **언어 server 외 도구 (linter, formatter) 통합** — LSP 만.
- E13. **plug-in 방식 grammar / LSP server 추가** — 별도 SPEC.
- E14. **GPUI 0.3+ 마이그레이션** — Phase 7+ 별도 SPEC.

---

## 15. 용어 정의

- **Surface**: moai-studio 의 4 가지 leaf 종류 (Terminal / FileExplorer / MarkdownViewer / CodeViewer). PaneTree 의 leaf payload 가 surface entity.
- **LeafKind**: 4 surface 의 다형성을 표현하는 enum. SPEC-V3-004 의 generic L 자리에 들어간다.
- **OpenFileEvent**: SPEC-V3-005 가 발행하고 본 SPEC 이 수신하는 file-open trigger event.
- **EARS SPEC**: MoAI-ADK 의 `.moai/specs/SPEC-XXX/spec.md` 형식의 EARS (Easy Approach to Requirements Syntax) 마크다운 문서.
- **CommonMark**: 표준 Markdown 사양 (https://commonmark.org/).
- **GFM**: GitHub Flavored Markdown — table, strikethrough, tasklist, footnote 등 확장.
- **KaTeX**: JS 기반 수학 수식 렌더 라이브러리 (https://katex.org/).
- **Mermaid**: JS 기반 다이어그램 렌더 라이브러리 (graph, flowchart, sequence 등).
- **tree-sitter**: incremental parser library, capture name 기반 syntax highlight 제공.
- **Highlight queries (`highlights.scm`)**: tree-sitter grammar 의 capture 패턴 정의.
- **LSP**: Language Server Protocol — 언어별 진단 / completion / hover 등 표준 프로토콜.
- **Squiggly underline**: 진단 위치를 텍스트 아래에 물결 / 점선 형태로 표시.
- **@MX gutter**: 코드 좌측의 narrow column 영역 (폭 ~20px) 으로 @MX 태그 아이콘을 표시.
- **fan_in**: 해당 함수를 호출하는 다른 함수의 수. v1.0.0 은 정적 분석 미지원이라 "N/A".
- **Virtual scroll**: 화면에 보이는 라인만 element tree 에 마운트하는 기법.
- **USER-DECISION 게이트**: 명확한 default 와 옵션이 있으나 사용자 명시 결정이 필요한 분기점. 본 SPEC 은 3 개.

---

## 16. 열린 결정 사항

| ID | 결정 사항 | Default / 권장 | 결정 시점 |
|----|----------|----------------|----------|
| OD-MV1 | KaTeX / Mermaid 렌더링 전략 | (c) MS-1/MS-2 텍스트 fallback + MS-3 시점 (a) WebView 채택 | MS-2 진입 직전 ([USER-DECISION-REQUIRED: katex-mermaid-rendering-strategy-v3-006]) |
| OD-MV2 | tree-sitter 언어 priority | (a) 4 lang: Rust + Go + Python + TypeScript | MS-2 진입 직전 ([USER-DECISION-REQUIRED: tree-sitter-language-priority-v3-006]) |
| OD-MV3 | LSP server binary 부재 시 동작 | (i) Graceful degradation | MS-3 진입 직전 ([USER-DECISION-REQUIRED: lsp-server-binary-discovery-v3-006]) |
| OD-MV4 | LSP client crate 의 위치 | `crates/moai-studio-ui/src/lsp/` 내부 모듈 (현 SPEC) → 추후 별도 crate 추출 (별도 SPEC) | 본 SPEC plan 시점 확정 |
| OD-MV5 | viewer 모듈의 위치 | `crates/moai-studio-ui/src/viewer/` (단일 crate 내) | 본 SPEC plan 시점 확정 |
| OD-MV6 | @MX scan 정밀도 | per-file line-based regex (in-memory only) | 본 SPEC v1.0.0 확정. SQLite cache + fan_in 은 별도 SPEC. |
| OD-MV7 | Markdown link 의 SPEC ID 패턴 | 정규식 `SPEC-[A-Z0-9]+-[0-9]+` 매칭 시 OpenFileEvent | 본 SPEC v1.0.0 확정 |
| OD-MV8 | 가상 스크롤 line_height 결정 | hardcoded 18px (MS-1 기본) → MS-3 시점 design token 추출 | MS-3 시점 |

---

작성: 2026-04-25
브랜치: `feature/SPEC-V3-004-render` (plan 단계 commit)
다음: implementation phase (`/moai run SPEC-V3-006`) — 별도 브랜치 (예: `feature/SPEC-V3-006-viewer`)
