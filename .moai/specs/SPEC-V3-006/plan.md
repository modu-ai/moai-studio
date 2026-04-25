# SPEC-V3-006 Implementation Plan

작성: MoAI (manager-spec, 2026-04-25)
브랜치 (plan 산출): `feature/SPEC-V3-004-render`
브랜치 (implementation 예정): `feature/SPEC-V3-006-viewer` (별도)
범위: SPEC-V3-006 spec.md MS-1 / MS-2 / MS-3 + 3 USER-DECISION 게이트.
선행: SPEC-V3-001 / V3-002 / V3-003 / V3-004 모두 완료 또는 plan 단계 산출 (V3-004 는 plan 산출 후 별도 run 단계).
병행: SPEC-V3-005 (File Explorer Surface) — `OpenFileEvent` canonical 정의 공급자.

---

## 1. Milestone × Task 표

| Task | Milestone | 책임 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|------|----------------------|------|----|
| **T1** | MS-1 | LeafKind enum + impl Render | `crates/moai-studio-ui/src/viewer/mod.rs` (신규) | — | AC-MV-1 (선행), AC-MV-9 |
| **T2** | MS-1 | RootView::handle_open_file 메서드 + route_by_extension | `crates/moai-studio-ui/src/lib.rs` (메서드 추가만), `viewer/mod.rs` | T1 | AC-MV-1, AC-MV-11 |
| **T3** | MS-1 | MarkdownViewer entity (Loading/Ready/Error state) | `crates/moai-studio-ui/src/viewer/markdown/mod.rs` (신규) | T1 | AC-MV-1 |
| **T4** | MS-1 | pulldown-cmark Event → IntoElement 변환 | `crates/moai-studio-ui/src/viewer/markdown/parser.rs` (신규) | T3 | AC-MV-1 |
| **T5** | MS-1 | VirtualScroll 자료구조 + visible_range | `crates/moai-studio-ui/src/viewer/scroll.rs` (신규) | — | AC-MV-8 (선행) |
| **T6** | MS-1 | 파일 read async + ViewerError 정의 | `crates/moai-studio-ui/src/viewer/mod.rs` 또는 별도 `error.rs` | — | AC-MV-1, AC-MV-11 |
| **T7** | MS-1 | binary file detection (PNG/PDF/NUL byte) | `crates/moai-studio-ui/src/viewer/mod.rs::is_binary` | T6 | AC-MV-11 |
| **T8** | MS-1 | mock OpenFileEvent unit test (SPEC-V3-005 미완 환경) | `crates/moai-studio-ui/src/viewer/mod.rs::tests` | T2, T3 | AC-MV-1, AC-MV-11 |
| **T9** | MS-2 | USER-DECISION (1, 2) 게이트 호출 | (게이트 호출, Cargo.toml 갱신) | T1~T8 | OD-MV1, OD-MV2 |
| **T10** | MS-2 | tree-sitter 의존성 추가 + grammar 번들 | `crates/moai-studio-ui/Cargo.toml`, `viewer/code/languages.rs` (신규) | T9 | AC-MV-3 |
| **T11** | MS-2 | CodeViewer entity + impl Render | `crates/moai-studio-ui/src/viewer/code/mod.rs` (신규) | T10 | AC-MV-2 |
| **T12** | MS-2 | tree-sitter highlight queries 통합 | `crates/moai-studio-ui/src/viewer/code/highlight.rs` (신규) | T11 | AC-MV-2 |
| **T13** | MS-2 | Markdown 의 fenced code block highlight (T12 reuse) | `crates/moai-studio-ui/src/viewer/markdown/parser.rs` (T4 확장) | T12 | AC-MV-2 추가 |
| **T14** | MS-2 | T13 의 USER-DECISION (c) 시 mono-font fallback + 배너 | `viewer/markdown/parser.rs` | T13 | (USER-DECISION 결과 반영) |
| **T15** | MS-3 | USER-DECISION (3) 게이트 호출 | (게이트 호출) | T9~T14 | OD-MV3 |
| **T16** | MS-3 | async-lsp + lsp-types 의존성 추가 | `crates/moai-studio-ui/Cargo.toml` | T15 | AC-MV-4 |
| **T17** | MS-3 | LspClient 추상 + server_registry | `crates/moai-studio-ui/src/lsp/{mod.rs, server_registry.rs}` (신규) | T16 | AC-MV-4, AC-MV-5 |
| **T18** | MS-3 | publishDiagnostics 처리 + 진단 cache | `crates/moai-studio-ui/src/viewer/diagnostics.rs` (신규) | T17 | AC-MV-4 |
| **T19** | MS-3 | squiggly underline render + hover tooltip | `viewer/diagnostics.rs`, `viewer/code/mod.rs` (확장) | T18 | AC-MV-4 |
| **T20** | MS-3 | LSP server lifecycle (shutdown on drop) | `viewer/code/mod.rs::Drop`, `lsp/mod.rs` | T17 | AC-MV-12 |
| **T21** | MS-3 | mx_scan 함수 + MxTag struct | `crates/moai-studio-ui/src/viewer/code/mx_scan.rs` (신규) | T11 | AC-MV-6 |
| **T22** | MS-3 | @MX gutter element + popover | `crates/moai-studio-ui/src/viewer/code/gutter.rs` (신규) | T21 | AC-MV-6 |
| **T23** | MS-3 | SPEC ID link 클릭 → OpenFileEvent | `viewer/markdown/parser.rs` 확장, `viewer/mod.rs` 라우터 | T2, T22 | AC-MV-10 |
| **T24** | MS-3 | KaTeX/Mermaid 렌더 (USER-DECISION (a) 채택 시 WebView, (c) 시 fallback 유지) | `viewer/markdown/{katex.rs, mermaid.rs}` (신규), `Cargo.toml` (`wry` 추가) | T9 결과, T15 | AC-MV-7 |
| **T25** | MS-3 | 가상 스크롤 GPUI 통합 + 100MB bench | `viewer/scroll.rs` 확장, `viewer/code/mod.rs`, `benches/viewer_scroll.rs` (신규) | T5, T11 | AC-MV-8 |
| **T26** | MS-3 | LSP server binary 부재 시 graceful degradation 배너 | `lsp/mod.rs`, `viewer/code/mod.rs` | T17 | AC-MV-5 |
| **T27** | 전체 | regression + smoke test + commit | (git 작업, progress.md 갱신) | T1~T26 | AC-MV-9, AC-MV-12 |

---

## 2. T1 — LeafKind enum + impl Render

### 2.1 신규 파일

`crates/moai-studio-ui/src/viewer/mod.rs`:

```text
//! SPEC-V3-006: 4-surface viewer 통합 진입점.
//!
//! `LeafKind` enum 이 SPEC-V3-004 의 generic L 자리에 들어가 Pane leaf 의
//! 다형성을 제공한다.

pub mod markdown;
pub mod code;
pub mod diagnostics;
pub mod scroll;

use crate::terminal::TerminalSurface;
use markdown::MarkdownViewer;
use code::CodeViewer;
use gpui::{Entity, IntoElement, ParentElement, Render, Styled, div};
use std::path::Path;

/// 활성 Pane leaf 의 표시 종류.
///
/// SPEC-V3-006 RG-MV-? : SPEC-V3-004 의 `render_pane_tree<L>` generic 자리에
/// `L = LeafKind` 로 인스턴스화된다. SPEC-V3-004 공개 API 변경 없음.
// @MX:ANCHOR: leaf-kind-surface-dispatch
// @MX:REASON: SPEC-V3-006 RG-MV-?. 4 surface (Terminal/Markdown/Code/Empty) 의
//   다형성 진입점. fan_in >= 3: render_pane_tree, RootView::handle_open_file,
//   integration tests.
pub enum LeafKind {
    Empty,
    Terminal(Entity<TerminalSurface>),
    Markdown(Entity<MarkdownViewer>),
    Code(Entity<CodeViewer>),
}

impl Render for LeafKind {
    fn render(&mut self, _w: &mut gpui::Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        match self {
            LeafKind::Empty => empty_leaf_placeholder(),
            LeafKind::Terminal(e) => e.clone().into_element(),
            LeafKind::Markdown(e) => e.clone().into_element(),
            LeafKind::Code(e) => e.clone().into_element(),
        }
    }
}

fn empty_leaf_placeholder() -> impl IntoElement {
    div().size_full().bg(gpui::rgb(0x1a1a1a))
}

/// 확장자 → SurfaceHint 라우터 (SPEC-V3-005 의 OpenFileEvent.surface_hint 가 None 일 때 사용).
pub fn route_by_extension(path: &Path) -> SurfaceHint { /* T2 에서 작성 */ unimplemented!() }

pub enum SurfaceHint { Markdown, Code, Terminal }
```

### 2.2 신규 단위 테스트

- `viewer::tests::leaf_kind_empty_render_does_not_panic`
- `viewer::tests::route_by_extension_md_returns_markdown`
- `viewer::tests::route_by_extension_rs_returns_code`
- `viewer::tests::route_by_extension_unknown_returns_code` (text fallback)

### 2.3 AC 매핑

- AC-MV-1 (선행): LeafKind enum 의 인스턴스화 + impl Render 가 panic 없이 동작.
- AC-MV-9 (선행): SPEC-V3-004 의 `render_pane_tree<L>` generic 시그니처 변경 없음 검증.

---

## 3. T2 — RootView::handle_open_file 메서드

### 3.1 변경 대상

`crates/moai-studio-ui/src/lib.rs` — RootView impl block 에 메서드 추가만 (필드 추가 없음, SPEC-V3-004 carry 의 tab_container 그대로):

```text
impl RootView {
    /// SPEC-V3-006 RG-MV-8: OpenFileEvent 수신 시 viewer entity 를 생성하여
    /// 활성 탭의 last_focused_pane 자리에 마운트.
    ///
    /// SPEC-V3-005 의 OpenFileEvent canonical 정의를 import 하여 사용.
    pub fn handle_open_file(
        &mut self,
        ev: viewer::OpenFileEvent,  // SPEC-V3-005 의 type 을 re-export
        cx: &mut gpui::Context<Self>,
    ) {
        // 1. binary detection
        if viewer::is_likely_binary(&ev.path) {
            tracing::warn!(path = %ev.path.display(), "binary file rejected");
            // status bar / toast (별도 helper)
            return;
        }
        // 2. surface 결정
        let surface = ev.surface_hint.unwrap_or_else(|| viewer::route_by_extension(&ev.path));
        // 3. viewer entity 생성
        let leaf = match surface {
            viewer::SurfaceHint::Markdown => {
                let e = cx.new(|cx| viewer::markdown::MarkdownViewer::open(ev.path, cx));
                viewer::LeafKind::Markdown(e)
            }
            viewer::SurfaceHint::Code => {
                let e = cx.new(|cx| viewer::code::CodeViewer::open(ev.path, cx));
                viewer::LeafKind::Code(e)
            }
            viewer::SurfaceHint::Terminal => {
                // SPEC-V3-002 carry — placeholder, 실제 PTY-per-pane SPEC 에서 확장
                viewer::LeafKind::Empty
            }
        };
        // 4. 활성 탭의 last_focused_pane 자리에 in-place 교체
        let Some(tc) = self.tab_container.as_ref() else { return; };
        tc.update(cx, |tc, cx| {
            let focused = tc.active_tab().last_focused_pane.clone();
            if let Some(leaf_id) = focused {
                tc.active_tab_mut().pane_tree.set_leaf_payload(leaf_id, leaf).ok();
                cx.notify();
            }
        });
    }
}
```

### 3.2 영향 범위

- `RootView` 구조체: 변경 없음 (tab_container 필드 그대로).
- `Render for RootView`: 변경 없음 (SPEC-V3-004 carry).
- 신규 메서드 `handle_open_file` 만 추가.
- `pane_tree::set_leaf_payload(leaf_id, payload)` 메서드: SPEC-V3-003 의 `PaneTree<L>` 가 이 메서드를 가지고 있어야 함 — **SPEC-V3-003 와 충돌 없는지 확인 필요**. 부재 시 SPEC-V3-006 implementation 단계에서 SPEC-V3-003 의 공개 API 추가 (RG-MV-7 의 "공개 API 무변경" 과 충돌 — 아래 §3.3 escalation 참조).

### 3.3 SPEC-V3-003 호환성 escalation

SPEC-V3-003 의 `PaneTree<L>` 가 `set_leaf_payload` 메서드를 이미 제공하지 않는 경우:

- **Option A (권장)**: SPEC-V3-006 implementation 단계에서 `PaneTree<L>::set_leaf_payload(leaf_id, payload) -> Result<(), PaneTreeError>` 를 신규 추가. 기존 메서드 시그니처 무변경 → RG-MV-7 의 공개 API 무변경 원칙은 "수정 금지" 가 아니라 "확장 허용" 으로 해석. progress.md 에 명시.
- **Option B**: 본 SPEC 의 RootView::handle_open_file 이 직접 PaneTree 의 internals 를 mutate (tree::Leaf payload 직접 교체). private helper 추가만. SPEC-V3-003 변경 없음.

권장 Option A. SPEC-V3-006 implementation phase 의 첫 commit 에 SPEC-V3-003 호환성 추가.

### 3.4 AC 매핑

- AC-MV-1: handle_open_file 호출 시 LeafKind::Markdown 마운트.
- AC-MV-11: binary 파일 rejection.

### 3.5 신규 단위 테스트

- `lib::tests::handle_open_file_md_creates_markdown_leaf`
- `lib::tests::handle_open_file_rs_creates_code_leaf`
- `lib::tests::handle_open_file_binary_is_rejected`

---

## 4. T3 — MarkdownViewer entity (Loading/Ready/Error state)

### 4.1 신규 파일

`crates/moai-studio-ui/src/viewer/markdown/mod.rs`:

```text
//! SPEC-V3-006 RG-MV-1: MarkdownViewer entity.

pub mod parser;
#[cfg(feature = "katex-mermaid-webview")]
pub mod katex;
#[cfg(feature = "katex-mermaid-webview")]
pub mod mermaid;

use std::path::PathBuf;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, div, rgb};

pub struct MarkdownViewer {
    pub path: PathBuf,
    pub state: ViewerState,
    pub scroll: crate::viewer::scroll::VirtualScroll,
}

pub enum ViewerState {
    Loading,
    Ready { source: String, /* parsed events cache */ },
    Error(crate::viewer::ViewerError),
}

impl MarkdownViewer {
    pub fn open(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let _task = cx.spawn(|view, mut cx| async move {
            let result = crate::viewer::read_file_for_viewer(&path).await;
            view.update(&mut cx, |view, cx| {
                view.state = match result {
                    Ok(src) => ViewerState::Ready { source: src.source, /* ... */ },
                    Err(e) => ViewerState::Error(e),
                };
                cx.notify();
            }).ok();
        });
        Self {
            path,
            state: ViewerState::Loading,
            scroll: Default::default(),
        }
    }
}

impl Render for MarkdownViewer {
    fn render(&mut self, _w: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.state {
            ViewerState::Loading => spinner_element(),
            ViewerState::Ready { source, .. } => parser::render_markdown(source),
            ViewerState::Error(e) => error_box_element(e),
        }
    }
}
```

### 4.2 신규 단위 테스트

- `markdown::tests::markdown_viewer_initial_state_is_loading`
- `markdown::tests::markdown_viewer_open_invalid_path_transitions_to_error`

### 4.3 AC 매핑

- AC-MV-1: MarkdownViewer 가 entity 로 생성, Loading → Ready 전이.

---

## 5. T4 — pulldown-cmark Event → IntoElement

### 5.1 신규 파일

`crates/moai-studio-ui/src/viewer/markdown/parser.rs`:

```text
//! SPEC-V3-006 RG-MV-1: pulldown-cmark Event → GPUI element 변환.

use pulldown_cmark::{Parser, Options, Event, Tag, CodeBlockKind};
use gpui::{IntoElement, ParentElement, Styled, div, rgb};

pub fn render_markdown(source: &str) -> impl IntoElement {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    opts.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(source, opts);
    let mut col = div().flex().flex_col().size_full().p_4();
    let mut current_block: BlockState = BlockState::None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                current_block = BlockState::Heading(level);
            }
            Event::End(Tag::Heading(_, _, _)) => current_block = BlockState::None,
            Event::Text(s) => match current_block {
                BlockState::Heading(level) => col = col.child(heading_element(level, &s)),
                _ => col = col.child(text_element(&s)),
            },
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                current_block = BlockState::CodeBlock(lang.to_string());
            }
            Event::End(Tag::CodeBlock(_)) => current_block = BlockState::None,
            // ... DisplayMath, InlineMath, Link, ...
            _ => {}
        }
    }
    col
}

enum BlockState {
    None,
    Heading(pulldown_cmark::HeadingLevel),
    CodeBlock(String),
    // ...
}
```

### 5.2 USER-DECISION 결과 분기

- (a) WebView 채택 시: `Event::DisplayMath(s)` → `katex::render_math(s)` 호출.
- (c) text fallback: `Event::DisplayMath(s)` → `code_block_element(s, "math")`.

### 5.3 AC 매핑

- AC-MV-1: GFM table / strikethrough / tasklist 렌더 검증.

### 5.4 신규 단위 테스트

- `parser::tests::render_heading_emits_correct_size`
- `parser::tests::render_table_emits_grid_element`
- `parser::tests::render_strikethrough_emits_line_through`
- `parser::tests::render_math_with_text_fallback_emits_code_block`

---

## 6. T5 — VirtualScroll 자료구조

### 6.1 신규 파일

`crates/moai-studio-ui/src/viewer/scroll.rs`:

```text
//! SPEC-V3-006 RG-MV-6: 가상 스크롤.

use std::ops::Range;

#[derive(Default, Clone, Copy)]
pub struct VirtualScroll {
    pub line_count: usize,
    pub line_height_px: f32,
    pub viewport_top_px: f32,
    pub viewport_height_px: f32,
}

impl VirtualScroll {
    pub fn visible_range(&self) -> Range<usize> {
        if self.line_height_px <= 0.0 {
            return 0..self.line_count.min(50);
        }
        let first = (self.viewport_top_px / self.line_height_px).floor() as usize;
        let count = (self.viewport_height_px / self.line_height_px).ceil() as usize + 2;
        first..(first + count).min(self.line_count)
    }
}
```

### 6.2 신규 단위 테스트

- `scroll::tests::visible_range_at_top_starts_at_zero`
- `scroll::tests::visible_range_in_middle_calculates_correctly`
- `scroll::tests::visible_range_at_end_clamps_to_line_count`
- `scroll::tests::visible_range_with_zero_line_height_returns_default`

### 6.3 AC 매핑

- AC-MV-8 (선행): visible_range 계산 정확성.

---

## 7. T6 — 파일 read async + ViewerError 정의

### 7.1 viewer/mod.rs 확장

```text
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ViewerError {
    #[error("file too large: {path:?} ({bytes} bytes, max 200MB)")]
    TooLarge { path: PathBuf, bytes: usize },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("binary file rejected: {0:?}")]
    Binary(PathBuf),
}

pub struct ViewerSource {
    pub path: PathBuf,
    pub source: String,
    pub byte_len: usize,
}

pub async fn read_file_for_viewer(path: &Path) -> Result<ViewerSource, ViewerError> {
    let bytes = tokio::fs::read(path).await?;
    let max = 200 * 1024 * 1024;
    if bytes.len() > max {
        return Err(ViewerError::TooLarge { path: path.to_path_buf(), bytes: bytes.len() });
    }
    if is_likely_binary_bytes(&bytes) {
        return Err(ViewerError::Binary(path.to_path_buf()));
    }
    let source = String::from_utf8_lossy(&bytes).into_owned();
    Ok(ViewerSource { path: path.to_path_buf(), source, byte_len: bytes.len() })
}

pub fn is_likely_binary(path: &Path) -> bool {
    // 확장자 + magic bytes 검사
    /* T7 에서 확장 */
    false
}

fn is_likely_binary_bytes(bytes: &[u8]) -> bool {
    // PNG / PDF / JPEG magic byte 검사 + NUL byte 다수 검사
    /* T7 에서 확장 */
    false
}
```

### 7.2 AC 매핑

- AC-MV-1: read 성공 시 Ready 전이.
- AC-MV-11: binary 거부.

---

## 8. T7 — Binary detection

### 8.1 알고리즘

```text
fn is_likely_binary_bytes(bytes: &[u8]) -> bool {
    // Magic bytes
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";
    const PDF: &[u8] = b"%PDF-";
    const JPEG: &[u8] = b"\xff\xd8\xff";
    const ZIP: &[u8] = b"PK\x03\x04";
    if bytes.starts_with(PNG) || bytes.starts_with(PDF)
        || bytes.starts_with(JPEG) || bytes.starts_with(ZIP) {
        return true;
    }
    // NUL byte 비율 (첫 8KB 검사)
    let sample = &bytes[..bytes.len().min(8192)];
    let nul_count = sample.iter().filter(|&&b| b == 0).count();
    nul_count > sample.len() / 100  // 1% 이상 NUL → binary
}
```

### 8.2 신규 단위 테스트

- `viewer::tests::png_signature_is_binary`
- `viewer::tests::pdf_signature_is_binary`
- `viewer::tests::utf8_text_is_not_binary`
- `viewer::tests::null_heavy_is_binary`

### 8.3 AC 매핑

- AC-MV-11: binary 파일 rejection 정확성.

---

## 9. T8 — Mock OpenFileEvent unit test

### 9.1 SPEC-V3-005 미완 환경의 테스트 전략

```text
#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC-V3-005 의 OpenFileEvent 가 published 되기 전 mock.
    pub struct MockOpenFileEvent {
        pub path: PathBuf,
        pub surface_hint: Option<SurfaceHint>,
    }

    #[test]
    fn handle_open_file_md_creates_markdown_leaf() {
        // RootView mock + handle_open_file 직접 호출 (Context<Self> 필요)
        // SPEC-V3-004 의 USER-DECISION (a) 결과 (gpui test-support) 활용
    }
}
```

### 9.2 SPEC-V3-005 PASS 후 통합

SPEC-V3-005 가 OpenFileEvent struct 를 published 하면, MockOpenFileEvent 를 실제 type 으로 교체:
```text
use moai_studio_workspace::file_explorer::OpenFileEvent;  // SPEC-V3-005 가 export
// 또는 viewer 모듈에서 re-export
```

### 9.3 AC 매핑

- AC-MV-1: MockOpenFileEvent 로 unit 검증.
- AC-MV-11: MockOpenFileEvent (binary path) 로 unit 검증.

---

## 10. T9 — USER-DECISION 게이트 (1, 2)

### 10.1 USER-DECISION 1: katex-mermaid-rendering-strategy-v3-006

[USER-DECISION-REQUIRED: katex-mermaid-rendering-strategy-v3-006]

질문 (AskUserQuestion 형식):
- "KaTeX 수식과 Mermaid 다이어그램의 렌더링 전략을 어떻게 결정하시겠습니까?"

옵션:
- (a) MS-1 부터 즉시 WebView (`wry`) — 비용 선지급, 가치 즉시.
- (b) Native Rust 렌더 — 비용 매우 큼 (KaTeX port 부재). **권장하지 않음**.
- (c) **(권장)** MS-1/MS-2 텍스트 fallback + MS-3 시점 (a) 채택.

### 10.2 USER-DECISION 2: tree-sitter-language-priority-v3-006

[USER-DECISION-REQUIRED: tree-sitter-language-priority-v3-006]

질문:
- "tree-sitter 번들에 포함할 언어 grammar 우선순위를 결정하시겠습니까?"

옵션:
- (a) **(권장)** 4 lang: Rust + Go + Python + TypeScript.
- (b) 8 lang: + C + C++ + JavaScript + JSON.
- (c) 6 lang: + C + JSON.

### 10.3 게이트 결과 처리

- 결과를 progress.md (implementation phase) 에 기록.
- (b) 채택 시: T10 의 Cargo.toml 에 추가 grammar 의존.
- (c) 채택 시 (Mermaid/KaTeX): T14 가 mono-font fallback 으로 진행, T24 가 MS-3 시점 (a) 로 갱신.

### 10.4 AC 매핑

- OD-MV1, OD-MV2 의 결정 reflection.

---

## 11. T10 — tree-sitter 의존성 + grammar 번들

### 11.1 Cargo.toml 갱신 (USER-DECISION (a) default 4 lang)

```toml
[dependencies]
# ... existing ...
tree-sitter = "0.25"
tree-sitter-rust = "0.21"
tree-sitter-go = "0.20"
tree-sitter-python = "0.21"
tree-sitter-typescript = "0.21"
pulldown-cmark = "0.13"
async-lsp = "0.2"
lsp-types = "0.97"
regex = "1"
# ... 추가 ...

[features]
katex-mermaid-webview = ["dep:wry"]  # USER-DECISION (a) 채택 시 활성

[target.'cfg(feature = "katex-mermaid-webview")'.dependencies]
wry = "0.45"
```

### 11.2 languages.rs 신규

```text
//! SPEC-V3-006 RG-MV-3: tree-sitter grammar registry.

use tree_sitter::Language;
use std::path::Path;

pub struct GrammarEntry {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub language: fn() -> Language,
    pub highlights_query: &'static str,  // 컴파일 타임 include_str!
}

pub static SUPPORTED_LANGUAGES: &[GrammarEntry] = &[
    GrammarEntry {
        name: "rust",
        extensions: &["rs"],
        language: tree_sitter_rust::language,
        highlights_query: include_str!("../../../../../../grammars/rust/highlights.scm"),
    },
    GrammarEntry { name: "go", extensions: &["go"], language: tree_sitter_go::language, highlights_query: include_str!(...) },
    GrammarEntry { name: "python", extensions: &["py"], language: tree_sitter_python::language, highlights_query: include_str!(...) },
    GrammarEntry { name: "typescript", extensions: &["ts", "tsx"], language: tree_sitter_typescript::language_typescript, highlights_query: include_str!(...) },
];

pub fn for_extension(ext: &str) -> Option<&'static GrammarEntry> {
    SUPPORTED_LANGUAGES.iter().find(|g| g.extensions.contains(&ext))
}
```

### 11.3 highlights.scm 번들 위치

- `crates/moai-studio-ui/grammars/{rust|go|python|typescript}/highlights.scm` (신규 디렉터리).
- nvim-treesitter 의 queries 를 그대로 복제 (라이센스 호환 — 대부분 MIT).

### 11.4 AC 매핑

- AC-MV-3: SUPPORTED_LANGUAGES.len() == 4 (default) 또는 == 8 (option b).
- 빌드 통과.

---

## 12. T11 — CodeViewer entity + impl Render

### 12.1 신규 파일

`crates/moai-studio-ui/src/viewer/code/mod.rs`:

```text
//! SPEC-V3-006 RG-MV-3, RG-MV-4, RG-MV-5: CodeViewer.

pub mod highlight;
pub mod languages;
pub mod gutter;
pub mod mx_scan;

use std::path::PathBuf;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, div};

pub struct CodeViewer {
    pub path: PathBuf,
    pub state: crate::viewer::ViewerState,  // shared
    pub grammar: Option<&'static languages::GrammarEntry>,
    pub tree: Option<tree_sitter::Tree>,
    pub mx_tags: Vec<mx_scan::MxTag>,
    pub diagnostics: Vec<crate::viewer::diagnostics::Diagnostic>,
    pub scroll: crate::viewer::scroll::VirtualScroll,
    pub lsp_client: Option<crate::lsp::LspClient>,
}

impl CodeViewer {
    pub fn open(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let grammar = path.extension().and_then(|e| e.to_str()).and_then(languages::for_extension);
        // async file read ... (T6 reuse)
        // grammar 결정 후 parser.parse(source, None) → tree
        // mx_scan(source) → mx_tags
        // grammar 가 있으면 LSP client spawn (T17)
        Self { /* ... */ }
    }
}

impl Render for CodeViewer {
    fn render(&mut self, _w: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.scroll.visible_range();
        div().flex().flex_row().size_full()
            .child(gutter::render_mx_gutter(&self.mx_tags, visible.clone()))
            .child(
                div().flex().flex_col().flex_grow_1()
                    .children(visible.map(|line_no| self.render_line(line_no)))
            )
    }
}
```

### 12.2 AC 매핑

- AC-MV-2: 토큰 색상 적용.
- AC-MV-12: Drop 시 LSP shutdown (T20).

---

## 13. T12 — tree-sitter highlight queries 통합

### 13.1 신규 파일

`crates/moai-studio-ui/src/viewer/code/highlight.rs`:

```text
//! SPEC-V3-006 RG-MV-3: tree-sitter capture → color 매핑.

use tree_sitter::{Tree, Node, QueryCursor};

pub struct HighlightSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub capture_name: &'static str,
}

pub fn highlight_visible_lines(
    tree: &Tree,
    source: &str,
    grammar: &super::languages::GrammarEntry,
    line_range: std::ops::Range<usize>,
) -> Vec<HighlightSpan> {
    let query = tree_sitter::Query::new(&(grammar.language)(), grammar.highlights_query).unwrap();
    let mut cursor = QueryCursor::new();
    // line_range 를 byte_range 로 변환 후 cursor.set_byte_range
    let mut spans = vec![];
    for m in cursor.matches(&query, tree.root_node(), source.as_bytes()) {
        for cap in m.captures {
            spans.push(HighlightSpan {
                start_byte: cap.node.start_byte(),
                end_byte: cap.node.end_byte(),
                capture_name: query.capture_names()[cap.index as usize].as_str(),
            });
        }
    }
    spans
}

pub fn capture_to_color(name: &str) -> u32 {
    match name {
        "function" | "function.method" => 0xdcdcaa,
        "string" => 0xce9178,
        "comment" => 0x6a9955,
        "keyword" | "keyword.control" => 0x569cd6,
        "type" | "type.builtin" => 0x4ec9b0,
        "variable" => 0x9cdcfe,
        "number" => 0xb5cea8,
        "operator" => 0xd4d4d4,
        _ => 0xd4d4d4,
    }
}
```

### 13.2 신규 단위 테스트

- `highlight::tests::rust_function_capture_returns_yellow`
- `highlight::tests::python_string_capture_returns_orange`
- `highlight::tests::typescript_keyword_capture_returns_blue`

### 13.3 AC 매핑

- AC-MV-2: highlight queries 통합 정확성.

---

## 14. T13 — Markdown fenced code block highlight (T12 reuse)

### 14.1 markdown/parser.rs 확장

```text
match event {
    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
        let code_lang = lang.to_string();
        if code_lang == "mermaid" {
            // T24 의 mermaid 처리
        } else if let Some(grammar) = super::super::code::languages::SUPPORTED_LANGUAGES.iter().find(|g| g.name == code_lang) {
            // T12 의 highlight reuse
        } else {
            // 기본 mono-font code block
        }
    }
    // ...
}
```

### 14.2 AC 매핑

- AC-MV-2 추가: 마크다운 본문의 ` ```rust ... ``` ` 블록도 동일 highlight.

---

## 15. T14 — USER-DECISION (c) 시 KaTeX/Mermaid mono-font fallback

### 15.1 fallback 동작

USER-DECISION 1 의 결과가 (c) 인 동안:
- `Event::DisplayMath(s)` → `code_block_element(s, "math")` (mono-font 박스).
- `Event::InlineMath(s)` → `inline_code_element(s)`.
- ` ```mermaid ``` ` 블록 → `code_block_element(body, "mermaid")` + uppermost banner.

### 15.2 배너 표시

```text
fn maybe_render_fallback_banner(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
    if !viewer.has_shown_fallback_banner && viewer.detected_math_or_mermaid {
        Some(div().bg(rgb(0xfff3cd)).p_2().child("Math/diagram render disabled (USER-DECISION c)"))
    } else { None }
}
```

### 15.3 AC 매핑

- AC-MV-7 (text fallback path): mono-font + 배너.

---

## 16. T15 — USER-DECISION 게이트 (3)

### 16.1 USER-DECISION 3: lsp-server-binary-discovery-v3-006

[USER-DECISION-REQUIRED: lsp-server-binary-discovery-v3-006]

질문:
- "LSP server binary 가 `$PATH` 에 없을 때 viewer 의 동작을 결정하시겠습니까?"

옵션:
- (i) **(권장)** Graceful degradation — syntax highlight 만 활성, 배너.
- (ii) Fail-fast — install 안내 popup.

Default: (i). 결과를 progress.md 기록.

### 16.2 AC 매핑

- OD-MV3.

---

## 17. T16 — async-lsp + lsp-types 의존성

### 17.1 Cargo.toml

```toml
async-lsp = "0.2"
lsp-types = "0.97"
```

### 17.2 빌드 검증

`cargo build -p moai-studio-ui` 통과 확인.

### 17.3 AC 매핑

- AC-MV-4 (선행): LSP client crate 통합.

---

## 18. T17 — LspClient 추상 + server_registry

### 18.1 신규 파일

`crates/moai-studio-ui/src/lsp/mod.rs`:

```text
//! SPEC-V3-006 RG-MV-4: LSP client (async-lsp wrapper).

pub mod server_registry;

use async_lsp::{LanguageServer, ServerSocket};
use lsp_types::{InitializeParams, PublishDiagnosticsParams};
use std::path::PathBuf;
use tokio::process::Child;

pub struct LspClient {
    pub server: ServerSocket,
    pub child: Child,
    pub language: &'static str,
    pub workspace_root: PathBuf,
}

impl LspClient {
    pub async fn spawn_for_language(lang: &'static str, workspace_root: PathBuf) -> Result<Self, LspSpawnError> {
        let cfg = server_registry::for_language(lang).ok_or(LspSpawnError::UnknownLanguage)?;
        let mut cmd = tokio::process::Command::new(&cfg.binary);
        cmd.args(&cfg.args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| LspSpawnError::BinaryNotFound { binary: cfg.binary.into(), source: e })?;
        // async-lsp 의 ClientBuilder 패턴
        // let (server, _io_task) = ClientBuilder::new().pipe_stdio(child.stdout, child.stdin);
        // initialize → initialized
        Ok(Self { /* ... */ })
    }

    pub async fn shutdown(mut self) -> Result<(), LspShutdownError> {
        self.server.shutdown().await?;
        self.server.exit().await?;
        self.child.kill().await?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LspSpawnError {
    #[error("LSP binary not found: {binary}")]
    BinaryNotFound { binary: String, #[source] source: std::io::Error },
    #[error("unknown language: {0}")]
    UnknownLanguage,
    // ...
}
```

### 18.2 server_registry.rs

```text
pub struct ServerConfig {
    pub binary: &'static str,
    pub args: &'static [&'static str],
}

pub fn for_language(lang: &str) -> Option<ServerConfig> {
    match lang {
        "rust" => Some(ServerConfig { binary: "rust-analyzer", args: &[] }),
        "go" => Some(ServerConfig { binary: "gopls", args: &["serve"] }),
        "python" => Some(ServerConfig { binary: "pyright-langserver", args: &["--stdio"] }),
        "typescript" => Some(ServerConfig { binary: "typescript-language-server", args: &["--stdio"] }),
        _ => None,
    }
}
```

### 18.3 AC 매핑

- AC-MV-4 (선행): LSP server spawn.
- AC-MV-5: spawn 실패 시 LspSpawnError::BinaryNotFound.

---

## 19. T18 — publishDiagnostics 처리 + 진단 cache

### 19.1 신규 파일

`crates/moai-studio-ui/src/viewer/diagnostics.rs`:

```text
//! SPEC-V3-006 RG-MV-4: 진단 cache + squiggly render.

use lsp_types::{Diagnostic as LspDiagnostic, DiagnosticSeverity};

pub struct Diagnostic {
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub severity: Severity,
    pub message: String,
    pub source: Option<String>,
}

pub enum Severity { Error, Warning, Information, Hint }

impl From<LspDiagnostic> for Diagnostic {
    fn from(d: LspDiagnostic) -> Self {
        Self {
            line: d.range.start.line as usize,
            col_start: d.range.start.character as usize,
            col_end: d.range.end.character as usize,
            severity: match d.severity {
                Some(DiagnosticSeverity::ERROR) => Severity::Error,
                Some(DiagnosticSeverity::WARNING) => Severity::Warning,
                Some(DiagnosticSeverity::INFORMATION) => Severity::Information,
                Some(DiagnosticSeverity::HINT) | None => Severity::Hint,
                _ => Severity::Hint,
            },
            message: d.message,
            source: d.source,
        }
    }
}

pub fn severity_color(s: &Severity) -> u32 {
    match s {
        Severity::Error => 0xff5555,
        Severity::Warning => 0xff8c1a,
        Severity::Information => 0x4080d0,
        Severity::Hint => 0x888888,
    }
}
```

### 19.2 AC 매핑

- AC-MV-4: publishDiagnostics 처리.

---

## 20. T19 — squiggly underline render + hover tooltip

### 20.1 viewer/code/mod.rs::render_line 확장

```text
fn render_line(&self, line_no: usize) -> impl IntoElement {
    let line_text = self.source_line(line_no);
    let highlights = highlight::highlight_for_line(&self.tree, &line_text, line_no);
    let diags = self.diagnostics.iter().filter(|d| d.line == line_no);
    let mut row = div().flex().flex_row();
    for (text_part, color) in build_colored_runs(&line_text, &highlights) {
        let mut span = div().child(text_part).text_color(rgb(color));
        // 진단 위치에 squiggly underline
        if let Some(d) = diags.find(...) {
            span = span
                .border_b_2()
                .border_color(rgb(diagnostics::severity_color(&d.severity)))
                .border_dashed()  // GPUI wavy 부재 시 dashed fallback
                .hover(|s| s.tooltip(...));  // tooltip 표시
        }
        row = row.child(span);
    }
    row
}
```

### 20.2 AC 매핑

- AC-MV-4: squiggly underline + hover tooltip.

---

## 21. T20 — LSP server lifecycle (shutdown on drop)

### 21.1 Drop impl

```text
impl Drop for CodeViewer {
    fn drop(&mut self) {
        if let Some(client) = self.lsp_client.take() {
            tokio::spawn(async move {
                let _ = client.shutdown().await;
            });
        }
        // tree-sitter 파서는 별도 처리 — Tree 가 Drop 시 메모리 해제
    }
}
```

### 21.2 AC 매핑

- AC-MV-12: Drop 시 LSP server 자식 프로세스 종료, zombie 없음.

---

## 22. T21 — mx_scan 함수 + MxTag struct

### 22.1 신규 파일

`crates/moai-studio-ui/src/viewer/code/mx_scan.rs`:

```text
//! SPEC-V3-006 RG-MV-5: @MX 태그 스캔.

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MxKind { Anchor, Warn, Note, Todo }

#[derive(Debug, Clone)]
pub struct MxTag {
    pub line: usize,         // 1-based
    pub kind: MxKind,
    pub body: String,
    pub reason: Option<String>,
    pub fan_in: Option<usize>,  // None = "N/A" (정적 분석 미지원)
    pub spec_id: Option<String>,
}

pub fn scan_mx_tags(source: &str) -> Vec<MxTag> {
    static RE_TAG: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(||
        Regex::new(r"@MX:(NOTE|WARN|ANCHOR|TODO)\s*(.*)").unwrap()
    );
    static RE_SPEC: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(||
        Regex::new(r"\bSPEC-[A-Z0-9]+-\d+\b").unwrap()
    );
    static RE_REASON: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(||
        Regex::new(r"@MX:REASON\s*:?\s*(.*)").unwrap()
    );

    let lines: Vec<&str> = source.lines().collect();
    let mut tags = vec![];
    for (idx, line) in lines.iter().enumerate() {
        if let Some(caps) = RE_TAG.captures(line) {
            let kind = match &caps[1] {
                "ANCHOR" => MxKind::Anchor,
                "WARN" => MxKind::Warn,
                "NOTE" => MxKind::Note,
                "TODO" => MxKind::Todo,
                _ => continue,
            };
            let body = caps[2].to_string();
            let spec_id = RE_SPEC.captures(&body).map(|c| c[0].to_string());
            // REASON sub-line 검사 (다음 1-2 라인)
            let reason = lines.iter().skip(idx + 1).take(2)
                .find_map(|l| RE_REASON.captures(l).map(|c| c[1].to_string()));
            tags.push(MxTag {
                line: idx + 1,
                kind,
                body,
                reason,
                fan_in: None,  // v1.0.0 = N/A
                spec_id,
            });
        }
    }
    tags
}
```

### 22.2 신규 단위 테스트

- `mx_scan::tests::scan_anchor_tag_extracts_body`
- `mx_scan::tests::scan_warn_with_reason_subline_attaches_reason`
- `mx_scan::tests::scan_warn_without_reason_returns_none`
- `mx_scan::tests::scan_anchor_with_spec_id_extracts_spec`
- `mx_scan::tests::scan_no_tags_returns_empty`

### 22.3 AC 매핑

- AC-MV-6: scan 정확성.

---

## 23. T22 — @MX gutter element + popover

### 23.1 신규 파일

`crates/moai-studio-ui/src/viewer/code/gutter.rs`:

```text
//! SPEC-V3-006 RG-MV-5: @MX gutter rendering.

use gpui::{IntoElement, ParentElement, Styled, div, rgb, px};

pub fn render_mx_gutter(tags: &[super::mx_scan::MxTag], visible_range: std::ops::Range<usize>) -> impl IntoElement {
    let mut col = div().flex().flex_col().w(px(20.0)).bg(rgb(0x1e1e1e));
    for line_no in visible_range {
        let tag = tags.iter().find(|t| t.line == line_no);
        col = col.child(render_gutter_cell(line_no, tag));
    }
    col
}

fn render_gutter_cell(line_no: usize, tag: Option<&super::mx_scan::MxTag>) -> impl IntoElement {
    use super::mx_scan::MxKind;
    let icon_data = match tag.map(|t| &t.kind) {
        Some(MxKind::Anchor) => Some(("★", 0xd4a017)),
        Some(MxKind::Warn) => Some(("⚠", 0xff8c1a)),
        Some(MxKind::Note) => Some(("ℹ", 0x4080d0)),
        Some(MxKind::Todo) => Some(("☐", 0x888888)),
        None => None,
    };
    let mut cell = div().h(px(18.0)).w_full();
    if let Some((icon, color)) = icon_data {
        cell = cell.child(icon).text_color(rgb(color));
        // WARN 의 REASON 누락 시 outline 시각 강조 (REQ-MV-055)
        if let Some(t) = tag {
            if t.kind == MxKind::Warn && t.reason.is_none() {
                cell = cell.border_1().border_color(rgb(0xff8c1a));
            }
        }
        // 클릭 → popover (Spike 3 결과 따라 GPUI popup API 또는 inline expand)
    }
    cell
}
```

### 23.2 AC 매핑

- AC-MV-6: 거터 아이콘 + 색상 + popover.

---

## 24. T23 — SPEC ID link → OpenFileEvent

### 24.1 markdown/parser.rs 확장

```text
match event {
    Event::Start(Tag::Link(_, dest, _)) => {
        let dest_str = dest.to_string();
        if let Some(spec_id) = SPEC_PATTERN.captures(&dest_str).map(|c| c[0].to_string()) {
            let spec_path = format!(".moai/specs/{}/spec.md", spec_id);
            // 클릭 핸들러: cx.listener(|view, _, _, cx| {
            //     view.emit_open_file_event(OpenFileEvent { path: spec_path.into(), hint: Some(Markdown) }, cx);
            // })
        }
    }
    // ...
}
```

### 24.2 AC 매핑

- AC-MV-10: SPEC ID link 클릭 → OpenFileEvent.

---

## 25. T24 — KaTeX/Mermaid 렌더 (USER-DECISION 결과 분기)

### 25.1 USER-DECISION (a) 채택 시 — WebView

`crates/moai-studio-ui/src/viewer/markdown/katex.rs` (신규, `feature = "katex-mermaid-webview"`):

```text
//! SPEC-V3-006 RG-MV-2: KaTeX 수식 렌더 (WebView).

use wry::WebView;

pub fn render_math(latex: &str) -> impl gpui::IntoElement {
    // WebView 인스턴스에 KaTeX 의 server-side render 또는 client-side eval
    // 결과 SVG 를 GPUI 의 inline svg element 로 마운트
    /* ... wry 통합 ... */
}
```

### 25.2 USER-DECISION (c) 채택 시 — fallback

T14 의 mono-font 처리 그대로 유지.

### 25.3 AC 매핑

- AC-MV-7: USER-DECISION 결과별 검증.

---

## 26. T25 — 가상 스크롤 GPUI 통합 + 100MB bench

### 26.1 GPUI 통합

CodeViewer / MarkdownViewer 의 `impl Render` 가 `visible_range` 만 element 로 마운트.

### 26.2 신규 bench

`crates/moai-studio-ui/benches/viewer_scroll.rs`:

```text
//! SPEC-V3-006 NFR-MV-2/3: 100MB 파일의 viewer 첫 paint + 스크롤 fps.

use criterion::{criterion_group, criterion_main, Criterion, BatchSize};

fn bench_viewer_first_paint(c: &mut Criterion) {
    c.bench_function("100mb_first_paint", |b| {
        b.iter_batched(
            || generate_100mb_text(),
            |source| {
                // CodeViewer 인스턴스 생성 + 첫 visible_range 마운트
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_viewer_scroll(c: &mut Criterion) {
    c.bench_function("100mb_scroll_50_pages", |b| {
        // viewport_top_px += line_height_px * 50; cx.notify; loop 50 회
    });
}
```

### 26.3 AC 매핑

- AC-MV-8: 첫 paint ≤ 200ms, 60fps 스크롤.

---

## 27. T26 — LSP graceful degradation 배너

### 27.1 viewer/code/mod.rs 확장

```text
pub struct CodeViewer {
    // ...
    pub lsp_status: LspStatus,
}

pub enum LspStatus {
    NotStarted,
    Initializing,
    Ready,
    Unavailable { reason: String },
}

// CodeViewer::open 의 LSP spawn 분기:
match crate::lsp::LspClient::spawn_for_language(grammar.name, ws_root).await {
    Ok(client) => { self.lsp_client = Some(client); self.lsp_status = LspStatus::Ready; }
    Err(LspSpawnError::BinaryNotFound { binary, .. }) => {
        self.lsp_status = LspStatus::Unavailable { reason: format!("LSP unavailable: {}", binary) };
        tracing::warn!(binary, "LSP server not in PATH; falling back to syntax-highlight only");
    }
    Err(e) => { self.lsp_status = LspStatus::Unavailable { reason: e.to_string() }; }
}
```

### 27.2 배너 element

```text
fn render_lsp_status_banner(&self) -> Option<impl IntoElement> {
    match &self.lsp_status {
        LspStatus::Unavailable { reason } => Some(div().bg(rgb(0xfff3cd)).p_2().child(reason.clone())),
        _ => None,
    }
}
```

### 27.3 AC 매핑

- AC-MV-5: graceful degradation + 배너.

---

## 28. T27 — Regression + Smoke + Commit

### 28.1 Regression 검증 명령

```
cargo test -p moai-studio-terminal --all-targets
cargo test -p moai-studio-workspace --all-targets
cargo test -p moai-studio-ui --lib panes::
cargo test -p moai-studio-ui --lib tabs::
cargo test -p moai-studio-ui --lib terminal::
cargo test -p moai-studio-ui --lib viewer::         # 신규
cargo test -p moai-studio-ui --lib lsp::            # 신규
cargo test -p moai-studio-ui --test integration_render   # SPEC-V3-004 carry
cargo test -p moai-studio-ui --test integration_viewer   # 신규
cargo bench -p moai-studio-ui --bench viewer_scroll      # NFR-MV-2/3
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

### 28.2 Smoke test

`cargo run -p moai-studio-app` 실행 후 spec.md §1.5 의 7 가지 사용자 가시 동작 수동 확인.

### 28.3 LSP 환경 검증

- macOS / Linux 환경에서 `rust-analyzer`, `gopls` 가 `$PATH` 에 있는 환경 + 없는 환경 두 케이스 모두 검증 (AC-MV-4, AC-MV-5).

### 28.4 Commit 전략

본 SPEC 의 plan 단계 산출 (research/plan/spec) 은 단일 commit (사용자 지시):
- `docs(spec): SPEC-V3-006 Markdown/Code Viewer v1.0.0 (research/plan/spec)`

implementation 단계 (T1~T26) 는 별도 SPEC run 단계에서 milestone 별 commit:
- MS-1 commit: T1-T8 (LeafKind + RootView + MarkdownViewer + VirtualScroll + binary detection)
- MS-2 commit: T9-T14 (USER-DECISION + tree-sitter + CodeViewer + highlight + markdown highlight 통합)
- MS-3 commit: T15-T26 (USER-DECISION + LSP + 진단 + @MX gutter + KaTeX/Mermaid + 가상 스크롤 + bench)
- T27 commit: regression + smoke + progress.md final.

---

## 29. Hard Thresholds (sprint exit 전제, SPEC-V3-004 carry)

- [ ] Coverage ≥ 85% per commit (viewer 모듈 목표)
- [ ] LSP `max_errors: 0`, `max_type_errors: 0`, `max_lint_errors: 0`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warning
- [ ] `cargo fmt --all -- --check` 통과
- [ ] SPEC-V3-002 regression 0 (13/13)
- [ ] SPEC-V3-003 logic tests regression 0
- [ ] SPEC-V3-004 render tests regression 0
- [ ] 신규 viewer tests ≥ 25 unit + 3 integration
- [ ] @MX 태그: ANCHOR ≥ 2 신규 (leaf-kind-surface-dispatch, render-pane-tree-recursion 은 SPEC-V3-004 에 이미), WARN ≥ 0, NOTE ≥ 3
- [ ] LSP zombie process 0 (AC-MV-12)
- [ ] 100MB 파일 첫 paint ≤ 200 ms (AC-MV-8)
- [ ] 60 fps 스크롤 (AC-MV-8)

---

## 30. Escalation Protocol

- USER-DECISION 게이트 1 (KaTeX/Mermaid) 가 (b) Native 채택 시 → 스코프 분리 escalation. Native KaTeX port 부재로 본 SPEC 의 MS-3 가 차단될 가능성 → AskUserQuestion 으로 (a) WebView 또는 (c) 영구 fallback 재선택.
- tree-sitter Rust binding 의 cargo build 시간이 CI 영향을 크게 미칠 시 (Spike 2 결과) → grammar 번들 lazy load 또는 build feature flag 도입.
- async-lsp 의 GPUI tokio runtime 통합 충돌 시 → 별도 thread pool 또는 GPUI 의 background task API 활용.
- LSP server 의 진단 latency 가 RG-MV-4 의 NFR 한계 초과 시 → 진단 throttle / debounce 도입.
- 가상 스크롤 fps 가 NFR-MV-3 미달 시 (Spike 4 결과) → element pooling / texture cache 도입.
- SPEC-V3-005 의 OpenFileEvent canonical 정의가 본 SPEC 가정과 크게 다를 시 → spec.md OD-MV-? 갱신 + 본 SPEC plan revision.
- AC-MV-12 (LSP zombie) 검증 실패 시 → process group kill / SIGKILL fallback 추가.

---

## 31. Sprint Exit Criteria (SPEC-V3-006 → 종결 gate)

- AC-MV-1 ~ AC-MV-12 전원 GREEN.
- 3 USER-DECISION 게이트 결과가 progress.md 에 기록.
- Hard thresholds 전원 통과.
- SPEC-V3-002 / V3-003 / V3-004 regression 0.
- SPEC-V3-005 와 합의된 e2e 검증 시점 PASS (단, SPEC-V3-005 미완 시점에는 mock event 통합으로 대체 + progress.md 명시).
- T27 commit 완료, progress.md SPEC complete 기록 (별도 progress.md 는 implementation 단계 산출, 본 plan 단계에서는 미생성).

---

## 32. SPEC-V3-005 와의 인터페이스 동기화

본 SPEC 은 SPEC-V3-005 의 OpenFileEvent 정의에 의존한다. 양 SPEC 의 plan 단계에서 다음을 합의:

| 합의 항목 | 본 SPEC 책임 | SPEC-V3-005 책임 |
|----------|--------------|------------------|
| OpenFileEvent struct 정의 | (consumer) | canonical 정의 — `crates/moai-studio-workspace/src/file_explorer/event.rs` 또는 동등 |
| `path: PathBuf` 필드 | import | 정의 |
| `surface_hint: Option<SurfaceHint>` 필드 | import | 정의 |
| `SurfaceHint` enum (Markdown/Code/Terminal) | import | 정의 |
| 발행 시점 (file double-click) | (consumer) | 발행 |
| 수신 시점 (RootView::handle_open_file) | 수신 | (no responsibility) |

본 SPEC 의 MS-1 시점에 SPEC-V3-005 가 미완이면 mock 으로 시작. SPEC-V3-005 의 plan 단계에서 OpenFileEvent 정의가 published 되면 본 SPEC implementation 의 첫 commit 에 SPEC-V3-005 의존 import 로 교체.

---

작성: 2026-04-25
브랜치 (plan 산출): `feature/SPEC-V3-004-render`
다음: implementation phase (`/moai run SPEC-V3-006`) — 별도 브랜치 `feature/SPEC-V3-006-viewer`
