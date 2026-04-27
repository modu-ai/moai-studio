//! SPEC-V3-006 RG-MV-1: pulldown-cmark Event → GPUI element 변환.
//!
//! `parse_markdown(input)` 은 pulldown-cmark 이벤트 스트림을 `Vec<MarkdownBlock>` 으로
//! 변환한다. 렌더링은 `MarkdownViewer::render` 에서 이 block list 를 소비한다.
//!
//! USER-DECISION (c): KaTeX/Mermaid 는 MS-3 시점 WebView 채택 전까지
//! mono-font 텍스트 fallback 으로 표시된다.
//!
//! C-2 (RELEASE-V0.1.2): EARS clause heading recognition added.
//! EARS headings starting with WHEN/WHILE/WHERE/IF/THEN/OTHERWISE at line-start
//! are tagged with `ClauseKind` for accent-color rendering.
// @MX:NOTE: [AUTO] katex-mermaid-fallback
// USER-DECISION-A=(c): MS-1/MS-2 에서 수식과 mermaid 블록은 코드 블록으로 fallback.
// MS-3 T24 에서 wry WebView + KaTeX/Mermaid 로 업그레이드 예정.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

// ============================================================
// ClauseKind — EARS clause heading tag (C-2)
// ============================================================

/// EARS requirement clause type detected from heading text (C-2 feature).
///
/// A heading whose text begins with one of the EARS keywords (case-sensitive,
/// at the very start of the trimmed text) is tagged with the matching variant.
/// Headings that do not match any keyword carry `None` in `MarkdownBlock::Heading`.
///
/// EARS keywords: WHEN, WHILE, WHERE, IF, THEN, OTHERWISE, SHALL NOT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClauseKind {
    /// Event-driven: "WHEN <trigger>, the system shall <response>"
    When,
    /// State-driven: "WHILE <state>, the system shall <response>"
    While,
    /// Optional: "WHERE <feature> is available, the system shall <response>"
    Where,
    /// Conditional: "IF <condition>, the system shall <response>"
    If,
    /// Sequential: "THEN the system shall <response>"
    Then,
    /// Alternative branch: "OTHERWISE the system shall <response>"
    Otherwise,
    /// Unwanted behaviour: "SHALL NOT <action>"
    ShallNot,
}

/// Detect an EARS clause kind from the trimmed heading text.
///
/// Returns `Some(ClauseKind)` when the text starts with a recognised EARS
/// keyword (case-sensitive match against the trimmed prefix), `None` otherwise.
pub fn detect_clause(heading_text: &str) -> Option<ClauseKind> {
    let t = heading_text.trim_start();
    // Order matters: check longer prefixes before shorter ones to avoid false
    // positives (e.g. "OTHERWISE" before "OTHER").
    if t.starts_with("OTHERWISE") {
        Some(ClauseKind::Otherwise)
    } else if t.starts_with("SHALL NOT") {
        Some(ClauseKind::ShallNot)
    } else if t.starts_with("WHEN ") || t == "WHEN" {
        Some(ClauseKind::When)
    } else if t.starts_with("WHILE ") || t == "WHILE" {
        Some(ClauseKind::While)
    } else if t.starts_with("WHERE ") || t == "WHERE" {
        Some(ClauseKind::Where)
    } else if t.starts_with("IF ") || t == "IF" {
        Some(ClauseKind::If)
    } else if t.starts_with("THEN ") || t == "THEN" {
        Some(ClauseKind::Then)
    } else {
        None
    }
}

// ============================================================
// CodeBlockLang — language-hint classification (C-2)
// ============================================================

/// Semantic classification of a fenced code block's language hint.
///
/// Used to distinguish Mermaid/math blocks (future renderer targets) from
/// regular code blocks without branching on raw strings at the render layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeBlockLang {
    /// A Mermaid diagram block — placeholder rendering until C-7.
    Mermaid,
    /// A KaTeX / LaTeX math block — placeholder rendering until future MS.
    Math,
    /// A named programming language (may have tree-sitter highlight).
    Named(String),
    /// No language hint was specified.
    Plain,
}

/// Classify a raw fenced-block language string into a `CodeBlockLang`.
///
/// Comparison is case-insensitive for the special keywords; the `Named`
/// variant preserves the original casing of the language string.
pub fn classify_lang(lang: Option<&str>) -> CodeBlockLang {
    match lang {
        None => CodeBlockLang::Plain,
        Some(s) => match s.to_ascii_lowercase().as_str() {
            "mermaid" => CodeBlockLang::Mermaid,
            "math" | "katex" | "latex" => CodeBlockLang::Math,
            _ => CodeBlockLang::Named(s.to_string()),
        },
    }
}

// ============================================================
// MarkdownBlock — 중간 표현
// ============================================================

/// pulldown-cmark 이벤트를 변환한 마크다운 블록.
///
/// MS-1 에서 지원하는 block 종류만 정의한다.
/// MS-2 에서 CodeBlock 에 `highlighted` 필드가 추가되었다.
/// MS-3 에서 Image, Table, TaskItem 등이 추가될 예정이다.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownBlock {
    /// Heading (H1 ~ H6).
    ///
    /// `clause` is `Some` when the heading text starts with a recognised EARS
    /// keyword (see `ClauseKind`), enabling accent-colour rendering (C-2).
    Heading {
        level: u8,
        text: String,
        /// EARS clause kind, detected from the heading text prefix (C-2).
        clause: Option<ClauseKind>,
    },
    /// 일반 단락 텍스트
    Paragraph(String),
    /// 펜스 코드 블록 (lang 은 `rust`, `python` 등, None = 언어 미지정).
    ///
    /// MS-2: `highlighted` 는 SupportedLang 에 해당하면 Some, 미지원이면 None.
    CodeBlock {
        lang: Option<String>,
        code: String,
        /// tree-sitter highlight 결과 (MS-2 T13). None 이면 plain text 렌더.
        highlighted: Option<Vec<crate::viewer::code::highlight::HighlightedLine>>,
    },
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
                    let clause = detect_clause(&text);
                    blocks.push(MarkdownBlock::Heading {
                        level,
                        text,
                        clause,
                    });
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
                        // T13: SupportedLang 에 해당하면 tree-sitter highlight 적용
                        let highlighted = maybe_highlight_code_block(lang.as_deref(), &code);
                        blocks.push(MarkdownBlock::CodeBlock {
                            lang,
                            code,
                            highlighted,
                        });
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

// ============================================================
// maybe_highlight_code_block — T13
// ============================================================

/// 펜스 코드 블록에 tree-sitter highlight 를 적용한다.
///
/// `lang` 이 `SupportedLang` 에 해당하면 `highlight_source` 를 호출하고
/// `Some(lines)` 를 반환한다. 미지원 언어이면 `None` 을 반환한다.
///
/// T14 (USER-DECISION c): `mermaid` 와 `math` 는 호출 전에 별도 처리되므로
/// 이 함수에 도달하지 않는다.
pub fn maybe_highlight_code_block(
    lang: Option<&str>,
    code: &str,
) -> Option<Vec<crate::viewer::code::highlight::HighlightedLine>> {
    use crate::viewer::code::highlight::highlight_source;
    use crate::viewer::code::languages::detect_lang_from_extension;

    let lang_str = lang?;
    // 언어 문자열 → 확장자 매핑 (펜스 코드 블록 lang 은 확장자처럼 동작)
    let ext = match lang_str.to_ascii_lowercase().as_str() {
        "rust" | "rs" => "rs",
        "go" | "golang" => "go",
        "python" | "py" => "py",
        "typescript" | "ts" | "tsx" => "ts",
        _ => return None, // 미지원 언어
    };

    let supported_lang = detect_lang_from_extension(ext)?;
    Some(highlight_source(code, supported_lang))
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
                if let MarkdownBlock::Heading { level, text, .. } = b {
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
                if let MarkdownBlock::CodeBlock { lang, code, .. } = b {
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
    fn markdown_fenced_code_uses_tree_sitter() {
        // AC-MV-2 추가: Markdown fenced code block 도 tree-sitter highlight 적용 (T13)
        let input = "```rust\nfn main() {}\n```\n";
        let blocks = parse_markdown(input);
        let highlighted_lines = blocks.iter().find_map(|b| {
            if let MarkdownBlock::CodeBlock { highlighted, .. } = b {
                highlighted.as_ref()
            } else {
                None
            }
        });
        assert!(
            highlighted_lines.is_some(),
            "Rust fenced code block 은 highlighted 가 Some 이어야 한다"
        );
        let lines = highlighted_lines.unwrap();
        assert!(!lines.is_empty(), "highlight 결과가 비어있지 않아야 한다");
    }

    #[test]
    fn markdown_fenced_code_unsupported_lang_has_no_highlight() {
        // 미지원 언어 (java) 는 highlighted = None 이어야 한다
        let input = "```java\npublic class Main {}\n```\n";
        let blocks = parse_markdown(input);
        let has_none_highlight = blocks.iter().any(|b| {
            if let MarkdownBlock::CodeBlock { highlighted, .. } = b {
                highlighted.is_none()
            } else {
                false
            }
        });
        assert!(
            has_none_highlight,
            "미지원 언어는 highlighted = None 이어야 한다"
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

    // ── C-2: EARS clause detection via detect_clause ──

    #[test]
    fn ears_when_keyword_detected() {
        assert_eq!(
            detect_clause("WHEN the user clicks"),
            Some(ClauseKind::When)
        );
    }

    #[test]
    fn ears_while_keyword_detected() {
        assert_eq!(
            detect_clause("WHILE the system is loading"),
            Some(ClauseKind::While)
        );
    }

    #[test]
    fn ears_where_keyword_detected() {
        assert_eq!(
            detect_clause("WHERE feature flag is set"),
            Some(ClauseKind::Where)
        );
    }

    #[test]
    fn ears_if_keyword_detected() {
        assert_eq!(detect_clause("IF the input is empty"), Some(ClauseKind::If));
    }

    #[test]
    fn ears_then_keyword_detected() {
        assert_eq!(
            detect_clause("THEN the system shall respond"),
            Some(ClauseKind::Then)
        );
    }

    #[test]
    fn ears_otherwise_keyword_detected() {
        assert_eq!(
            detect_clause("OTHERWISE fallback"),
            Some(ClauseKind::Otherwise)
        );
    }

    #[test]
    fn ears_shall_not_keyword_detected() {
        assert_eq!(
            detect_clause("SHALL NOT expose passwords"),
            Some(ClauseKind::ShallNot)
        );
    }

    #[test]
    fn ears_keyword_bare_word_detected() {
        // Exact keyword with no trailing space should also match.
        assert_eq!(detect_clause("WHEN"), Some(ClauseKind::When));
        assert_eq!(detect_clause("WHILE"), Some(ClauseKind::While));
    }

    #[test]
    fn non_ears_heading_returns_none() {
        assert_eq!(detect_clause("Overview"), None);
        assert_eq!(detect_clause("when lowercase"), None); // case-sensitive
        assert_eq!(detect_clause("While mixed case"), None);
    }

    #[test]
    fn leading_whitespace_stripped_before_clause_check() {
        // detect_clause trims leading whitespace before matching keywords.
        assert_eq!(
            detect_clause("  WHEN the user clicks"),
            Some(ClauseKind::When)
        );
    }

    #[test]
    fn parse_markdown_ears_heading_has_clause_kind() {
        let input = "## WHEN the user submits the form\n";
        let blocks = parse_markdown(input);
        let heading = blocks.iter().find_map(|b| {
            if let MarkdownBlock::Heading {
                level,
                text,
                clause,
            } = b
            {
                Some((*level, text.as_str(), clause.clone()))
            } else {
                None
            }
        });
        let (level, text, clause) = heading.expect("heading block must exist");
        assert_eq!(level, 2);
        assert!(text.starts_with("WHEN"), "text should start with WHEN");
        assert_eq!(clause, Some(ClauseKind::When));
    }

    #[test]
    fn parse_markdown_ordinary_heading_has_no_clause() {
        let input = "## Overview\n";
        let blocks = parse_markdown(input);
        let clause = blocks.iter().find_map(|b| {
            if let MarkdownBlock::Heading { clause, .. } = b {
                Some(clause.clone())
            } else {
                None
            }
        });
        assert_eq!(
            clause,
            Some(None),
            "non-EARS heading must have clause = None"
        );
    }

    #[test]
    fn parse_markdown_mixed_ears_and_plain_headings() {
        let input = "# Overview\n## WHEN user logs in\n### Details\n#### IF timeout\n";
        let blocks = parse_markdown(input);
        let headings: Vec<_> = blocks
            .iter()
            .filter_map(|b| {
                if let MarkdownBlock::Heading { level, clause, .. } = b {
                    Some((*level, clause.clone()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(headings.len(), 4);
        assert_eq!(headings[0], (1, None)); // Overview — no clause
        assert_eq!(headings[1], (2, Some(ClauseKind::When)));
        assert_eq!(headings[2], (3, None)); // Details — no clause
        assert_eq!(headings[3], (4, Some(ClauseKind::If)));
    }

    // ── C-2: CodeBlockLang classification ──

    #[test]
    fn classify_lang_none_is_plain() {
        assert_eq!(classify_lang(None), CodeBlockLang::Plain);
    }

    #[test]
    fn classify_lang_mermaid_lowercase() {
        assert_eq!(classify_lang(Some("mermaid")), CodeBlockLang::Mermaid);
    }

    #[test]
    fn classify_lang_mermaid_uppercase_is_also_mermaid() {
        assert_eq!(classify_lang(Some("MERMAID")), CodeBlockLang::Mermaid);
    }

    #[test]
    fn classify_lang_math_variants() {
        assert_eq!(classify_lang(Some("math")), CodeBlockLang::Math);
        assert_eq!(classify_lang(Some("katex")), CodeBlockLang::Math);
        assert_eq!(classify_lang(Some("latex")), CodeBlockLang::Math);
    }

    #[test]
    fn classify_lang_named_for_code_languages() {
        assert_eq!(
            classify_lang(Some("rust")),
            CodeBlockLang::Named("rust".to_string())
        );
        assert_eq!(
            classify_lang(Some("python")),
            CodeBlockLang::Named("python".to_string())
        );
    }

    #[test]
    fn classify_lang_named_preserves_original_casing() {
        // Named variant must preserve the original casing of the language string.
        assert_eq!(
            classify_lang(Some("TypeScript")),
            CodeBlockLang::Named("TypeScript".to_string())
        );
    }

    // ── C-2: Math/Mermaid block identification ──

    #[test]
    fn mermaid_block_produces_mermaid_variant() {
        let input = "```mermaid\ngraph TD; A-->B;\n```\n";
        let blocks = parse_markdown(input);
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, MarkdownBlock::Mermaid(_))),
            "mermaid fenced block must produce Mermaid variant"
        );
    }

    #[test]
    fn katex_fenced_block_produces_code_block_with_lang_katex() {
        // "katex" lang hint is not intercepted by pulldown-cmark math events;
        // it becomes a CodeBlock.  The renderer applies the pending-note.
        let input = "```katex\nx^2 + y^2 = z^2\n```\n";
        let blocks = parse_markdown(input);
        let code_block = blocks
            .iter()
            .find(|b| matches!(b, MarkdownBlock::CodeBlock { lang: Some(l), .. } if l == "katex"));
        assert!(
            code_block.is_some(),
            "katex fenced block must produce CodeBlock with lang=katex"
        );
    }
}
