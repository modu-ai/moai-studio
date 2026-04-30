//! SPEC-V3-006 MS-4 — Math expression → Unicode preview converter.
//!
//! Converts common LaTeX-style math expressions to Unicode equivalents for
//! plain-text preview when KaTeX WebView rendering is not available.
//! This is NOT a full LaTeX renderer — it covers the most common patterns
//! found in EARS specs and engineering docs (superscripts, subscripts,
//! Greek letters, common operators).
//!
//! Full KaTeX rendering remains deferred until WebView (wry) integration
//! lands per RG-MV-2 USER-DECISION (a).

/// Converts a LaTeX-style math expression to a best-effort Unicode preview.
///
/// Handles:
/// - Superscripts: `x^2` → `x²`, `x^{10}` → `x¹⁰`
/// - Subscripts: `x_1` → `x₁`, `H_{2}O` → `H₂O`
/// - Greek letters: `\alpha` → `α`, `\beta` → `β`, etc.
/// - Common operators: `\times` → `×`, `\div` → `÷`, `\pm` → `±`
/// - Relations: `\leq` → `≤`, `\geq` → `≥`, `\neq` → `≠`, `\approx` → `≈`
/// - Sets: `\in` → `∈`, `\notin` → `∉`, `\subset` → `⊂`, `\subseteq` → `⊆`
/// - Symbols: `\infty` → `∞`, `\partial` → `∂`, `\nabla` → `∇`
///
/// Unsupported sequences are passed through unchanged.
pub fn math_to_unicode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        // Handle superscripts: x^2, x^{10}
        if b == b'^' && i + 1 < bytes.len() {
            let (consumed, sup) = parse_script(&input[i + 1..], to_superscript);
            if consumed > 0 {
                out.push_str(&sup);
                i += 1 + consumed;
                continue;
            }
        }

        // Handle subscripts: x_1, H_{2}O
        if b == b'_' && i + 1 < bytes.len() {
            let (consumed, sub) = parse_script(&input[i + 1..], to_subscript);
            if consumed > 0 {
                out.push_str(&sub);
                i += 1 + consumed;
                continue;
            }
        }

        // Handle LaTeX commands: \alpha, \beta, etc.
        if b == b'\\' {
            let (consumed, sym) = parse_latex_command(&input[i + 1..]);
            if consumed > 0 {
                out.push_str(sym);
                i += 1 + consumed;
                continue;
            }
        }

        // Pass through any other byte/char
        let ch_start = i;
        let ch = input[ch_start..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

/// Parses either a single character or a `{...}` group following ^/_, returning
/// (bytes-consumed, converted-string). Returns (0, _) if nothing parsed.
fn parse_script(rest: &str, mut converter: impl FnMut(char) -> Option<char>) -> (usize, String) {
    let bytes = rest.as_bytes();
    if bytes.is_empty() {
        return (0, String::new());
    }

    // Brace group: {abc}
    if bytes[0] == b'{' {
        if let Some(close) = rest[1..].find('}') {
            let group = &rest[1..1 + close];
            let mut converted = String::with_capacity(group.len());
            let mut all_ok = true;
            for ch in group.chars() {
                match converter(ch) {
                    Some(u) => converted.push(u),
                    None => {
                        all_ok = false;
                        break;
                    }
                }
            }
            if all_ok {
                // Consumed: { + group + }
                return (1 + close + 1, converted);
            }
        }
        return (0, String::new());
    }

    // Single character: x^2, x_1
    let ch = rest.chars().next().unwrap();
    if let Some(u) = converter(ch) {
        return (ch.len_utf8(), u.to_string());
    }
    (0, String::new())
}

/// Maps a digit/letter to its Unicode superscript (returns None if not supported).
fn to_superscript(c: char) -> Option<char> {
    match c {
        '0' => Some('⁰'),
        '1' => Some('¹'),
        '2' => Some('²'),
        '3' => Some('³'),
        '4' => Some('⁴'),
        '5' => Some('⁵'),
        '6' => Some('⁶'),
        '7' => Some('⁷'),
        '8' => Some('⁸'),
        '9' => Some('⁹'),
        '+' => Some('⁺'),
        '-' => Some('⁻'),
        '=' => Some('⁼'),
        '(' => Some('⁽'),
        ')' => Some('⁾'),
        'n' => Some('ⁿ'),
        'i' => Some('ⁱ'),
        _ => None,
    }
}

/// Maps a digit to its Unicode subscript (returns None if not supported).
fn to_subscript(c: char) -> Option<char> {
    match c {
        '0' => Some('₀'),
        '1' => Some('₁'),
        '2' => Some('₂'),
        '3' => Some('₃'),
        '4' => Some('₄'),
        '5' => Some('₅'),
        '6' => Some('₆'),
        '7' => Some('₇'),
        '8' => Some('₈'),
        '9' => Some('₉'),
        '+' => Some('₊'),
        '-' => Some('₋'),
        '=' => Some('₌'),
        '(' => Some('₍'),
        ')' => Some('₎'),
        _ => None,
    }
}

/// Parses a LaTeX command at start of `rest` (without leading backslash) and
/// returns (bytes-consumed, replacement-string). Returns (0, "") if no match.
fn parse_latex_command(rest: &str) -> (usize, &'static str) {
    // Sort by descending length so longer matches win (e.g., \subseteq before \subset).
    const COMMANDS: &[(&str, &str)] = &[
        // Greek lowercase
        ("alpha", "α"),
        ("beta", "β"),
        ("gamma", "γ"),
        ("delta", "δ"),
        ("epsilon", "ε"),
        ("zeta", "ζ"),
        ("eta", "η"),
        ("theta", "θ"),
        ("iota", "ι"),
        ("kappa", "κ"),
        ("lambda", "λ"),
        ("mu", "μ"),
        ("nu", "ν"),
        ("xi", "ξ"),
        ("pi", "π"),
        ("rho", "ρ"),
        ("sigma", "σ"),
        ("tau", "τ"),
        ("phi", "φ"),
        ("chi", "χ"),
        ("psi", "ψ"),
        ("omega", "ω"),
        // Greek uppercase
        ("Gamma", "Γ"),
        ("Delta", "Δ"),
        ("Theta", "Θ"),
        ("Lambda", "Λ"),
        ("Xi", "Ξ"),
        ("Pi", "Π"),
        ("Sigma", "Σ"),
        ("Phi", "Φ"),
        ("Psi", "Ψ"),
        ("Omega", "Ω"),
        // Operators / relations (longer first to avoid prefix collision)
        ("subseteq", "⊆"),
        ("supseteq", "⊇"),
        ("approx", "≈"),
        ("times", "×"),
        ("notin", "∉"),
        ("infty", "∞"),
        ("partial", "∂"),
        ("nabla", "∇"),
        ("subset", "⊂"),
        ("supset", "⊃"),
        ("forall", "∀"),
        ("exists", "∃"),
        ("emptyset", "∅"),
        ("cdot", "·"),
        ("div", "÷"),
        ("pm", "±"),
        ("mp", "∓"),
        ("leq", "≤"),
        ("geq", "≥"),
        ("neq", "≠"),
        ("equiv", "≡"),
        ("sim", "∼"),
        ("in", "∈"),
        ("cup", "∪"),
        ("cap", "∩"),
        ("to", "→"),
        ("rightarrow", "→"),
        ("leftarrow", "←"),
        ("Rightarrow", "⇒"),
        ("Leftarrow", "⇐"),
        ("sum", "∑"),
        ("prod", "∏"),
        ("int", "∫"),
    ];

    for (name, repl) in COMMANDS {
        if rest.starts_with(name) {
            // Ensure it's not a prefix of a longer identifier (e.g. \alphabet).
            let after = rest.as_bytes().get(name.len()).copied().unwrap_or(0);
            if !after.is_ascii_alphabetic() {
                return (name.len(), repl);
            }
        }
    }
    (0, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_when_no_math_syntax() {
        assert_eq!(math_to_unicode("hello world"), "hello world");
    }

    #[test]
    fn superscript_single_digit() {
        assert_eq!(math_to_unicode("x^2"), "x²");
        assert_eq!(math_to_unicode("E = mc^2"), "E = mc²");
    }

    #[test]
    fn superscript_brace_group() {
        assert_eq!(math_to_unicode("x^{10}"), "x¹⁰");
        assert_eq!(math_to_unicode("a^{2n}"), "a²ⁿ");
    }

    #[test]
    fn subscript_single_digit() {
        assert_eq!(math_to_unicode("H_2"), "H₂");
        assert_eq!(math_to_unicode("a_0 + a_1"), "a₀ + a₁");
    }

    #[test]
    fn subscript_brace_group() {
        assert_eq!(math_to_unicode("H_{2}O"), "H₂O");
    }

    #[test]
    fn greek_lowercase() {
        assert_eq!(math_to_unicode("\\alpha + \\beta"), "α + β");
        assert_eq!(math_to_unicode("\\theta"), "θ");
    }

    #[test]
    fn greek_uppercase() {
        assert_eq!(math_to_unicode("\\Sigma"), "Σ");
        assert_eq!(math_to_unicode("\\Delta x"), "Δ x");
    }

    #[test]
    fn operators_and_relations() {
        assert_eq!(math_to_unicode("a \\leq b"), "a ≤ b");
        assert_eq!(math_to_unicode("a \\neq b"), "a ≠ b");
        assert_eq!(math_to_unicode("3 \\times 4"), "3 × 4");
        assert_eq!(math_to_unicode("\\infty"), "∞");
    }

    #[test]
    fn longer_command_wins_over_prefix() {
        // \subseteq must match before \subset
        assert_eq!(math_to_unicode("A \\subseteq B"), "A ⊆ B");
        assert_eq!(math_to_unicode("A \\subset B"), "A ⊂ B");
    }

    #[test]
    fn unsupported_command_passes_through() {
        // \unsupported has no mapping, should pass through verbatim
        assert_eq!(math_to_unicode("\\unsupported"), "\\unsupported");
    }

    #[test]
    fn complex_expression_combination() {
        // Subscript brace `_{i=0}` is left verbatim because `i` is not in the
        // subscript map. Superscript `^{n}` converts because `n` is supported.
        // `x_i` stays unchanged because `i` is not in the subscript map.
        // Only `^2` and `\sum` and `^{n}` convert. Full LaTeX rendering is
        // deferred to KaTeX WebView (REQ-MV-010).
        assert_eq!(math_to_unicode("\\sum_{i=0}^{n} x_i^2"), "∑_{i=0}ⁿ x_i²");
    }

    #[test]
    fn empty_input() {
        assert_eq!(math_to_unicode(""), "");
    }

    #[test]
    fn caret_without_following_char_passes_through() {
        assert_eq!(math_to_unicode("a^"), "a^");
    }

    #[test]
    fn unsupported_superscript_char_passes_through() {
        // 'z' is not in the superscript map
        assert_eq!(math_to_unicode("x^z"), "x^z");
    }
}
