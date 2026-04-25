//! SPEC-V3-006 MS-2 T10: tree-sitter 언어 지원 모듈.
//!
//! `SupportedLang` enum 과 확장자 기반 언어 감지 함수를 제공한다.
//! `load_grammar` 은 각 언어의 tree-sitter `Language` 를 반환한다.

// @MX:ANCHOR: [AUTO] supported-lang-set
// @MX:REASON: [AUTO] 4-lang 정책 invariant (REQ-MV-021). Rust/Go/Python/TypeScript.
//   확장 시 이 enum 과 load_grammar, detect_lang_from_extension 세 곳을 모두 수정해야 한다.

use tree_sitter::Language;

// ============================================================
// SupportedLang
// ============================================================

/// tree-sitter syntax highlight 지원 언어 집합 (REQ-MV-021, USER-DECISION OD-MV2).
///
/// MS-2 에서는 Rust / Go / Python / TypeScript 4개 언어를 우선 지원한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLang {
    Rust,
    Go,
    Python,
    TypeScript,
}

// ============================================================
// detect_lang_from_extension
// ============================================================

/// 파일 확장자 문자열로 `SupportedLang` 을 감지한다.
///
/// 매핑:
/// - `rs` → Rust
/// - `go` → Go
/// - `py` → Python
/// - `ts`, `tsx` → TypeScript
/// - 그 외 → None
pub fn detect_lang_from_extension(ext: &str) -> Option<SupportedLang> {
    match ext.to_ascii_lowercase().as_str() {
        "rs" => Some(SupportedLang::Rust),
        "go" => Some(SupportedLang::Go),
        "py" => Some(SupportedLang::Python),
        "ts" | "tsx" => Some(SupportedLang::TypeScript),
        _ => None,
    }
}

// ============================================================
// load_grammar
// ============================================================

/// 주어진 언어에 해당하는 tree-sitter `Language` 를 반환한다.
///
/// 각 grammar crate 의 `language()` fn 을 wrapping 한다.
pub fn load_grammar(lang: SupportedLang) -> Language {
    match lang {
        SupportedLang::Rust => tree_sitter_rust::LANGUAGE.into(),
        SupportedLang::Go => tree_sitter_go::LANGUAGE.into(),
        SupportedLang::Python => tree_sitter_python::LANGUAGE.into(),
        SupportedLang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
    }
}

// ============================================================
// 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_lang_from_extension_rs_returns_rust() {
        assert_eq!(detect_lang_from_extension("rs"), Some(SupportedLang::Rust));
    }

    #[test]
    fn detect_lang_unknown_returns_none() {
        assert_eq!(detect_lang_from_extension("html"), None);
        assert_eq!(detect_lang_from_extension(""), None);
        assert_eq!(detect_lang_from_extension("java"), None);
    }

    #[test]
    fn detect_lang_from_extension_go_returns_go() {
        assert_eq!(detect_lang_from_extension("go"), Some(SupportedLang::Go));
    }

    #[test]
    fn detect_lang_from_extension_py_returns_python() {
        assert_eq!(
            detect_lang_from_extension("py"),
            Some(SupportedLang::Python)
        );
    }

    #[test]
    fn detect_lang_from_extension_ts_returns_typescript() {
        assert_eq!(
            detect_lang_from_extension("ts"),
            Some(SupportedLang::TypeScript)
        );
        assert_eq!(
            detect_lang_from_extension("tsx"),
            Some(SupportedLang::TypeScript)
        );
    }

    #[test]
    fn load_grammar_returns_valid_language_for_all_langs() {
        // 각 grammar 를 로드할 수 있어야 한다 (panic 없음 = 성공)
        let _ = load_grammar(SupportedLang::Rust);
        let _ = load_grammar(SupportedLang::Go);
        let _ = load_grammar(SupportedLang::Python);
        let _ = load_grammar(SupportedLang::TypeScript);
    }
}
