//! SPEC-V3-006 RG-MV-1: pulldown-cmark Event → GPUI element 변환.
//!
//! `parse_markdown(input)` 은 pulldown-cmark 이벤트 스트림을 `Vec<MarkdownBlock>` 으로
//! 변환한다. 렌더링은 `MarkdownViewer::render` 에서 이 block list 를 소비한다.
//!
//! USER-DECISION (c): KaTeX/Mermaid 는 MS-3 시점 WebView 채택 전까지
//! mono-font 텍스트 fallback 으로 표시된다.
// @MX:NOTE: [AUTO] katex-mermaid-fallback
// USER-DECISION-A=(c): MS-1/MS-2 에서 수식과 mermaid 블록은 코드 블록으로 fallback.
// MS-3 T24 에서 wry WebView + KaTeX/Mermaid 로 업그레이드 예정.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

// ============================================================
// MarkdownBlock — 중간 표현
// ============================================================

/// pulldown-cmark 이벤트를 변환한 마크다운 블록.
///
/// MS-1 에서 지원하는 block 종류만 정의한다.
/// MS-2/MS-3 에서 Image, Table, TaskItem 등이 추가될 예정이다.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownBlock {
    /// 헤딩 (H1 ~ H6)
    Heading { level: u8, text: String },
    /// 일반 단락 텍스트
    Paragraph(String),
    /// 펜스 코드 블록 (lang 은 `rust`, `python` 등, None = 언어 미지정)
    CodeBlock { lang: Option<String>, code: String },
    /// 인라인 코드
    InlineCode(String),
    /// 수식 (USER-DECISION c: text fallback)
    Math(String),
    /// Mermaid 다이어그램 (USER-DECISION c: text fallback)
    Mermaid(String),
    /// 리스트 항목 목록
    List(Vec<String>),
    /// 블록쿼트
    Quote(String),
    /// 수평선
    Rule,
}

// ============================================================
// parse_markdown
// ============================================================

/// 마크다운 소스를 `Vec<MarkdownBlock>` 으로 파싱한다 (REQ-MV-002).
///
/// 활성화된 옵션:
/// - `ENABLE_TABLES` — GFM 테이블
/// - `ENABLE_STRIKETHROUGH` — 취소선
/// - `ENABLE_TASKLISTS` — 체크박스 리스트
/// - `ENABLE_FOOTNOTES` — 각주
/// - `ENABLE_HEADING_ATTRIBUTES` — 헤딩 id 속성
/// - `ENABLE_MATH` — `$...$` / `$$...$$` 수식 이벤트
pub fn parse_markdown(input: &str) -> Vec<MarkdownBlock> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    opts.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(input, opts);
    let mut blocks: Vec<MarkdownBlock> = Vec::new();

    // 현재 수집 중인 컨텍스트
    let mut current_heading: Option<(u8, String)> = None;
    let mut current_para: Option<String> = None;
    let mut current_code: Option<(Option<String>, String)> = None; // (lang, code)
    let mut current_list: Option<Vec<String>> = None;
    let mut current_list_item: Option<String> = None;
    let mut current_quote: Option<String> = None;

    for event in parser {
        match event {
            // ── 헤딩 ──
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = heading_level_to_u8(level);
                current_heading = Some((lvl, String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, text)) = current_heading.take() {
                    blocks.push(MarkdownBlock::Heading { level, text });
                }
            }

            // ── 단락 ──
            Event::Start(Tag::Paragraph) => {
                current_para = Some(String::new());
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(text) = current_para.take().filter(|t| !t.trim().is_empty()) {
                    blocks.push(MarkdownBlock::Paragraph(text));
                }
            }

            // ── 코드 블록 ──
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang_cow) => {
                        let s = lang_cow.to_string();
                        if s.is_empty() { None } else { Some(s) }
                    }
                    CodeBlockKind::Indented => None,
                };
                current_code = Some((lang, String::new()));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((lang, code)) = current_code.take() {
                    // mermaid 블록을 별도 처리 (USER-DECISION c fallback)
                    if lang.as_deref() == Some("mermaid") {
                        blocks.push(MarkdownBlock::Mermaid(code));
                    } else {
                        blocks.push(MarkdownBlock::CodeBlock { lang, code });
                    }
                }
            }

            // ── 리스트 ──
            Event::Start(Tag::List(_)) => {
                current_list = Some(Vec::new());
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(items) = current_list.take().filter(|v| !v.is_empty()) {
                    blocks.push(MarkdownBlock::List(items));
                }
            }
            Event::Start(Tag::Item) => {
                current_list_item = Some(String::new());
            }
            Event::End(TagEnd::Item) => {
                if let (Some(item), Some(list)) = (current_list_item.take(), current_list.as_mut())
                {
                    list.push(item);
                }
            }

            // ── 블록쿼트 ──
            Event::Start(Tag::BlockQuote(_)) => {
                current_quote = Some(String::new());
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if let Some(q) = current_quote.take() {
                    blocks.push(MarkdownBlock::Quote(q));
                }
            }

            // ── 수평선 ──
            Event::Rule => {
                blocks.push(MarkdownBlock::Rule);
            }

            // ── 텍스트 (공통 수집) ──
            Event::Text(s) => {
                let text = s.as_ref();
                if let Some((_, buf)) = current_heading.as_mut() {
                    buf.push_str(text);
                } else if let Some((_, buf)) = current_code.as_mut() {
                    buf.push_str(text);
                } else if let Some(buf) = current_list_item.as_mut() {
                    buf.push_str(text);
                } else if let Some(buf) = current_quote.as_mut() {
                    buf.push_str(text);
                } else if let Some(buf) = current_para.as_mut() {
                    buf.push_str(text);
                }
            }

            // ── 인라인 코드 ──
            Event::Code(s) => {
                let text = s.as_ref().to_string();
                if let Some(buf) = current_para.as_mut() {
                    // 단락 안의 인라인 코드는 backtick 으로 표현
                    buf.push_str(&format!("`{}`", text));
                } else {
                    blocks.push(MarkdownBlock::InlineCode(text));
                }
            }

            // ── 수식 (USER-DECISION c fallback) ──
            Event::DisplayMath(s) | Event::InlineMath(s) => {
                blocks.push(MarkdownBlock::Math(s.as_ref().to_string()));
            }

            // ── 줄바꿈 ──
            Event::SoftBreak | Event::HardBreak => {
                if let Some(buf) = current_para.as_mut() {
                    buf.push(' ');
                }
            }

            _ => {} // 그 외 이벤트는 무시 (Link, Image, etc.)
        }
    }

    blocks
}

/// `HeadingLevel` → 1-based u8 변환
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

// ============================================================
// 단위 테스트 — T4
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_heading_emits_correct_level() {
        let input = "# 제목 1\n## 제목 2\n### 제목 3\n";
        let blocks = parse_markdown(input);
        let headings: Vec<_> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::Heading { level, text } = b {
                    Some((*level, text.clone()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0], (1, "제목 1".to_string()));
        assert_eq!(headings[1], (2, "제목 2".to_string()));
        assert_eq!(headings[2], (3, "제목 3".to_string()));
    }

    #[test]
    fn render_paragraph_emits_text() {
        let input = "This is a paragraph.\n";
        let blocks = parse_markdown(input);
        let paras: Vec<_> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::Paragraph(t) = b {
                    Some(t.clone())
                } else {
                    None
                }
            })
            .collect();
        assert!(!paras.is_empty(), "단락 블록이 존재해야 한다");
        assert!(paras[0].contains("This is a paragraph"));
    }

    #[test]
    fn render_fenced_code_block_emits_code() {
        let input = "```rust\nfn main() {}\n```\n";
        let blocks = parse_markdown(input);
        let code_blocks: Vec<_> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::CodeBlock { lang, code } = b {
                    Some((lang.clone(), code.clone()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(code_blocks.len(), 1);
        assert_eq!(code_blocks[0].0, Some("rust".to_string()));
        assert!(code_blocks[0].1.contains("fn main()"));
    }

    #[test]
    fn render_mermaid_block_emits_mermaid() {
        let input = "```mermaid\ngraph TD; A-->B;\n```\n";
        let blocks = parse_markdown(input);
        let mermaid: Vec<_> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::Mermaid(s) = b {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(mermaid.len(), 1, "mermaid 블록이 1개여야 한다");
        assert!(mermaid[0].contains("graph TD"));
    }

    #[test]
    fn render_math_with_text_fallback_emits_math_block() {
        // USER-DECISION (c): 수식은 Math 블록으로 fallback
        let input = "$$E = mc^2$$\n";
        let blocks = parse_markdown(input);
        let math: Vec<_> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::Math(s) = b {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();
        assert!(!math.is_empty(), "수식 블록이 존재해야 한다");
    }

    #[test]
    fn render_list_emits_list_items() {
        let input = "- item A\n- item B\n- item C\n";
        let blocks = parse_markdown(input);
        let lists: Vec<_> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::List(items) = b {
                    Some(items.clone())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].len(), 3);
    }

    #[test]
    fn render_strikethrough_as_paragraph() {
        // GFM strikethrough 는 단락 텍스트로 수집된다
        let input = "~~삭제된 텍스트~~\n";
        let blocks = parse_markdown(input);
        // 단락이나 다른 형태로 emit 되었는지 확인
        assert!(!blocks.is_empty(), "빈 결과가 아니어야 한다");
    }

    #[test]
    fn empty_markdown_returns_empty_blocks() {
        let blocks = parse_markdown("");
        assert!(blocks.is_empty(), "빈 입력은 빈 블록 목록을 반환해야 한다");
    }

    #[test]
    fn horizontal_rule_emits_rule_block() {
        let input = "---\n";
        let blocks = parse_markdown(input);
        assert!(
            blocks.contains(&MarkdownBlock::Rule),
            "수평선 블록이 있어야 한다"
        );
    }

    #[test]
    fn heading_level_conversion_covers_all_levels() {
        let input = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n";
        let blocks = parse_markdown(input);
        let levels: Vec<u8> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::Heading { level, .. } = b {
                    Some(*level)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(levels, vec![1, 2, 3, 4, 5, 6]);
    }
}
