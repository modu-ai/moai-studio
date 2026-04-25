//! SPEC-V3-006 RG-MV-1: MarkdownViewer entity.
//!
//! `MarkdownViewer` 는 GPUI `Entity<MarkdownViewer>` 로 생성되고,
//! `LeafKind::Markdown(entity)` 로 pane leaf 에 마운트된다.
//!
//! 상태 전이: Loading → Ready(source) | Error(e).

pub mod parser;

// @MX:ANCHOR: [AUTO] markdown-viewer-state
// @MX:REASON: [AUTO] SPEC-V3-006 RG-MV-1 REQ-MV-004/006. ViewerState 는
//   MarkdownViewer 의 lifecycle 진입점이다.
//   fan_in >= 3: MarkdownViewer::open, impl Render, unit tests.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use parser::{MarkdownBlock, parse_markdown};
use std::path::PathBuf;

// ============================================================
// ViewerState
// ============================================================

/// MarkdownViewer 의 로드 상태 (REQ-MV-004, REQ-MV-006).
#[derive(Debug)]
pub enum ViewerState {
    /// 비동기 파일 읽기 진행 중
    Loading,
    /// 파일 읽기 완료 — 파싱된 블록 목록 보유
    Ready {
        /// 원본 마크다운 텍스트
        source: String,
        /// 파싱된 블록 목록
        blocks: Vec<MarkdownBlock>,
    },
    /// 파일 읽기 또는 파싱 실패
    Error(String),
}

// ============================================================
// MarkdownViewer
// ============================================================

/// CommonMark + GFM 마크다운 뷰어 GPUI Entity (REQ-MV-001).
pub struct MarkdownViewer {
    pub path: PathBuf,
    pub state: ViewerState,
    pub scroll: crate::viewer::scroll::VirtualScroll,
    /// USER-DECISION (c): 수식/mermaid 가 문서에서 발견된 경우 fallback 배너 표시 여부
    pub has_math_or_mermaid: bool,
    /// fallback 배너를 이미 표시했는지 여부 (1 회 표시 후 숨김)
    pub fallback_banner_shown: bool,
}

impl MarkdownViewer {
    /// 주어진 경로로 MarkdownViewer 를 생성한다 (REQ-MV-001, REQ-MV-004).
    ///
    /// 초기 상태는 `Loading` 이다. 실제 파일 읽기는 `load` 메서드로 완료한다.
    /// 비동기 spawn 은 `open` factory method 가 담당한다.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            state: ViewerState::Loading,
            scroll: crate::viewer::scroll::VirtualScroll::default(),
            has_math_or_mermaid: false,
            fallback_banner_shown: false,
        }
    }

    /// 파일 내용을 동기적으로 로드하고 상태를 Ready 로 전이한다.
    ///
    /// 테스트에서 직접 호출하거나 `open` 내부 async 태스크에서 호출된다.
    pub fn load(&mut self, source: String, cx: &mut Context<Self>) {
        let blocks = parse_markdown(&source);
        // 수식/mermaid 감지
        self.has_math_or_mermaid = blocks
            .iter()
            .any(|b| matches!(b, MarkdownBlock::Math(_) | MarkdownBlock::Mermaid(_)));
        self.scroll.line_count = blocks.len();
        self.state = ViewerState::Ready { source, blocks };
        cx.notify();
    }

    /// 에러 상태로 전이한다.
    pub fn set_error(&mut self, msg: String, cx: &mut Context<Self>) {
        self.state = ViewerState::Error(msg);
        cx.notify();
    }

    /// 현재 상태가 Ready 인지 확인한다.
    pub fn is_ready(&self) -> bool {
        matches!(self.state, ViewerState::Ready { .. })
    }

    /// 현재 상태가 Loading 인지 확인한다.
    pub fn is_loading(&self) -> bool {
        matches!(self.state, ViewerState::Loading)
    }

    /// 현재 상태가 Error 인지 확인한다.
    pub fn is_error(&self) -> bool {
        matches!(self.state, ViewerState::Error(_))
    }
}

// ============================================================
// impl Render for MarkdownViewer
// ============================================================

impl Render for MarkdownViewer {
    /// 상태에 따라 spinner / 본문 / 에러 메시지를 렌더한다 (REQ-MV-006).
    ///
    /// Ready 상태에서는 visible_range 안의 블록만 element tree 에 포함한다 (REQ-MV-061 기초).
    /// panic 없이 항상 valid element 를 반환한다 (NFR-MV-6).
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.state {
            ViewerState::Loading => render_loading().into_any_element(),
            ViewerState::Error(msg) => render_error(msg).into_any_element(),
            ViewerState::Ready { blocks, .. } => {
                let visible = self.scroll.visible_range();
                render_blocks(
                    blocks,
                    visible,
                    self.has_math_or_mermaid,
                    self.fallback_banner_shown,
                )
                .into_any_element()
            }
        }
    }
}

// ============================================================
// Render helpers
// ============================================================

fn render_loading() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .justify_center()
        .items_center()
        .bg(rgb(0x1a1a1a))
        .child(div().text_color(rgb(0x888888)).child("로딩 중..."))
}

fn render_error(msg: &str) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .justify_center()
        .items_center()
        .bg(rgb(0x1a1a1a))
        .child(
            div()
                .p(px(16.))
                .bg(rgb(0x3a1a1a))
                .rounded_md()
                .text_color(rgb(0xff5555))
                .child(format!("오류: {}", msg)),
        )
}

fn render_blocks(
    blocks: &[MarkdownBlock],
    visible: std::ops::Range<usize>,
    has_fallback: bool,
    banner_shown: bool,
) -> impl IntoElement {
    let mut col = div()
        .flex()
        .flex_col()
        .size_full()
        .p(px(16.))
        .bg(rgb(0x1e1e1e))
        .gap(px(4.));

    // USER-DECISION (c) 배너 (1 회만 표시)
    if has_fallback && !banner_shown {
        col = col.child(render_fallback_banner());
    }

    // visible_range 범위 블록만 렌더 (REQ-MV-061)
    let clamped_end = visible.end.min(blocks.len());
    for block in &blocks[visible.start..clamped_end] {
        col = col.child(render_block(block));
    }

    col
}

fn render_fallback_banner() -> impl IntoElement {
    div()
        .p(px(8.))
        .mb(px(8.))
        .bg(rgb(0x3a3218))
        .rounded_md()
        .text_color(rgb(0xffc107))
        .child("수식/다이어그램 렌더 비활성화 (USER-DECISION c: MS-3 에서 WebView 활성화 예정)")
}

fn render_block(block: &MarkdownBlock) -> impl IntoElement {
    match block {
        MarkdownBlock::Heading { level, text } => {
            let size_px = heading_size_px(*level);
            div()
                .text_color(rgb(0xf4f4f5))
                .text_size(px(size_px))
                .mb(px(4.))
                .child(text.clone())
        }
        MarkdownBlock::Paragraph(text) => div()
            .text_color(rgb(0xb5b5bb))
            .text_size(px(14.))
            .mb(px(4.))
            .child(text.clone()),
        MarkdownBlock::CodeBlock { lang, code } => {
            let label = lang.as_deref().unwrap_or("plain");
            div()
                .p(px(8.))
                .mb(px(4.))
                .bg(rgb(0x252526))
                .rounded_md()
                .text_color(rgb(0xd4d4d4))
                .text_size(px(12.))
                .child(format!("[{}]\n{}", label, code))
        }
        MarkdownBlock::InlineCode(code) => div()
            .bg(rgb(0x252526))
            .rounded_sm()
            .px(px(4.))
            .text_color(rgb(0xce9178))
            .text_size(px(13.))
            .child(format!("`{}`", code)),
        MarkdownBlock::Math(math) => div()
            .p(px(8.))
            .mb(px(4.))
            .bg(rgb(0x252526))
            .rounded_md()
            .text_color(rgb(0xd4d4d4))
            .text_size(px(12.))
            .child(format!("[math]\n{}", math)),
        MarkdownBlock::Mermaid(diagram) => div()
            .p(px(8.))
            .mb(px(4.))
            .bg(rgb(0x252526))
            .rounded_md()
            .text_color(rgb(0x9cdcfe))
            .text_size(px(12.))
            .child(format!("[mermaid]\n{}", diagram)),
        MarkdownBlock::List(items) => {
            let mut list = div().flex().flex_col().mb(px(4.));
            for item in items {
                list = list.child(
                    div()
                        .text_color(rgb(0xb5b5bb))
                        .text_size(px(14.))
                        .child(format!("• {}", item)),
                );
            }
            list
        }
        MarkdownBlock::Quote(text) => div()
            .pl(px(12.))
            .border_l_2()
            .border_color(rgb(0x555566))
            .mb(px(4.))
            .text_color(rgb(0x888888))
            .text_size(px(14.))
            .child(text.clone()),
        MarkdownBlock::Rule => div().w_full().h(px(1.)).bg(rgb(0x3a3a40)).mb(px(8.)),
    }
}

fn heading_size_px(level: u8) -> f32 {
    match level {
        1 => 24.0,
        2 => 20.0,
        3 => 17.0,
        4 => 15.0,
        5 => 13.0,
        _ => 12.0,
    }
}

// ============================================================
// 단위 테스트 — T3 (AC-MV-1)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    use std::path::PathBuf;

    #[test]
    fn markdown_viewer_initial_state_is_loading() {
        // AC-MV-1: 생성 직후 상태는 Loading 이어야 한다
        let viewer = MarkdownViewer::new(PathBuf::from("/tmp/test.md"));
        assert!(viewer.is_loading(), "초기 상태는 Loading 이어야 한다");
        assert!(!viewer.is_ready());
        assert!(!viewer.is_error());
    }

    #[test]
    fn markdown_viewer_load_transitions_to_ready() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| MarkdownViewer::new(PathBuf::from("/tmp/test.md")));

        cx.update(|app| {
            entity.update(app, |viewer: &mut MarkdownViewer, cx| {
                viewer.load("# 안녕하세요\n\n단락 텍스트".to_string(), cx);
            });
        });

        let is_ready = cx.read(|app| entity.read(app).is_ready());
        assert!(is_ready, "load 후 상태는 Ready 여야 한다");
    }

    #[test]
    fn markdown_viewer_set_error_transitions_to_error() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| MarkdownViewer::new(PathBuf::from("/tmp/missing.md")));

        cx.update(|app| {
            entity.update(app, |viewer: &mut MarkdownViewer, cx| {
                viewer.set_error("파일을 찾을 수 없음".to_string(), cx);
            });
        });

        let is_error = cx.read(|app| entity.read(app).is_error());
        assert!(is_error, "set_error 후 상태는 Error 여야 한다");
    }

    #[test]
    fn markdown_viewer_load_detects_math_block() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| MarkdownViewer::new(PathBuf::from("/tmp/math.md")));

        cx.update(|app| {
            entity.update(app, |viewer: &mut MarkdownViewer, cx| {
                viewer.load("$$E = mc^2$$\n".to_string(), cx);
            });
        });

        let has_math = cx.read(|app| entity.read(app).has_math_or_mermaid);
        assert!(
            has_math,
            "수식이 있으면 has_math_or_mermaid = true 여야 한다"
        );
    }

    #[test]
    fn markdown_viewer_load_detects_mermaid_block() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| MarkdownViewer::new(PathBuf::from("/tmp/mermaid.md")));

        cx.update(|app| {
            entity.update(app, |viewer: &mut MarkdownViewer, cx| {
                viewer.load("```mermaid\ngraph TD; A-->B;\n```\n".to_string(), cx);
            });
        });

        let has_mermaid = cx.read(|app| entity.read(app).has_math_or_mermaid);
        assert!(
            has_mermaid,
            "mermaid 가 있으면 has_math_or_mermaid = true 여야 한다"
        );
    }

    #[test]
    fn markdown_viewer_ready_state_has_correct_block_count() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| MarkdownViewer::new(PathBuf::from("/tmp/test.md")));

        cx.update(|app| {
            entity.update(app, |viewer: &mut MarkdownViewer, cx| {
                viewer.load("# 제목\n\n단락 A\n\n단락 B\n".to_string(), cx);
            });
        });

        let block_count = cx.read(|app| {
            let v = entity.read(app);
            if let ViewerState::Ready { blocks, .. } = &v.state {
                blocks.len()
            } else {
                0
            }
        });
        // 헤딩 1 + 단락 2 = 최소 2개 블록
        assert!(
            block_count >= 2,
            "최소 2개 블록이어야 한다 (실제: {})",
            block_count
        );
    }

    #[test]
    fn markdown_viewer_entity_can_be_created_via_gpui_context() {
        // AC-MV-1: Entity<MarkdownViewer> 생성 smoke 테스트
        let mut cx = TestAppContext::single();
        let path = PathBuf::from("/tmp/test.md");
        let entity = cx.new(|_cx| MarkdownViewer::new(path.clone()));
        let ws_path = cx.read(|app| entity.read(app).path.clone());
        assert_eq!(ws_path, PathBuf::from("/tmp/test.md"));
    }
}
