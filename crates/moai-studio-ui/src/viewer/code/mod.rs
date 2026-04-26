//! SPEC-V3-006 MS-2 T11: CodeViewer GPUI entity.
//!
//! `CodeViewer` 는 tree-sitter syntax highlight 로 소스 코드를 렌더하는 GPUI Entity 다.
//! `LeafKind::Code(entity)` 로 pane leaf 에 마운트된다.
//!
//! @MX:TODO(MS-3-lsp-diagnostic): LSP 진단 (squiggly underline + hover tooltip) 은
//!   MS-3 에서 async-lsp + lsp-types 통합 시 추가된다 (RG-MV-4).

pub mod highlight;
pub mod languages;

use crate::design::tokens::{self as tok, semantic};
use crate::viewer::scroll::VirtualScroll;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use highlight::{HighlightedLine, HighlightedSpan, highlight_source, scope_to_color};
use languages::SupportedLang;
use std::path::PathBuf;

// ============================================================
// CodeViewer
// ============================================================

/// tree-sitter syntax highlight 코드 뷰어 GPUI Entity (REQ-MV-020).
///
/// 상태 전이: Loading → Ready { lines } | Error(e).
pub struct CodeViewer {
    /// 열린 파일 경로
    pub path: PathBuf,
    /// 뷰어 로드 상태
    pub state: CodeViewerState,
    /// 감지된 언어 (None = 미지원 확장자)
    pub lang: Option<SupportedLang>,
    /// 원본 소스 코드
    pub raw_code: String,
    /// tree-sitter highlight 결과
    pub highlighted: Vec<HighlightedLine>,
    /// 스크롤 상태
    pub scroll: VirtualScroll,
}

/// CodeViewer 의 로드 상태.
#[derive(Debug)]
pub enum CodeViewerState {
    Loading,
    Ready,
    Error(String),
}

impl CodeViewer {
    /// 주어진 경로로 CodeViewer 를 생성한다.
    ///
    /// 파일 확장자로 언어를 감지하고 초기 상태는 `Loading` 이다.
    pub fn new(path: PathBuf) -> Self {
        let lang = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(languages::detect_lang_from_extension);

        Self {
            path,
            state: CodeViewerState::Loading,
            lang,
            raw_code: String::new(),
            highlighted: Vec::new(),
            scroll: VirtualScroll::default(),
        }
    }

    /// 소스 코드를 로드하고 tree-sitter highlight 파이프라인을 실행한다.
    ///
    /// `lang` 이 Some 인 경우 highlight_source 를 호출한다.
    /// None 이면 plain text 로 처리한다.
    pub fn load(&mut self, code: String, cx: &mut Context<Self>) {
        self.highlighted = if let Some(lang) = self.lang {
            highlight_source(&code, lang)
        } else {
            // 미지원 언어: plain text fallback
            code.lines()
                .map(|line| HighlightedLine {
                    spans: vec![HighlightedSpan {
                        text: line.to_string(),
                        scope: None,
                    }],
                })
                .collect()
        };
        self.scroll.line_count = self.highlighted.len();
        self.raw_code = code;
        self.state = CodeViewerState::Ready;
        cx.notify();
    }

    /// 에러 상태로 전이한다.
    pub fn set_error(&mut self, msg: String, cx: &mut Context<Self>) {
        self.state = CodeViewerState::Error(msg);
        cx.notify();
    }

    /// 현재 상태가 Ready 인지 확인한다.
    pub fn is_ready(&self) -> bool {
        matches!(self.state, CodeViewerState::Ready)
    }

    /// 현재 상태가 Loading 인지 확인한다.
    pub fn is_loading(&self) -> bool {
        matches!(self.state, CodeViewerState::Loading)
    }

    /// 현재 상태가 Error 인지 확인한다.
    pub fn is_error(&self) -> bool {
        matches!(self.state, CodeViewerState::Error(_))
    }
}

// ============================================================
// impl Render for CodeViewer
// ============================================================

impl Render for CodeViewer {
    /// 상태에 따라 spinner / 코드 / 에러를 렌더한다.
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.state {
            CodeViewerState::Loading => render_loading().into_any_element(),
            CodeViewerState::Error(msg) => render_error(msg).into_any_element(),
            CodeViewerState::Ready => {
                let visible = self.scroll.visible_range();
                render_highlighted_lines(&self.highlighted, visible).into_any_element()
            }
        }
    }
}

// ============================================================
// 렌더 헬퍼
// ============================================================

fn render_loading() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .justify_center()
        .items_center()
        .bg(rgb(tok::BG_APP))
        .child(div().text_color(rgb(tok::FG_MUTED)).child("로딩 중..."))
}

fn render_error(msg: &str) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .justify_center()
        .items_center()
        .bg(rgb(tok::BG_APP))
        .child(
            div()
                .p(px(16.))
                .bg(rgb(tok::BG_SURFACE))
                .rounded_md()
                .text_color(rgb(semantic::DANGER))
                .child(format!("오류: {}", msg)),
        )
}

fn render_highlighted_lines(
    lines: &[HighlightedLine],
    visible: std::ops::Range<usize>,
) -> impl IntoElement {
    let mut container = div()
        .flex()
        .flex_col()
        .size_full()
        .p(px(8.))
        .bg(rgb(tok::BG_PANEL));

    let end = visible.end.min(lines.len());
    for line in &lines[visible.start..end] {
        let mut row = div().flex().flex_row().text_size(px(12.));
        for span in &line.spans {
            let color = span
                .scope
                .as_ref()
                .map(scope_to_color)
                .map(|[r, g, b]| (r as u32) << 16 | (g as u32) << 8 | b as u32)
                .unwrap_or(tok::FG_SECONDARY); // 기본 텍스트 색상
            row = row.child(div().text_color(rgb(color)).child(span.text.clone()));
        }
        container = container.child(row);
    }

    container
}

// ============================================================
// 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    use std::path::PathBuf;

    #[test]
    fn code_viewer_initial_state_is_loading() {
        let viewer = CodeViewer::new(PathBuf::from("/tmp/main.rs"));
        assert!(viewer.is_loading());
        assert!(!viewer.is_ready());
        assert!(!viewer.is_error());
    }

    #[test]
    fn code_viewer_detects_rust_from_path() {
        let viewer = CodeViewer::new(PathBuf::from("/tmp/main.rs"));
        assert_eq!(viewer.lang, Some(SupportedLang::Rust));
    }

    #[test]
    fn code_viewer_detects_typescript_from_path() {
        let viewer = CodeViewer::new(PathBuf::from("/tmp/app.ts"));
        assert_eq!(viewer.lang, Some(SupportedLang::TypeScript));
    }

    #[test]
    fn code_viewer_unknown_extension_has_no_lang() {
        let viewer = CodeViewer::new(PathBuf::from("/tmp/file.xml"));
        assert_eq!(viewer.lang, None);
    }

    #[test]
    fn code_viewer_load_transitions_to_ready() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| CodeViewer::new(PathBuf::from("/tmp/main.rs")));

        cx.update(|app| {
            entity.update(app, |viewer: &mut CodeViewer, cx| {
                viewer.load("fn main() {}".to_string(), cx);
            });
        });

        let is_ready = cx.read(|app| entity.read(app).is_ready());
        assert!(is_ready, "load 후 상태는 Ready 여야 한다");
    }

    #[test]
    fn code_viewer_highlights_rust_keywords() {
        // AC-MV-2: Rust 코드에서 fn 키워드가 highlight 되어야 한다
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| CodeViewer::new(PathBuf::from("/tmp/main.rs")));

        cx.update(|app| {
            entity.update(app, |viewer: &mut CodeViewer, cx| {
                viewer.load("fn main() {}".to_string(), cx);
            });
        });

        let has_keyword_highlight = cx.read(|app| {
            let v = entity.read(app);
            v.highlighted.iter().any(|line| {
                line.spans
                    .iter()
                    .any(|s| s.scope == Some(highlight::HighlightScope::Keyword))
            })
        });
        assert!(
            has_keyword_highlight,
            "fn 이 Keyword scope 로 highlight 되어야 한다"
        );
    }
}
