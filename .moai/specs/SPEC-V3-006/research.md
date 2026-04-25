# SPEC-V3-006 Research — Markdown / Code Viewer Surface

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render` (SPEC-V3-006 의 plan 단계 산출은 본 브랜치에서 작성, implementation 은 향후 별도 브랜치)
선행: SPEC-V3-001 (RootView scaffold), SPEC-V3-002 (Terminal Core, 무관 carry), SPEC-V3-003 (Pane/Tab logic), SPEC-V3-004 (Render Layer Integration — RootView ↔ TabContainer 배선)
병행: SPEC-V3-005 (File Explorer Surface — 본 SPEC 의 file-open 트리거 공급자)
범위: moai-studio 의 4 대 surface (Terminal / FileTree / **MarkdownViewer** / **CodeViewer**) 중 두 viewer surface 의 Rust + GPUI 0.2.2 구현. EARS SPEC 마크다운 (CommonMark + GFM + KaTeX 수식 + Mermaid 다이어그램) 과 코드 (tree-sitter syntax highlight + LSP 진단 inline + @MX gutter) 두 viewer 를 단일 SPEC 으로 통합한다.

---

## 1. 동기 — 왜 두 viewer 를 묶는가

### 1.1 4-surface 비전 위치

`.moai/design/v3/` 의 v3 비전은 moai-studio 가 다음 4 surface 를 단일 GPUI shell 위에서 일관 렌더하는 것을 정의한다.

| Surface | 핵심 기능 | 본 SPEC 책임 | 다른 SPEC |
|---------|-----------|--------------|-----------|
| Terminal | PTY + ANSI render + 키 입력 | — | SPEC-V3-002 (완료) |
| FileExplorer | 파일 트리 + open trigger 공급 | — | SPEC-V3-005 (병행) |
| **MarkdownViewer** | EARS SPEC / docs 렌더 (KaTeX, Mermaid 포함) | ✅ 본 SPEC | — |
| **CodeViewer** | 소스 코드 syntax highlight + LSP 진단 + @MX gutter | ✅ 본 SPEC | — |

두 viewer 를 **하나의 SPEC** 으로 묶는 이유는:

- 공통 leaf-payload 패턴 (`Entity<XxxView>` 가 `PaneTree::Leaf` 의 payload) 을 한 번에 정의한다 → SPEC-V3-004 의 leaf 추상이 placeholder String 에서 실제 viewer entity 로 점진 교체될 수 있다.
- 공통 파일 로딩 경로 (`tokio::fs::read_to_string` + 인코딩 감지 + 가상 스크롤) 가 두 viewer 모두 필요 — 중복 SPEC 회피.
- @MX 거터 정책은 코드 뷰어 전용이지만, 마크다운 본문에서 SPEC 식별자 link (예: `SPEC-V3-006`) 클릭 시 마크다운 뷰어가 해당 SPEC 파일을 여는 동작과 한 쌍이다.
- 본 SPEC 의 EARS group 분리 (RG-MV-1 ~ RG-MV-6) 가 각각 한 viewer 에 명확히 귀속되므로 통합 SPEC 의 indirection 비용이 작다.

### 1.2 SPEC-V3-005 와의 분담 (병행 SPEC)

| 책임 | SPEC-V3-005 (File Explorer) | SPEC-V3-006 (본 SPEC) |
|------|----------------------------|----------------------|
| 파일 트리 위젯 / 디렉터리 watch | ✅ | ❌ |
| `OpenFileEvent { path, surface_hint }` 발행 | ✅ (event 정의 + emit) | ❌ |
| `OpenFileEvent` 수신 후 파일 read + viewer entity 생성 | ❌ | ✅ |
| viewer leaf 가 활성 탭의 PaneTree leaf 로 마운트 | ❌ (leaf 마운트는 본 SPEC + SPEC-V3-004 carry) | ✅ |
| 파일 → surface 결정 로직 (확장자별 라우팅) | ✅ (hint 결정) | ✅ (최종 선택, hint 우선) |
| 가상 스크롤 (대용량 파일) | — | ✅ |

본 SPEC 이 SPEC-V3-005 의 `OpenFileEvent` 의 **수신자 (consumer)** 다. SPEC-V3-005 가 PASS 되기 전에는 본 SPEC 의 file-open 동작이 배선될 수 없으므로, MS-1 시점의 unit test 는 mock event 로, 통합 e2e 는 SPEC-V3-005 와 합의된 시점 (양 SPEC 모두 PASS 후) 에 검증한다 — research §11.2.

### 1.3 SPEC-V3-004 entity 패턴 의존

SPEC-V3-004 가 다음을 확정했다:

- `RootView.tab_container: Option<Entity<TabContainer>>`
- `TabContainer` 의 활성 탭 `pane_tree: PaneTree<L>` 의 leaf payload 타입 `L` 은 generic.
- SPEC-V3-004 시점의 leaf payload 는 placeholder (예: `String`) 또는 단일 `Entity<TerminalSurface>` 재사용.

본 SPEC 은 leaf payload 의 실제 타입을 다음 enum 으로 확정한다:

```text
pub enum LeafKind {
    Terminal(Entity<TerminalSurface>),
    Markdown(Entity<MarkdownViewer>),
    Code(Entity<CodeViewer>),
    Empty,                        // 분할 직후 / placeholder
}
```

SPEC-V3-004 의 `render_pane_tree<L>` 는 `L: Render + 'static` 만 요구하므로 `LeafKind` 가 `Render` 를 구현하면 그대로 호환. **SPEC-V3-004 공개 API 는 변경하지 않는다** (RG-MV-7 carry).

### 1.4 v2 SPEC-M3-001 의 cross-reference

레거시 v2 (Swift Shell) 의 `SPEC-M3-001` 은 SwiftTreeSitter + LSP + @MX 거터 + tri-pane diff + time-travel 을 단일 SPEC 으로 정의했다. 본 SPEC 은 그 SPEC 의 다음 하위 집합만 v3 (Rust + GPUI) 로 이관한다:

| v2 SPEC-M3-001 항목 | v3 SPEC-V3-006 매핑 | 비고 |
|--------------------|---------------------|------|
| RG-M3-1 SwiftTreeSitter (6 lang grammar) | **RG-MV-3 tree-sitter (4 lang grammar 우선)** | Swift binding → `tree-sitter` Rust crate, 언어 priority USER-DECISION |
| RG-M3-2 LSP 진단 (sourcekit-lsp/gopls/rust-analyzer) | **RG-MV-4 LSP 진단** | `powernap` LSP client (`.claude/rules/moai/core/lsp-client.md`) 재사용 |
| RG-M3-3 @MX 거터 | **RG-MV-5 @MX gutter** | 4 종 태그 (ANCHOR/WARN/NOTE/TODO), v2 와 정책 동일 |
| RG-M3-4 Tri-pane Diff | ❌ 본 SPEC 범위 외 (별도 SPEC) | E1 |
| RG-M3-5 Time-travel | ❌ 본 SPEC 범위 외 (별도 SPEC) | E2 |
| RG-M3-6 성능 / 메모리 | RG-MV-6 가상 스크롤 + NFR 으로 부분 carry | 60fps 스크롤은 v3 에서도 동일 목표 |
| (없음) Markdown 렌더 | **RG-MV-1, RG-MV-2 신규** | v2 에는 별도 markdown viewer SPEC 부재, 본 SPEC 이 신규 정의 |

v2 에서 v3 로 옮기지 않는 항목은 본 SPEC §14 Exclusions 로 명시.

### 1.5 사용자 가시 정의

본 SPEC 이 PASS 한 시점에 사용자가 `cargo run -p moai-studio-app` 으로 다음을 직접 관찰할 수 있어야 한다:

1. FileExplorer 에서 `.md` 파일 더블클릭 → 활성 탭 leaf 가 MarkdownViewer 로 교체되어 CommonMark + GFM 본문이 가시.
2. EARS SPEC 마크다운 본문의 `$$ E = mc^2 $$` 수식이 KaTeX 로 렌더되어 표시 (전략 결정 후).
3. 마크다운 본문의 ` ```mermaid ... ``` ` 코드 블록이 Mermaid 다이어그램으로 렌더 (전략 결정 후).
4. FileExplorer 에서 `.rs` / `.go` / `.py` / `.ts` 파일 더블클릭 → CodeViewer 가 tree-sitter 기반 syntax highlight 로 토큰 색상 표시.
5. 활성 LSP 서버가 진단을 emit 하면 코드 뷰어 본문에 squiggly underline + hover 시 진단 메시지 tooltip 가시.
6. 코드 본문에 `// @MX:ANCHOR ...` / `// @MX:WARN ...` / `// @MX:NOTE ...` / `// @MX:TODO ...` 가 있으면 좌측 거터에 색상 아이콘 (★ 금색 / ⚠ 주황 / ℹ 파랑 / ☐ 회색) 가시. 클릭 시 popover.
7. 100 MB 코드 파일을 열어도 첫 화면 paint < 200 ms, 스크롤 시 60 fps 유지 (가상 스크롤).

---

## 2. 라이브러리 분석

### 2.1 markdown 파서 — `pulldown-cmark`

#### 2.1.1 후보 비교

| 후보 | 라이센스 | CommonMark | GFM | 활성도 | 평가 |
|------|----------|-----------|-----|--------|------|
| `pulldown-cmark` (raphlinus) | MIT/Apache-2.0 | ✅ | ✅ (extension flag) | crates.io 30M+ DL | **권장** — 사실상 표준, Zed/mdbook/cargo-doc 사용 |
| `comrak` | BSD-2 | ✅ | ✅ | 활성 | 강력하나 의존 트리 큼 (libxml2 옵션) |
| `markdown-rs` | MIT | ✅ | 일부 | 신규 | mdast 모델 우수, 안정성 검증 부족 |
| `cmark-gfm-rs` | C 바인딩 | ✅ | ✅ | low | C lib 의존 — 빌드 복잡도 |

**채택**: `pulldown-cmark` (latest stable, 0.10+ 또는 0.13). 이유:
- Pure Rust, no FFI.
- CommonMark + GFM (`Options::ENABLE_TABLES`, `ENABLE_STRIKETHROUGH`, `ENABLE_TASKLISTS`, `ENABLE_FOOTNOTES`, `ENABLE_HEADING_ATTRIBUTES`) 모두 단일 crate.
- Streaming Parser API → 가상 스크롤과 결합 용이.
- Zed 채택 사례 (`zed-industries/zed` 의 markdown crate).

#### 2.1.2 GFM 옵션 활성화

```text
let mut opts = Options::empty();
opts.insert(Options::ENABLE_TABLES);
opts.insert(Options::ENABLE_FOOTNOTES);
opts.insert(Options::ENABLE_STRIKETHROUGH);
opts.insert(Options::ENABLE_TASKLISTS);
opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
opts.insert(Options::ENABLE_SMART_PUNCTUATION);  // optional
let parser = Parser::new_ext(&source, opts);
```

#### 2.1.3 Event → GPUI element 변환

`pulldown_cmark::Event` 가 stream 으로 emit:
- `Start(Tag::Heading(level, ...))` / `End(Tag::Heading(...))` → `div().text_3xl()` (h1) / `text_2xl()` (h2) ...
- `Text(s)` → `div().child(s)`
- `Code(s)` → inline code `span().bg(rgb(...))`
- `Start(Tag::CodeBlock(kind))` → 코드 블록 시작 (kind 가 `Fenced("rust")` 등 → tree-sitter highlight 적용 가능)
- `Start(Tag::Link(_, dest, _))` → 링크 (`SPEC-V3-006` 패턴 매칭 시 SPEC 파일 open trigger)

본 SPEC §3.2 의 `MarkdownEvent → IntoElement` 변환 함수가 신규 코드.

### 2.2 KaTeX / Mermaid 렌더링 전략 — **USER-DECISION 게이트**

#### 2.2.1 옵션 (a) WebView 임베드

방식: `wry` 또는 `webkit2gtk` (Linux) / `WKWebView` (macOS) 의 GPUI 통합.

장점:
- KaTeX (JS) / Mermaid (JS) 가 **upstream 그대로** 동작 — 완전 호환.
- 수식 / 다이어그램 렌더 품질이 web 표준.
- 신규 lib 의존성 적음 (`wry` 1 개).

단점:
- WebView 인스턴스의 메모리 비용 (수십 MB / 인스턴스).
- GPUI 0.2.2 의 element tree 와 WebView 의 시각 합성이 까다로움 (z-index, scroll 동기화).
- Linux WebView (gtk + webkit2gtk) 의존성 → CI 환경 빌드 부담.
- macOS / Linux 이외 (Windows) 추가 작업.

#### 2.2.2 옵션 (b) Native render

방식:
- KaTeX → Rust mathjax-port (`mathjax-rs` ?) 또는 `latex2mathml` + GPUI element. 또는 KaTeX 의 server-side render output (HTML+CSS) 을 해석하여 GPUI text element 로 매핑.
- Mermaid → Rust port 부재. `mermaid` JS 를 headless `deno_core` / `boa_engine` 으로 실행하여 SVG 출력 → SVG 를 GPUI 로 그리기. 또는 사용자가 mermaid 를 미리 SVG 로 export 한 형태만 지원 (대화형 렌더 포기).

장점:
- WebView 인스턴스 없음 → 메모리 fingerprint 작음.
- GPUI element tree 와 일관 (compositor 단순화).
- 의존성이 GPUI 위에 직접.

단점:
- KaTeX native port 가 Rust 생태에 사실상 없음 → 복잡한 수식 렌더 구현 비용 매우 큼.
- Mermaid 는 JS 엔진 의존이 사실상 필수 (Rust port 없음).
- 렌더 품질이 upstream KaTeX/Mermaid 보다 떨어질 가능성.

#### 2.2.3 옵션 (c) MS-1 / MS-2 시점 텍스트 fallback

방식:
- MS-1 시점에는 `$$...$$` 수식과 ```` ```mermaid ``` ```` 블록을 **렌더하지 않고** 코드 블록 형태 (mono font) 로 표시.
- MS-3 시점에 (a) 또는 (b) 중 하나를 채택하여 실제 렌더 추가.

장점:
- MS-1/MS-2 의 충실 마크다운 렌더 (CommonMark + GFM + tree-sitter code block) 부터 PASS 가능.
- USER-DECISION 을 MS-3 시점으로 미룰 수 있음.

단점:
- 사용자 가시 §1.5 의 (2)(3) 항목이 MS-3 까지 PENDING.
- USER-DECISION 게이트 deferral.

#### 2.2.4 권장

**USER-DECISION-REQUIRED: katex-mermaid-rendering-strategy-v3-006** — MS-2 진입 직전 발동.

- Default / 권장: **(c) MS-1/MS-2 텍스트 fallback + MS-3 시점 (a) WebView 채택**
  - 이유: KaTeX native port 부재로 (b) 비용이 크고, WebView (a) 의 메모리 비용은 수식/다이어그램 포함 SPEC 만 인스턴스화하면 통제 가능.
  - 비용: `wry` crate 의존성 추가, macOS/Linux WebView setup 코드 (≤ 200 LOC).
- Alt 1: (a) MS-1 부터 즉시 WebView — 비용 선지급, 가치 즉시.
- Alt 2: (b) Native render — 권장하지 않음, 비용 매우 큼.
- Alt 3: 영구 (c) — 수식/다이어그램 미지원 (비권장, EARS SPEC 의 핵심 기능 누락).

본 SPEC 은 default 가 (c+MS-3 a) 임을 가정하고 plan 을 작성. USER 결정 시 spec.md 의 OD-MV-? 갱신.

#### 2.2.5 통합 시점의 수식/다이어그램 추출

```text
match event {
    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) if lang.as_ref() == "mermaid" => {
        // collect text until End(Tag::CodeBlock)
        // hand off to mermaid renderer (a/b/c)
    }
    Event::Code(text) if text.starts_with("$$") && text.ends_with("$$") => {
        // KaTeX inline
    }
    // ... pulldown-cmark 의 math extension (0.10+) 사용 시 별도 Tag
}
```

`pulldown-cmark` 0.10+ 는 `Options::ENABLE_MATH` 를 통해 `$..$` (inline) / `$$..$$` (block) 수식을 별도 event 로 emit — 채택 권장 (REQ-MV-002).

### 2.3 Syntax Highlighting — `tree-sitter` (Rust crate)

#### 2.3.1 후보 비교

| 후보 | 라이센스 | API 안정성 | grammar 생태 | 평가 |
|------|----------|-----------|--------------|------|
| `tree-sitter` (Rust binding to C lib) | MIT | stable | nvim-treesitter 와 동일 grammar 재사용 | **권장** |
| `tree-sitter-rust-bindings` | community fork | 일부 | 미흡 | skip |
| `synoptic` | MIT | unstable | 자체 grammar | mini-tree-sitter, prod 채택 부담 |
| 자체 lexer (tokenizer) | — | — | language-by-language | TRUST 5 위반 (재발명) |

**채택**: `tree-sitter` Rust binding (latest, 0.22+ 또는 0.25). 이유:
- C lib 위 Rust binding 이지만 `cc` 빌드는 standard, 이미 cargo 에서 충분히 검증.
- 각 언어 grammar (`tree-sitter-rust`, `tree-sitter-go`, `tree-sitter-python`, `tree-sitter-typescript`) 도 별도 crate 로 publish.
- Incremental parsing 지원 — 편집 중 부분 reparse.
- nvim-treesitter / Helix / Zed 모두 동일 패턴 채택 — 사실상 표준.

#### 2.3.2 언어 priority — **USER-DECISION 게이트**

| 옵션 | 언어 set | 비용 | 가치 |
|------|----------|------|------|
| (a) **권장: 4 lang** | Rust, Go, Python, TypeScript | grammar 4 개 + queries 4 개 | moai-studio 자체 코드베이스 (Rust) + 사용자 주류 언어 (Go/Py/TS) 충분 |
| (b) 8 lang | + C, C++, JavaScript, JSON | grammar 8 개 + queries 8 개 | 더 넓은 커버리지, 빌드 시간 증가 |
| (c) 6 lang | + C, JSON (4 lang + 2 small) | 6 개 | 중간 |

**USER-DECISION-REQUIRED: tree-sitter-language-priority-v3-006** — MS-2 진입 직전 발동.

권장: **(a) 4 lang** Rust + Go + Python + TypeScript. 이유:
- 본 레포 자체 코드베이스 = Rust → 자기 검증.
- moai-adk-go 사용자 주류 언어 (Go).
- 사용자 SPEC 마크다운 코드 블록 fence 의 빈도 분석 (휴리스틱): rust/go/python/ts 가 80%+.
- 추후 grammar 추가는 plug-in 방식 (별도 SPEC) 으로 점진 가능.

#### 2.3.3 highlight queries

각 grammar 의 `queries/highlights.scm` 를 번들에 포함하여 token → capture name 매핑.

```text
(function_item name: (identifier) @function)
(string_literal) @string
(line_comment) @comment
```

GPUI text element 의 `text_color(rgb(...))` 로 capture name → 색상 매핑. design token 매핑은 design system 의 `code.syntax.{function|string|comment|...}.color` 사용 (별도 design token SPEC 미정 — 본 SPEC 은 hardcoded color 로 시작, MS-3 시점 token 추출).

#### 2.3.4 Incremental reparsing

```text
let mut parser = Parser::new();
parser.set_language(&tree_sitter_rust::language()).unwrap();
let tree = parser.parse(&source, None).unwrap();

// 사용자가 편집:
let edit = InputEdit { ... };
let mut tree = tree.clone();
tree.edit(&edit);
let new_tree = parser.parse(&new_source, Some(&tree)).unwrap();
```

본 SPEC 의 MS-2 시점에는 read-only viewer 라 incremental reparsing 의 직접 효익 작음. 그러나 미래 editor SPEC 의 인프라 선행 투자 가치 있음. Reparse fallback (REQ-MV-031) 은 v2 SPEC-M3-001 RG-M3-1 의 fallback 정책 carry.

### 2.4 LSP 진단 — `powernap` (재사용)

#### 2.4.1 lsp-client.md 의 결정 carry

`.claude/rules/moai/core/lsp-client.md` (SPEC-LSP-CORE-002) 가 다음을 확정:
- LSP client 는 `github.com/charmbracelet/x/powernap` v0.1.4 사용 (Go 코드 — 단, 본 SPEC 은 Rust crate context).
- **주의**: powernap 은 Go lib. 본 SPEC 은 Rust crate 환경이므로 직접 powernap 사용 불가.

#### 2.4.2 Rust LSP client 후보

본 SPEC 의 Rust 환경에서 LSP client 옵션:

| 후보 | 라이센스 | 평가 |
|------|----------|------|
| `tower-lsp` | MIT | LSP server 작성용. **client 가 아님**. |
| `lsp-types` | MIT | 타입만 (request/response 정의). 필요. |
| `async-lsp` | MIT/Apache | LSP client + server 양방향, tower 기반. **권장** |
| `lspower` | MIT | tower-lsp fork. server 위주. |

**채택**: `async-lsp` + `lsp-types`. 이유:
- Rust 생태의 LSP client 표준 (Helix, lapce 채택).
- `tower::Service` 기반 → tokio 와 자연 결합.
- request / notification 양방향 dispatch.

**lsp-client.md 와의 관계**: lsp-client.md 의 powernap 결정은 moai-adk-go 의 Go context 만 적용된다. moai-studio 의 Rust 환경에는 별도 client 가 필요. 본 SPEC 은 그 사실을 §13 에서 명시하고 lsp-client.md 의 정책 (multi-language 지원, ClientConfig 패턴) 을 Rust 등가물로 매핑한다.

#### 2.4.3 진단 데이터 흐름

```text
async-lsp Client → "textDocument/publishDiagnostics" notification
   │
   ▼
moai-studio-lsp (가칭 신규 crate 또는 ui crate 내 모듈) — 진단 cache
   │
   ▼
CodeViewer 가 활성 파일에 대한 진단 subscribe → render 시 squiggly + hover tooltip
```

**의존성 결정**: lsp client + 진단 cache 를 별도 crate `moai-studio-lsp` 로 분리할지, `moai-studio-ui` 내 모듈로 둘지.

권장: **`moai-studio-ui::lsp` 모듈** (본 SPEC MS-3 범위). 이유:
- LSP 의존이 viewer 외 다른 surface (terminal/file-explorer) 에 필요하지 않음.
- 추후 editor SPEC 등장 시 별도 crate 로 추출 가능 (점진 분리).

#### 2.4.4 LSP server spawn 정책

CodeViewer 가 활성화되면 해당 언어의 LSP server 를 spawn:
- `.rs` → `rust-analyzer`
- `.go` → `gopls`
- `.py` → `pyright` (또는 `pylsp`)
- `.ts` / `.tsx` → `typescript-language-server`

각 language server 의 binary 가 `$PATH` 에 있어야 함 (없으면 graceful degradation: syntax highlight 만, LSP 진단 disabled).

LSP server 의 lifecycle (start / shutdown) 은 SPEC 의 NFR-MV-? 로 다룸.

#### 2.4.5 진단 squiggly underline 의 GPUI 표현

GPUI 0.2.2 에 `text-decoration-line: wavy underline` 직접 등가물 부재. 우회:
- 진단 위치의 text run 을 별도 inline element 로 분리 → `border_b_2()` + `border_color(rgb(red))` + `border_dashed()` 또는 `border_wavy()` (GPUI 미지원 시 fallback dashed).
- 또는 진단 위치 아래에 1px tall element 를 absolute position 으로 그리기.

**Spike 1 (MS-3 진입 시 ≤ 4h budget)**: GPUI 0.2.2 의 text decoration API 표면 검증.

### 2.5 가상 스크롤 (대용량 파일)

#### 2.5.1 후보

| 후보 | 평가 |
|------|------|
| 자체 viewport 계산 + line buffer | GPUI 가 native scroll element 미제공 → 자체 구현 권장 |
| `egui` style virtual scroll | egui crate 의존 — overkill |

GPUI 0.2.2 의 `uniform_list` / `virtualized` API 가 있는지 확인 필요 (Spike 2, MS-3 진입 시 ≤ 2h).

#### 2.5.2 알고리즘 골조

```text
struct VirtualScroll {
    line_count: usize,
    line_height_px: f32,
    viewport_top_px: f32,
    viewport_height_px: f32,
}

impl VirtualScroll {
    fn visible_range(&self) -> Range<usize> {
        let first = (self.viewport_top_px / self.line_height_px).floor() as usize;
        let count = (self.viewport_height_px / self.line_height_px).ceil() as usize + 2;
        first..(first + count).min(self.line_count)
    }
}
```

100 MB / 평균 라인 80 byte → 약 1.25M lines. 보이는 라인만 element 로 마운트하면 element tree 크기는 O(viewport) = 50-100 lines.

---

## 3. 아키텍처 — Module Layout

### 3.1 Module 트리 (계획)

```text
crates/moai-studio-ui/src/
├── lib.rs                    # SPEC-V3-001 ~ V3-004 carry
├── tabs/                     # SPEC-V3-003/004 carry, 무변경
├── panes/                    # SPEC-V3-003/004 carry, 무변경
├── terminal/                 # SPEC-V3-002 carry, 무변경
├── viewer/                   # 본 SPEC 신규
│   ├── mod.rs                # LeafKind enum + dispatch
│   ├── markdown/             # RG-MV-1, RG-MV-2
│   │   ├── mod.rs            # MarkdownViewer struct + impl Render
│   │   ├── parser.rs         # pulldown-cmark wrapper
│   │   ├── katex.rs          # 수식 렌더 (USER-DECISION 후 분기)
│   │   ├── mermaid.rs        # 다이어그램 렌더 (USER-DECISION 후 분기)
│   │   └── tests.rs
│   ├── code/                 # RG-MV-3, RG-MV-4, RG-MV-5
│   │   ├── mod.rs            # CodeViewer struct + impl Render
│   │   ├── highlight.rs      # tree-sitter wrapper
│   │   ├── languages.rs      # 4 lang (or 8) registry
│   │   ├── gutter.rs         # @MX gutter rendering
│   │   ├── mx_scan.rs        # @MX 태그 파싱 (line-based)
│   │   └── tests.rs
│   ├── diagnostics.rs        # LSP 진단 cache + squiggly render
│   └── scroll.rs             # 가상 스크롤 (RG-MV-6)
└── lsp/                      # 본 SPEC 신규 (MS-3)
    ├── mod.rs                # LSP client 추상
    ├── server_registry.rs    # 언어별 server config
    └── tests.rs
```

### 3.2 LeafKind dispatch

```text
pub enum LeafKind {
    Empty,
    Terminal(Entity<TerminalSurface>),
    Markdown(Entity<MarkdownViewer>),
    Code(Entity<CodeViewer>),
}

impl gpui::Render for LeafKind {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self {
            LeafKind::Empty => empty_leaf_placeholder(),
            LeafKind::Terminal(e) => e.clone().into_element(),
            LeafKind::Markdown(e) => e.clone().into_element(),
            LeafKind::Code(e) => e.clone().into_element(),
        }
    }
}
```

`PaneTree<LeafKind>` 가 SPEC-V3-004 의 generic 자리에 들어간다. SPEC-V3-004 의 `render_pane_tree<L: Render>` 가 그대로 호환.

### 3.3 OpenFileEvent 수신 (SPEC-V3-005 와의 인터페이스)

```text
// SPEC-V3-005 가 정의하는 event (본 SPEC 은 consumer)
pub struct OpenFileEvent {
    pub path: PathBuf,
    pub surface_hint: Option<SurfaceHint>,  // None 이면 본 SPEC 의 라우터가 결정
}

pub enum SurfaceHint {
    Markdown,
    Code,
    Terminal,
}

impl RootView {
    fn handle_open_file(&mut self, ev: OpenFileEvent, cx: &mut Context<Self>) {
        let surface = ev.surface_hint.unwrap_or_else(|| route_by_extension(&ev.path));
        let leaf = match surface {
            SurfaceHint::Markdown => {
                let mv = cx.new(|cx| MarkdownViewer::open(ev.path.clone(), cx));
                LeafKind::Markdown(mv)
            }
            SurfaceHint::Code => {
                let cv = cx.new(|cx| CodeViewer::open(ev.path.clone(), cx));
                LeafKind::Code(cv)
            }
            SurfaceHint::Terminal => {
                // SPEC-V3-002 carry
            }
        };
        // leaf 를 활성 탭의 last_focused_pane 위치에 마운트 (SPEC-V3-004 의 set_leaf_payload)
    }
}
```

`route_by_extension`: `.md`/`.markdown` → Markdown, `.rs`/`.go`/`.py`/`.ts`/`.tsx` → Code, `.txt`/`.json` → Code (text mode), 그 외 → 거부 (binary file 경고).

---

## 4. 파일 로딩 / 인코딩 / 가상 스크롤

### 4.1 파일 read 정책

```text
async fn read_file_for_viewer(path: &Path) -> Result<ViewerSource, ViewerError> {
    let bytes = tokio::fs::read(path).await?;
    let max_size_mb = 200;  // hard cap
    if bytes.len() > max_size_mb * 1024 * 1024 {
        return Err(ViewerError::TooLarge { path: path.to_path_buf(), bytes: bytes.len() });
    }
    let (encoding, source) = decode_utf8_or_lossy(&bytes);
    Ok(ViewerSource { path, source, encoding })
}
```

- UTF-8 우선, 실패 시 `String::from_utf8_lossy` (모든 invalid byte 를 U+FFFD 로 대체).
- BOM detection, UTF-16 / Shift-JIS 등 multi-encoding 지원은 본 SPEC 범위 외 (E?).

### 4.2 Viewer entity init

```text
impl MarkdownViewer {
    pub fn open(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let source_loading = AsyncTask::new(read_file_for_viewer(&path));
        Self { path, state: LoadingState::Loading(source_loading), .. }
    }
}

impl Render for MarkdownViewer {
    fn render(...) -> impl IntoElement {
        match &self.state {
            LoadingState::Loading(_) => spinner(),
            LoadingState::Ready(src) => render_markdown(&src.source, cx),
            LoadingState::Error(e) => error_view(e),
        }
    }
}
```

GPUI 0.2.2 의 async task API 와 entity update 패턴 (Spike 0, ≤ 2h, MS-1 진입 시) 검증.

---

## 5. @MX gutter 정책

### 5.1 4 종 태그

`.claude/rules/moai/core/moai-constitution.md` "MX Tag Quality Gates" 의 정의 carry:

| Tag | 시각 | 색상 | popover |
|-----|------|------|---------|
| `@MX:ANCHOR` | ★ | gold (`#d4a017`) | fan_in 카운트 + SPEC link |
| `@MX:WARN` | ⚠ | orange (`#ff8c1a`) | REASON 링크 (없으면 "REASON required" 경고) |
| `@MX:NOTE` | ℹ | blue (`#4080d0`) | NOTE 본문 |
| `@MX:TODO` | ☐ | gray (`#888`) | TODO 본문 |

### 5.2 Scan 알고리즘

본 SPEC v1.0.0 은 **per-file line-based regex scan**:

```text
fn scan_mx_tags(source: &str) -> Vec<MxTag> {
    let re = Regex::new(r"@MX:(NOTE|WARN|ANCHOR|TODO)\s*(.*)").unwrap();
    source.lines().enumerate().filter_map(|(idx, line)| {
        re.captures(line).map(|caps| MxTag {
            line: idx + 1,
            kind: caps[1].parse().unwrap(),
            body: caps[2].to_string(),
            reason: scan_reason_subline(source, idx),
            fan_in: 0,  // 본 SPEC 은 정적 분석 미지원, "N/A"
            spec_id: extract_spec_id(&caps[2]),
        })
    }).collect()
}
```

v2 SPEC-M3-001 의 SQLite cache (`mx_tags` 테이블) 는 본 SPEC 범위 외 (E5). 본 SPEC 은 활성 파일만 in-memory scan 하고 결과를 cache 하지 않는다.

### 5.3 Gutter element 렌더

```text
fn render_mx_gutter(tags: &[MxTag], visible_range: Range<usize>) -> impl IntoElement {
    let mut col = div().flex().flex_col().w(px(20.0));
    for line_no in visible_range {
        let tag = tags.iter().find(|t| t.line == line_no);
        let icon = match tag {
            Some(t) if t.kind == MxKind::Anchor => mx_icon("★", "gold"),
            Some(t) if t.kind == MxKind::Warn => mx_icon("⚠", "orange"),
            Some(t) if t.kind == MxKind::Note => mx_icon("ℹ", "blue"),
            Some(t) if t.kind == MxKind::Todo => mx_icon("☐", "gray"),
            None => empty_gutter_cell(),
        };
        col = col.child(icon);
    }
    col
}
```

클릭 시 popover (GPUI 0.2.2 의 popup API 검증 — Spike 3).

---

## 6. 의존성 결정 매트릭스

### 6.1 신규 외부 crate

| Crate | 버전 | 용도 | 의무? | 라이센스 |
|-------|------|------|-------|----------|
| `pulldown-cmark` | ^0.13 (또는 0.10+) | Markdown 파싱 | 필수 (RG-MV-1) | MIT/Apache |
| `tree-sitter` | ^0.25 (또는 0.22+) | Syntax 파싱 | 필수 (RG-MV-3) | MIT |
| `tree-sitter-rust` | matching version | Rust grammar | USER-DECISION | MIT |
| `tree-sitter-go` | matching | Go grammar | USER-DECISION | MIT |
| `tree-sitter-python` | matching | Python grammar | USER-DECISION | MIT |
| `tree-sitter-typescript` | matching | TS/TSX grammar | USER-DECISION | MIT |
| `async-lsp` | ^0.2 | LSP client | 필수 (RG-MV-4) | MIT/Apache |
| `lsp-types` | ^0.97 | LSP request/response 타입 | 필수 | MIT |
| `regex` | ^1 | @MX scan, link 패턴 | 필수 | MIT/Apache |
| `wry` | ^0.45 | WebView (KaTeX/Mermaid 렌더 — USER-DECISION 결과 a 시) | 조건부 | MIT/Apache |
| `tokio` | workspace | async file I/O, LSP transport | 이미 워크스페이스 | MIT |

### 6.2 USER-DECISION 게이트 (3 개)

1. **katex-mermaid-rendering-strategy-v3-006** (MS-2 진입 직전): WebView (a) / Native (b) / Text fallback (c). 권장 (c) → MS-3 시점 (a) 채택.
2. **tree-sitter-language-priority-v3-006** (MS-2 진입 직전): 4 lang (a) / 8 lang (b) / 6 lang (c). 권장 (a).
3. **lsp-server-binary-discovery-v3-006** (MS-3 진입 직전): LSP server binary 가 `$PATH` 에 없을 때 graceful degradation (i) / fail-fast 경고 popup (ii). 권장 (i).

세 개의 게이트는 SPEC-V3-004 의 단일 게이트 (gpui-test-support) 보다 많지만 각각 분리된 분기점이라 통합 묶음 어려움.

---

## 7. SPEC-V3-002 / SPEC-V3-003 / SPEC-V3-004 변경 금지 (FROZEN)

본 SPEC 은 다음을 변경하지 않는다 (RG-MV-7 carry):

- `crates/moai-studio-terminal/**` 전체 (SPEC-V3-002 RG-V3-002-1).
- `crates/moai-studio-ui/src/terminal/**` (SPEC-V3-002, 무변경 carry).
- `crates/moai-studio-ui/src/{tabs, panes}/**` 의 **공개 API** (SPEC-V3-003/004 carry). 본 SPEC 은 leaf payload type 을 generic L 로 사용할 뿐, 기존 메서드 시그니처 무변경.
- `crates/moai-studio-ui/src/lib.rs` 의 SPEC-V3-004 가 정의한 `RootView.tab_container` 필드 / `Render for RootView` 의 큰 구조. 본 SPEC 은 `handle_open_file` 메서드 추가만.
- `crates/moai-studio-workspace/**` 의 persistence schema (SPEC-V3-003 MS-3 산출).

---

## 8. 위험 요약

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R-MV1 | KaTeX/Mermaid 렌더링 전략 결정 지연 | MS-2 일정 지연 | USER-DECISION 게이트의 default (c) → MS-3 시점 a 채택. MS-2 는 default (c) 로 PASS 가능. |
| R-MV2 | tree-sitter Rust binding 의 빌드 부담 (C lib) | CI 빌드 시간 증가 | 기존 cargo build pipeline 으로 검증. nvim-treesitter 가 이미 prod 검증. Spike 2 (MS-2 진입 시 ≤ 2h). |
| R-MV3 | async-lsp + tokio 결합의 LSP server lifecycle 복잡성 | LSP server stale / leak | shutdown notification + child kill 명시. Drop on viewer close. |
| R-MV4 | WebView (wry) Linux 빌드 (gtk + webkit2gtk dep) | Linux CI 환경 셋업 부담 | USER-DECISION (a) 채택 시 SPEC-V3-005 / 본 SPEC 의 첫 PR 에서 docker / setup-script 검증. 기본 (c) 시 비활성. |
| R-MV5 | tree-sitter incremental reparse 실패 시 fallback 비용 (full reparse) | 큰 파일에서 재파싱 시간 비례 | RG-MV-3 의 fallback 정책 (REQ-MV-031). v2 carry. |
| R-MV6 | LSP server binary 부재 (rust-analyzer 등 미설치) | 사용자 환경마다 다른 동작 | USER-DECISION 게이트 3 (graceful degradation) 채택 시 syntax highlight 만 동작, 배너 안내. |
| R-MV7 | @MX gutter 의 popover GPUI API 부재 | UX 저하 | Spike 3 (MS-3 진입 시 ≤ 2h). 부재 시 inline expand fallback. |
| R-MV8 | 100MB 파일 가상 스크롤의 GPUI 0.2.2 native 지원 부재 | NFR-MV-? 미달 | Spike 4 (uniform_list / virtualized API 검증). 부재 시 자체 viewport 계산 (≤ 200 LOC). |
| R-MV9 | SPEC-V3-005 미완 시 file-open 트리거 부재 | e2e 검증 차단 | mock event 로 unit test, 통합 e2e 는 SPEC-V3-005 PASS 후 합의된 시점. |
| R-MV10 | leaf payload 가 `LeafKind` enum 으로 전환되며 SPEC-V3-004 generic L 의 instantiation 변경 | SPEC-V3-004 PaneTree<String> placeholder 와 호환 | LeafKind::Empty 가 placeholder 역할 + impl Render. SPEC-V3-004 는 무변경. |

---

## 9. Spike 계획

| Spike | 시점 | budget | 검증 항목 |
|-------|------|--------|-----------|
| Spike 0 | MS-1 진입 직후 | ≤ 2h | GPUI 0.2.2 의 async file I/O + entity state machine (Loading/Ready/Error) 패턴 |
| Spike 1 | MS-3 진입 직후 | ≤ 4h | GPUI 0.2.2 의 text decoration (squiggly underline 등가물) API 표면 |
| Spike 2 | MS-2 진입 직후 | ≤ 2h | tree-sitter Rust binding 의 cargo build 시간 (CI 영향) |
| Spike 3 | MS-3 진입 직후 | ≤ 2h | GPUI 0.2.2 의 popover / popup API 표면 |
| Spike 4 | MS-3 진입 직후 | ≤ 2h | GPUI 0.2.2 의 가상 스크롤 / uniform_list / virtualized API 표면 |

---

## 10. 테스트 전략

### 10.1 Unit (logic-only)

- markdown parser → element spec representation (테스트 가능한 enum)
- @MX scan: 각 tag kind 별 추출 정확성
- tree-sitter highlight: token capture 와 line/col mapping
- LSP 진단 cache: publishDiagnostics → 위치별 진단 lookup
- 가상 스크롤: visible_range 계산
- LeafKind dispatch: extension → SurfaceHint 라우팅

### 10.2 Integration (GPUI test-support — SPEC-V3-004 USER-DECISION 결과에 의존)

SPEC-V3-004 가 USER-DECISION (a) 채택 시 그 dev-dependencies (gpui test-support) 를 본 SPEC 도 그대로 활용:
- MarkdownViewer entity → render → element tree 검증
- CodeViewer entity → render → 토큰 색상 + 진단 squiggly 자식 element 존재 확인
- 가상 스크롤 viewport_top_px 변경 → visible lines element 변경

### 10.3 Manual smoke (MS-3 종결)

- §1.5 의 7 가지 사용자 가시 동작 직접 확인.

### 10.4 LSP integration test

- `rust-analyzer` 가 `$PATH` 에 있는 환경 (CI 검증):
  - 의도적 syntax error 가 있는 `.rs` 파일 open → 진단 표시 확인
  - LSP shutdown on viewer close 확인 (no zombie process)

---

## 11. SPEC-V3-005 와의 인터페이스 시점

### 11.1 OpenFileEvent contract

본 SPEC 은 SPEC-V3-005 의 OpenFileEvent **타입 정의** 만 의존하고, 실제 emitter 동작은 SPEC-V3-005 가 책임진다. 시점:

- SPEC-V3-006 MS-1: mock OpenFileEvent 로 file-open 경로 unit test.
- SPEC-V3-006 MS-2: SPEC-V3-005 의 OpenFileEvent struct 가 published. 본 SPEC 은 그것을 import.
- SPEC-V3-006 MS-3 + SPEC-V3-005 PASS 후: 양 SPEC 합의된 e2e 검증.

### 11.2 인터페이스 충돌 회피

OpenFileEvent 의 필드 / variant 가 양 SPEC 사이에서 변경 시 양쪽 모두 영향. 합의 채널:

- SPEC-V3-005 의 spec.md 에서 OpenFileEvent 정의를 canonical 로 명시 (본 SPEC 은 그것을 reference).
- 변경 시 양 SPEC 의 plan revision 동시 수행.
- 본 SPEC §13 References 에서 SPEC-V3-005 의 정확한 정의 위치를 가리킴.

---

## 12. 결정 요약 (Decisions)

| ID | 결정 | 근거 |
|----|------|------|
| D-MV1 | Markdown 파서 = `pulldown-cmark` | 사실상 표준, GFM 지원, pure Rust |
| D-MV2 | KaTeX/Mermaid 전략 = USER-DECISION (default c → MS-3 a) | KaTeX native port 부재, WebView 메모리 비용 통제 가능 |
| D-MV3 | Syntax 라이브러리 = `tree-sitter` Rust binding | nvim-treesitter / Helix / Zed 표준 |
| D-MV4 | tree-sitter 언어 priority = USER-DECISION (default 4: rust/go/py/ts) | moai-studio 자체 + 사용자 주류 언어 |
| D-MV5 | LSP client = `async-lsp` + `lsp-types` (Rust) | lsp-client.md 의 powernap 결정은 Go context — Rust 등가물 매핑 |
| D-MV6 | @MX scan = per-file line-based regex (in-memory only) | v1.0.0 단순화. SQLite cache 는 별도 SPEC. |
| D-MV7 | LeafKind enum 으로 SPEC-V3-004 generic L 인스턴스화 | SPEC-V3-004 공개 API 무변경, leaf payload 다형성 추가 |
| D-MV8 | 가상 스크롤은 자체 구현 (viewport_top_px + line_height_px) | GPUI 0.2.2 native 부재 추정, Spike 4 결과 따라 변경 가능 |
| D-MV9 | OpenFileEvent 정의 = SPEC-V3-005 canonical, 본 SPEC consumer | 단일 책임 원칙 |
| D-MV10 | LSP server binary 부재 시 graceful degradation 권장 | 사용자 환경 다양성 — fail-fast 는 UX 저해 |

---

## 13. 시작 시 알아야 할 5 가지

1. 본 SPEC 은 4-surface 비전의 두 개 (MarkdownViewer + CodeViewer) 를 한 묶음으로 다룬다. SPEC-V3-005 (FileExplorer) 가 file-open 트리거를 공급하고, 본 SPEC 이 viewer 마운트를 책임진다.
2. SPEC-V3-004 의 `render_pane_tree<L>` generic 을 그대로 활용 — `L = LeafKind` enum 으로 4 가지 surface 를 통합한다. SPEC-V3-004 의 공개 API 변경 없음.
3. KaTeX / Mermaid 전략은 USER-DECISION 게이트 (default = MS-1/2 텍스트 fallback + MS-3 WebView 채택). MS-1/2 만으로도 CommonMark + GFM 충분 PASS 가능.
4. tree-sitter 언어 우선순위는 USER-DECISION 게이트 (default 4 lang = rust + go + python + typescript). moai-studio 자체 코드베이스 (Rust) 를 자기 검증 환경으로 활용.
5. LSP client 는 `lsp-client.md` 의 powernap 결정과 분리. powernap 은 Go lib, 본 SPEC 은 Rust 환경이라 `async-lsp + lsp-types` Rust crate 사용. 정책 (multi-language ClientConfig) 은 carry.

---

작성 완료: 2026-04-25
다음: spec.md (canonical contract), plan.md (execution table)
