// @MX:NOTE: [AUTO] normalize-cross-platform
// @MX:SPEC: SPEC-V3-005 RG-FE-1 REQ-FE-004
// cross-platform path 정규화 단일 진입점.
// git2 / FsWatcher / display 가 모두 forward-slash 표준을 따르도록 보장.
// Windows: 백슬래시 → 슬래시 변환 (cfg-gated).

use std::path::Path;

/// 경로를 display 용 forward-slash 문자열로 정규화한다.
///
/// - macOS / Linux: 그대로 반환
/// - Windows: 백슬래시를 슬래시로 변환
/// - 후행 슬래시 제거
pub fn normalize_for_display(p: &Path) -> String {
    let s = p.to_string_lossy();

    // Windows 백슬래시 변환
    #[cfg(windows)]
    let s = s.replace('\\', "/");
    #[cfg(not(windows))]
    let s = s.into_owned();

    // 후행 슬래시 제거 (루트 "/" 는 유지)
    if s.len() > 1 && s.ends_with('/') {
        s.trim_end_matches('/').to_string()
    } else {
        s
    }
}

// ============================================================
// 단위 테스트 — AC-FE-3
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    #[cfg(not(windows))]
    fn normalize_unix_preserves_forward_slash() {
        // Unix: forward-slash 경로는 그대로 유지
        let result = normalize_for_display(Path::new("foo/bar/baz.rs"));
        assert_eq!(result, "foo/bar/baz.rs");
    }

    #[test]
    #[cfg(windows)]
    fn normalize_windows_converts_backslash() {
        // Windows: 백슬래시를 슬래시로 변환
        let result = normalize_for_display(Path::new(r"foo\bar\baz.rs"));
        assert_eq!(result, "foo/bar/baz.rs");
    }

    #[test]
    fn normalize_strips_trailing_slash() {
        // 후행 슬래시 제거
        let result = normalize_for_display(Path::new("foo/bar/"));
        assert_eq!(result, "foo/bar");
    }

    #[test]
    fn normalize_handles_root() {
        // 루트 경로 "/"는 그대로 유지
        let result = normalize_for_display(Path::new("/"));
        assert_eq!(result, "/");
    }
}
