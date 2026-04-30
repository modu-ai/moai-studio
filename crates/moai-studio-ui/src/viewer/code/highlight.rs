//! SPEC-V3-006 MS-2 T12: tree-sitter syntax highlight 파이프라인.
//!
//! `highlight_source` 는 소스 코드를 파싱하고 노드 종류 기반으로
//! `HighlightedLine` 목록을 반환하는 pure function 이다.
//!
//! `scope_to_color` 는 `HighlightScope` → RGB `[u8; 3]` 변환을
//! tokens.json `color.syntax` 섹션의 hex 값과 일치시켜 반환한다.

// @MX:ANCHOR: [AUTO] syntax-highlight-pipeline
// @MX:REASON: [AUTO] tree-sitter highlight 단일 진입점. fan_in >= 3:
//   CodeViewer::load (T11), markdown fenced code (T13), 미래 Diff viewer (V3-008).
// @MX:NOTE: [AUTO] scope-color-token-binding
// scope_to_color 의 RGB 값은 tokens.json color.syntax 섹션에서 import 한다.
// keyword=#C792EA, string=#C3E88D, number=#F78C6C, comment=#546E7A,
// function=#82AAFF, type=#FFCB6B, variable=#EEFFFF, operator=#89DDFF,
// constant=#F07178, tag=#F07178, attribute=#FFCB6B

use crate::viewer::code::languages::SupportedLang;
use tree_sitter::Parser;

// ============================================================
// HighlightScope
// ============================================================

/// syntax highlight 스코프 종류.
///
/// tree-sitter 노드 종류를 이 enum 으로 매핑한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightScope {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Constant,
    Tag,
    Attribute,
}

// ============================================================
// HighlightedSpan / HighlightedLine
// ============================================================

/// highlight 된 텍스트 조각.
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightedSpan {
    pub text: String,
    pub scope: Option<HighlightScope>,
}

/// 한 줄을 구성하는 `HighlightedSpan` 목록.
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightedLine {
    pub spans: Vec<HighlightedSpan>,
}

// ============================================================
// scope_to_color
// ============================================================

/// `HighlightScope` → RGB `[u8; 3]` 변환.
///
/// 값은 tokens.json `color.syntax` 섹션과 정확히 일치한다.
///
/// | scope     | hex     | RGB              |
/// |-----------|---------|------------------|
/// | Keyword   | C792EA  | [199, 146, 234]  |
/// | String    | C3E88D  | [195, 232, 141]  |
/// | Number    | F78C6C  | [247, 140, 108]  |
/// | Comment   | 546E7A  | [84, 110, 122]   |
/// | Function  | 82AAFF  | [130, 170, 255]  |
/// | Type      | FFCB6B  | [255, 203, 107]  |
/// | Variable  | EEFFFF  | [238, 255, 255]  |
/// | Operator  | 89DDFF  | [137, 221, 255]  |
/// | Constant  | F07178  | [240, 113, 120]  |
/// | Tag       | F07178  | [240, 113, 120]  |
/// | Attribute | FFCB6B  | [255, 203, 107]  |
pub fn scope_to_color(scope: &HighlightScope) -> [u8; 3] {
    match scope {
        // tokens.json color.syntax.keyword = #C792EA
        HighlightScope::Keyword => [0xC7, 0x92, 0xEA],
        // tokens.json color.syntax.string = #C3E88D
        HighlightScope::String => [0xC3, 0xE8, 0x8D],
        // tokens.json color.syntax.number = #F78C6C
        HighlightScope::Number => [0xF7, 0x8C, 0x6C],
        // tokens.json color.syntax.comment = #546E7A
        HighlightScope::Comment => [0x54, 0x6E, 0x7A],
        // tokens.json color.syntax.function = #82AAFF
        HighlightScope::Function => [0x82, 0xAA, 0xFF],
        // tokens.json color.syntax.type = #FFCB6B
        HighlightScope::Type => [0xFF, 0xCB, 0x6B],
        // tokens.json color.syntax.variable = #EEFFFF
        HighlightScope::Variable => [0xEE, 0xFF, 0xFF],
        // tokens.json color.syntax.operator = #89DDFF
        HighlightScope::Operator => [0x89, 0xDD, 0xFF],
        // tokens.json color.syntax.constant = #F07178
        HighlightScope::Constant => [0xF0, 0x71, 0x78],
        // tokens.json color.syntax.tag = #F07178
        HighlightScope::Tag => [0xF0, 0x71, 0x78],
        // tokens.json color.syntax.attribute = #FFCB6B
        HighlightScope::Attribute => [0xFF, 0xCB, 0x6B],
    }
}

// ============================================================
// highlight_source
// ============================================================

/// 소스 코드를 tree-sitter 로 파싱하여 `HighlightedLine` 목록을 반환한다.
///
/// 파이프라인:
/// 1. `Parser` 생성 및 grammar 설정
/// 2. 소스 파싱 → `Tree`
/// 3. 노드 순회 → 바이트 오프셋 기반 span 분리
/// 4. 노드 종류 → `HighlightScope` 매핑
/// 5. 줄 단위 분리 → `HighlightedLine` 반환
///
/// 파싱 실패 시 plain text 로 fallback 한다 (panic 없음 보장).
pub fn highlight_source(code: &str, lang: SupportedLang) -> Vec<HighlightedLine> {
    let grammar = crate::viewer::code::languages::load_grammar(lang);
    let mut parser = Parser::new();
    // language 설정 실패는 fallback
    if parser.set_language(&grammar).is_err() {
        return plain_lines(code);
    }

    let tree = match parser.parse(code, None) {
        Some(t) => t,
        None => return plain_lines(code),
    };

    let code_bytes = code.as_bytes();
    let root = tree.root_node();

    // 전체 줄 목록을 먼저 준비 (줄 경계 계산용)
    let line_ends: Vec<usize> = code_bytes
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if b == b'\n' { Some(i) } else { None })
        .collect();

    // 노드를 DFS 로 수집 → (start_byte, end_byte, kind) 목록
    let mut highlights: Vec<(usize, usize, Option<HighlightScope>)> = Vec::new();
    collect_highlights(root, code_bytes, lang, &mut highlights);
    highlights.sort_by_key(|h| h.0);

    // 바이트 오프셋 기반으로 줄별 span 분리
    build_highlighted_lines(code, &highlights, &line_ends)
}

// ============================================================
// 내부 헬퍼: collect_highlights
// ============================================================

/// 노드 트리를 DFS 로 순회하며 leaf 노드의 highlight 정보를 수집한다.
fn collect_highlights(
    node: tree_sitter::Node<'_>,
    code_bytes: &[u8],
    lang: SupportedLang,
    out: &mut Vec<(usize, usize, Option<HighlightScope>)>,
) {
    if node.child_count() == 0 {
        // leaf 노드
        let scope = map_node_kind(node.kind(), lang);
        out.push((node.start_byte(), node.end_byte(), scope));
    } else {
        let mut child_start = node.start_byte();
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            // child 이전 anonymous 텍스트 (공백 등)
            if child.start_byte() > child_start {
                out.push((child_start, child.start_byte(), None));
            }
            collect_highlights(child, code_bytes, lang, out);
            child_start = child.end_byte();
        }
        // 마지막 child 이후 텍스트
        if child_start < node.end_byte() {
            out.push((child_start, node.end_byte(), None));
        }
    }
    let _ = code_bytes; // 사용하지 않는 warning 억제
}

// ============================================================
// 내부 헬퍼: map_node_kind
// ============================================================

/// 언어별 tree-sitter 노드 종류 → `HighlightScope` 매핑.
///
/// MS-2 simplified mapping: 핵심 노드 종류 10개 우선.
/// MS-5: JavaScript / JSON 추가 매핑.
fn map_node_kind(kind: &str, lang: SupportedLang) -> Option<HighlightScope> {
    match lang {
        SupportedLang::Rust => map_rust_kind(kind),
        SupportedLang::Go => map_go_kind(kind),
        SupportedLang::Python => map_python_kind(kind),
        SupportedLang::TypeScript => map_typescript_kind(kind),
        SupportedLang::JavaScript => map_javascript_kind(kind),
        SupportedLang::Json => map_json_kind(kind),
    }
}

fn map_rust_kind(kind: &str) -> Option<HighlightScope> {
    match kind {
        // 키워드
        "fn" | "let" | "const" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl"
        | "trait" | "where" | "for" | "in" | "while" | "loop" | "if" | "else" | "match"
        | "return" | "type" | "self" | "Self" | "crate" | "super" | "async" | "await" | "move"
        | "ref" | "unsafe" | "extern" | "static" | "dyn" | "break" | "continue" => {
            Some(HighlightScope::Keyword)
        }
        // 문자열 리터럴
        "string_literal" | "raw_string_literal" | "char_literal" => Some(HighlightScope::String),
        // 숫자 리터럴
        "integer_literal" | "float_literal" => Some(HighlightScope::Number),
        // 주석
        "line_comment" | "block_comment" => Some(HighlightScope::Comment),
        // 함수 이름 (identifier in function context)
        "function_item" => Some(HighlightScope::Function),
        // 타입 식별자
        "type_identifier" | "primitive_type" => Some(HighlightScope::Type),
        // 연산자
        "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" | "!"
        | "&" | "|" | "^" | "<<" | ">>" | "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "->" | "=>"
        | "::" | "." | ".." | "..=" => Some(HighlightScope::Operator),
        // 상수
        "boolean_literal" => Some(HighlightScope::Constant),
        _ => None,
    }
}

fn map_go_kind(kind: &str) -> Option<HighlightScope> {
    match kind {
        // 키워드
        "func" | "var" | "const" | "type" | "struct" | "interface" | "map" | "chan" | "go"
        | "defer" | "select" | "case" | "default" | "for" | "range" | "if" | "else" | "switch"
        | "return" | "break" | "continue" | "goto" | "fallthrough" | "import" | "package" => {
            Some(HighlightScope::Keyword)
        }
        // 문자열 리터럴
        "interpreted_string_literal" | "raw_string_literal" | "rune_literal" => {
            Some(HighlightScope::String)
        }
        // 숫자 리터럴
        "int_literal" | "float_literal" | "imaginary_literal" => Some(HighlightScope::Number),
        // 주석
        "comment" => Some(HighlightScope::Comment),
        // 함수
        "function_declaration" | "method_declaration" => Some(HighlightScope::Function),
        // 타입
        "type_identifier" => Some(HighlightScope::Type),
        // 연산자
        "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" | "!"
        | "&" | "|" | "^" | "<<" | ">>" | "=" | "+=" | "-=" | ":=" => {
            Some(HighlightScope::Operator)
        }
        // 상수
        "true" | "false" | "nil" => Some(HighlightScope::Constant),
        _ => None,
    }
}

fn map_python_kind(kind: &str) -> Option<HighlightScope> {
    match kind {
        // 키워드
        "def" | "class" | "return" | "yield" | "import" | "from" | "as" | "if" | "elif"
        | "else" | "for" | "while" | "with" | "try" | "except" | "finally" | "raise" | "pass"
        | "break" | "continue" | "and" | "or" | "not" | "in" | "is" | "lambda" | "global"
        | "nonlocal" | "del" | "assert" | "async" | "await" => Some(HighlightScope::Keyword),
        // 문자열 (tree-sitter Python 0.25 leaf nodes: string_start, string_content, string_end)
        "string" | "string_start" | "string_content" | "string_end" | "concatenated_string" => {
            Some(HighlightScope::String)
        }
        // 숫자
        "integer" | "float" => Some(HighlightScope::Number),
        // 주석
        "comment" => Some(HighlightScope::Comment),
        // 함수
        "function_definition" | "decorated_definition" => Some(HighlightScope::Function),
        // 타입 (type annotation)
        "type" => Some(HighlightScope::Type),
        // 연산자
        "+" | "-" | "*" | "/" | "//" | "%" | "**" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "="
        | "+=" | "-=" | "*=" | "/=" | "//=" | "%=" | "**=" | "->" | ":" | "." => {
            Some(HighlightScope::Operator)
        }
        // 상수
        "true" | "false" | "none" | "True" | "False" | "None" => Some(HighlightScope::Constant),
        _ => None,
    }
}

fn map_typescript_kind(kind: &str) -> Option<HighlightScope> {
    match kind {
        // 키워드
        "function" | "const" | "let" | "var" | "class" | "interface" | "type" | "enum"
        | "namespace" | "module" | "import" | "export" | "default" | "from" | "as" | "if"
        | "else" | "for" | "while" | "do" | "switch" | "case" | "break" | "continue" | "return"
        | "throw" | "try" | "catch" | "finally" | "new" | "this" | "super" | "extends"
        | "implements" | "in" | "instanceof" | "typeof" | "void" | "delete" | "async" | "await"
        | "yield" | "static" | "public" | "private" | "protected" | "readonly" | "abstract"
        | "declare" | "override" => Some(HighlightScope::Keyword),
        // 문자열
        "string" | "template_string" => Some(HighlightScope::String),
        // 주석
        "comment" | "hash_bang_line" => Some(HighlightScope::Comment),
        // 함수
        "function_declaration" | "arrow_function" | "method_definition" => {
            Some(HighlightScope::Function)
        }
        // 타입 (tree-sitter-typescript 0.23: "number" leaf 는 타입 "number" 와 숫자 리터럴 둘 다 매핑됨)
        "type_identifier" | "predefined_type" | "number" => Some(HighlightScope::Type),
        // 연산자
        "+" | "-" | "*" | "/" | "%" | "==" | "===" | "!=" | "!==" | "<" | ">" | "<=" | ">="
        | "&&" | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | ">>>" | "=" | "+=" | "-=" | "*="
        | "/=" | "=>" | "?." | "??" => Some(HighlightScope::Operator),
        // 상수
        "true" | "false" | "null" | "undefined" => Some(HighlightScope::Constant),
        _ => None,
    }
}

/// SPEC-V3-006 MS-5: JavaScript node-kind → highlight scope mapping.
/// Mirrors map_typescript_kind, omitting TS-only keywords (interface, namespace, etc.)
/// and predefined_type. Tree-sitter-javascript exposes the same core node kinds
/// (function, string, comment, function_declaration) as the TS grammar.
fn map_javascript_kind(kind: &str) -> Option<HighlightScope> {
    match kind {
        // Keywords (no TS-only ones).
        "function" | "const" | "let" | "var" | "class" | "import" | "export" | "default"
        | "from" | "as" | "if" | "else" | "for" | "while" | "do" | "switch" | "case" | "break"
        | "continue" | "return" | "throw" | "try" | "catch" | "finally" | "new" | "this"
        | "super" | "extends" | "in" | "instanceof" | "typeof" | "void" | "delete" | "async"
        | "await" | "yield" | "static" | "of" => Some(HighlightScope::Keyword),
        // Strings (regular, template, regex).
        "string" | "template_string" | "regex" => Some(HighlightScope::String),
        // Comments (line + block + hashbang for Node.js scripts).
        "comment" | "hash_bang_line" => Some(HighlightScope::Comment),
        // Numbers (no separate type node in JS).
        "number" => Some(HighlightScope::Number),
        // Functions.
        "function_declaration" | "arrow_function" | "method_definition" => {
            Some(HighlightScope::Function)
        }
        // Operators.
        "+" | "-" | "*" | "/" | "%" | "**" | "==" | "===" | "!=" | "!==" | "<" | ">" | "<="
        | ">=" | "&&" | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | ">>>" | "=" | "+=" | "-="
        | "*=" | "/=" | "=>" | "?." | "??" | "..." => Some(HighlightScope::Operator),
        // Constants.
        "true" | "false" | "null" | "undefined" => Some(HighlightScope::Constant),
        _ => None,
    }
}

/// SPEC-V3-006 MS-5: JSON node-kind → highlight scope mapping.
/// JSON grammar is small: object, array, string, number, true/false/null,
/// pair (key:value). Comments are non-standard but supported by jsonc dialect.
fn map_json_kind(kind: &str) -> Option<HighlightScope> {
    match kind {
        // Strings (both keys and values share the "string" node kind in tree-sitter-json).
        "string" | "string_content" => Some(HighlightScope::String),
        // Numbers.
        "number" => Some(HighlightScope::Number),
        // JSON literals.
        "true" | "false" | "null" => Some(HighlightScope::Constant),
        // Comments (only in JSONC dialect; tree-sitter-json may emit them anyway).
        "comment" => Some(HighlightScope::Comment),
        // Structural punctuation as operators.
        "{" | "}" | "[" | "]" | ":" | "," => Some(HighlightScope::Operator),
        _ => None,
    }
}

// ============================================================
// 내부 헬퍼: build_highlighted_lines
// ============================================================

/// 바이트 오프셋 기반 highlight 목록을 줄 단위 `HighlightedLine` 으로 변환한다.
fn build_highlighted_lines(
    code: &str,
    highlights: &[(usize, usize, Option<HighlightScope>)],
    _line_ends: &[usize],
) -> Vec<HighlightedLine> {
    let code_bytes = code.as_bytes();
    let total_len = code_bytes.len();

    // 각 바이트 위치에 scope 를 매핑
    let mut byte_scopes: Vec<Option<HighlightScope>> = vec![None; total_len];
    for (start, end, scope) in highlights {
        let end_clamped = (*end).min(total_len);
        for slot in byte_scopes.iter_mut().take(end_clamped).skip(*start) {
            if slot.is_none() {
                *slot = scope.clone();
            }
        }
    }

    // 줄 단위로 분리
    let mut lines: Vec<HighlightedLine> = Vec::new();
    let mut line_spans: Vec<HighlightedSpan> = Vec::new();
    let mut current_text = String::new();
    let mut current_scope: Option<HighlightScope> = None;

    let chars: Vec<char> = code.chars().collect();
    let mut byte_pos = 0usize;

    for ch in &chars {
        let ch_len = ch.len_utf8();

        if *ch == '\n' {
            // 현재 span 마무리
            if !current_text.is_empty() {
                line_spans.push(HighlightedSpan {
                    text: current_text.clone(),
                    scope: current_scope.clone(),
                });
                current_text.clear();
            }
            // 줄 완성
            lines.push(HighlightedLine {
                spans: line_spans.clone(),
            });
            line_spans.clear();
            current_scope = None;
        } else {
            let scope = if byte_pos < total_len {
                byte_scopes[byte_pos].clone()
            } else {
                None
            };

            if scope != current_scope && !current_text.is_empty() {
                line_spans.push(HighlightedSpan {
                    text: current_text.clone(),
                    scope: current_scope.clone(),
                });
                current_text.clear();
            }
            current_scope = scope;
            current_text.push(*ch);
        }

        byte_pos += ch_len;
    }

    // 마지막 미완성 줄
    if !current_text.is_empty() {
        line_spans.push(HighlightedSpan {
            text: current_text,
            scope: current_scope,
        });
    }
    if !line_spans.is_empty() {
        lines.push(HighlightedLine { spans: line_spans });
    }

    lines
}

/// plain text fallback: 줄 단위로 분리하여 scope 없는 span 반환.
fn plain_lines(code: &str) -> Vec<HighlightedLine> {
    code.lines()
        .map(|line| HighlightedLine {
            spans: vec![HighlightedSpan {
                text: line.to_string(),
                scope: None,
            }],
        })
        .collect()
}

// ============================================================
// 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewer::code::languages::SupportedLang;

    #[test]
    fn highlight_rust_basic_keyword() {
        // "fn" 키워드가 Keyword scope 로 highlight 되어야 한다
        let code = "fn main() {}";
        let lines = highlight_source(code, SupportedLang::Rust);
        assert!(!lines.is_empty(), "결과가 비어있지 않아야 한다");

        let has_keyword = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Keyword) && s.text.contains("fn"))
        });
        assert!(has_keyword, "fn 이 Keyword scope 로 highlight 되어야 한다");
    }

    #[test]
    fn highlight_python_string_literal() {
        // Python 문자열 리터럴이 String scope 로 highlight 되어야 한다
        let code = "x = \"hello world\"";
        let lines = highlight_source(code, SupportedLang::Python);
        assert!(!lines.is_empty());

        let has_string = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::String))
        });
        assert!(
            has_string,
            "문자열 리터럴이 String scope 로 highlight 되어야 한다"
        );
    }

    #[test]
    fn highlight_typescript_type_annotation() {
        // TypeScript 타입 식별자가 Type scope 로 highlight 되어야 한다
        let code = "const x: number = 42;";
        let lines = highlight_source(code, SupportedLang::TypeScript);
        assert!(!lines.is_empty());

        let has_type = lines.iter().any(|line| {
            line.spans.iter().any(|s| {
                s.scope == Some(HighlightScope::Type) || s.scope == Some(HighlightScope::Number)
            })
        });
        assert!(has_type, "타입 또는 숫자가 highlight 되어야 한다");
    }

    #[test]
    fn highlight_go_function_definition() {
        // Go 함수 정의가 highlight 되어야 한다
        let code = "func main() {}";
        let lines = highlight_source(code, SupportedLang::Go);
        assert!(!lines.is_empty());

        let has_keyword = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Keyword) && s.text.contains("func"))
        });
        assert!(
            has_keyword,
            "func 가 Keyword scope 로 highlight 되어야 한다"
        );
    }

    // ── SPEC-V3-006 MS-5: JavaScript / JSON highlight tests ──

    #[test]
    fn highlight_javascript_const_keyword() {
        let code = "const x = 42;";
        let lines = highlight_source(code, SupportedLang::JavaScript);
        assert!(!lines.is_empty(), "JS highlight result must not be empty");

        let has_keyword = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Keyword) && s.text.contains("const"))
        });
        assert!(has_keyword, "JS `const` must highlight as Keyword");
    }

    #[test]
    fn highlight_javascript_operators_and_constants() {
        // Pipeline only assigns scope to LEAF nodes, so we test leaf-level
        // tokens that the JS grammar exposes directly: =>, ===, true, false.
        let code = "const ok = (a === b) => true;";
        let lines = highlight_source(code, SupportedLang::JavaScript);

        let has_operator = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Operator))
        });
        let has_constant = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Constant) && s.text == "true")
        });
        assert!(
            has_operator,
            "JS operators (===, =>) must highlight Operator scope"
        );
        assert!(has_constant, "JS `true` must highlight Constant scope");
    }

    #[test]
    fn highlight_javascript_number_literal() {
        let code = "let x = 3.14;";
        let lines = highlight_source(code, SupportedLang::JavaScript);

        let has_number = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Number))
        });
        assert!(has_number, "JS number literal must highlight Number scope");
    }

    #[test]
    fn highlight_json_object_with_strings_and_numbers() {
        let code = r#"{"name": "moai", "version": 12, "active": true}"#;
        let lines = highlight_source(code, SupportedLang::Json);

        let has_string = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::String))
        });
        let has_number = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Number))
        });
        let has_constant = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Constant) && s.text == "true")
        });

        assert!(has_string, "JSON strings must highlight String scope");
        assert!(has_number, "JSON numbers must highlight Number scope");
        assert!(
            has_constant,
            "JSON `true` literal must highlight Constant scope"
        );
    }

    #[test]
    fn highlight_json_null_literal() {
        let code = r#"{"x": null}"#;
        let lines = highlight_source(code, SupportedLang::Json);

        let has_null = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Constant) && s.text == "null")
        });
        assert!(has_null, "JSON `null` must highlight Constant scope");
    }

    #[test]
    fn highlight_json_punctuation_as_operator() {
        let code = r#"{"a":1,"b":2}"#;
        let lines = highlight_source(code, SupportedLang::Json);

        let has_operator = lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|s| s.scope == Some(HighlightScope::Operator))
        });
        assert!(
            has_operator,
            "JSON braces / colon / comma must highlight Operator scope"
        );
    }

    #[test]
    fn scope_to_color_matches_tokens_json() {
        // tokens.json color.syntax 섹션 hex 값과 정확히 일치해야 한다
        assert_eq!(scope_to_color(&HighlightScope::Keyword), [0xC7, 0x92, 0xEA]); // #C792EA
        assert_eq!(scope_to_color(&HighlightScope::String), [0xC3, 0xE8, 0x8D]); // #C3E88D
        assert_eq!(scope_to_color(&HighlightScope::Number), [0xF7, 0x8C, 0x6C]); // #F78C6C
        assert_eq!(scope_to_color(&HighlightScope::Comment), [0x54, 0x6E, 0x7A]); // #546E7A
        assert_eq!(
            scope_to_color(&HighlightScope::Function),
            [0x82, 0xAA, 0xFF]
        ); // #82AAFF
        assert_eq!(scope_to_color(&HighlightScope::Type), [0xFF, 0xCB, 0x6B]); // #FFCB6B
        assert_eq!(
            scope_to_color(&HighlightScope::Variable),
            [0xEE, 0xFF, 0xFF]
        ); // #EEFFFF
        assert_eq!(
            scope_to_color(&HighlightScope::Operator),
            [0x89, 0xDD, 0xFF]
        ); // #89DDFF
        assert_eq!(
            scope_to_color(&HighlightScope::Constant),
            [0xF0, 0x71, 0x78]
        ); // #F07178
        assert_eq!(scope_to_color(&HighlightScope::Tag), [0xF0, 0x71, 0x78]); // #F07178
        assert_eq!(
            scope_to_color(&HighlightScope::Attribute),
            [0xFF, 0xCB, 0x6B]
        ); // #FFCB6B
    }
}
