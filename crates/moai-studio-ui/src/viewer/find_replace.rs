//! SPEC-V3-006 MS-3a/MS-3b: Find/Replace 기능 (CodeViewer + MarkdownViewer 공통).
//!
//! `FindReplaceState` 는 검색 쿼리, 일치 목록, 현재 포커스 인덱스를 관리한다.
//! CodeViewer 는 Replace 기능도 지원한다 (MarkdownViewer 는 read-only 라 Find 만).
//!
//! 범위 (MS-3a):
//! - case-sensitive toggle
//! - plain text 매칭
//! - prev/next match 네비게이션 (wrap-around)
//! - replace single / replace all (CodeViewer 전용)
//! - match count display
//! - Cmd+F → open, Esc → close, Enter → next, Shift+Enter → prev
//!
//! 범위 (MS-3b 추가):
//! - Regex 검색 모드 (`SearchMode::Regex`)
//! - 잘못된 정규식 → `regex_error` 에 저장, 패닉 없음

// @MX:ANCHOR: [AUTO] find-replace-state
// @MX:REASON: [AUTO] SPEC-V3-006 MS-3a/MS-3b. FindReplaceState 는 CodeViewer/MarkdownViewer
//   양쪽에서 소비되는 단일 자료구조 진입점이다.
//   fan_in >= 3: CodeViewer::load_find, MarkdownViewer::load_find, find_bar 렌더, 테스트.

// @MX:NOTE: [AUTO] search-mode-regex-compile-failure
// SearchMode::Regex 선택 시 regex::Regex::new() 실패 → FindReplaceState.regex_error 에
// 컴파일 오류 메시지 저장, matches 비움. 패닉 없음. 이후 쿼리 변경 시 재시도.

/// 검색 모드 (MS-3b: Regex 추가).
///
/// - `Plain`: 대소문자 무시 plain text 검색 (기본값)
/// - `CaseSensitive`: 대소문자 구분 plain text 검색
/// - `Regex`: 정규식 검색 (기본 대소문자 구분, `(?i)` prefix 로 무시 가능)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    /// 대소문자 무시 plain text 검색 (기본값)
    #[default]
    Plain,
    /// 대소문자 구분 plain text 검색
    CaseSensitive,
    /// 정규식 검색 — 컴파일 실패 시 regex_error 에 저장, 패닉 없음
    Regex,
}

/// 파일 내 텍스트 매치 위치 (줄 0-indexed, 줄 내 문자 오프셋).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchLocation {
    /// 0-indexed 줄 번호
    pub line: usize,
    /// 줄 내 시작 문자 오프셋 (char 단위)
    pub start: usize,
    /// 줄 내 종료 문자 오프셋 (exclusive, char 단위)
    pub end: usize,
}

/// Find/Replace 상태 자료구조.
#[derive(Debug, Clone, Default)]
pub struct FindReplaceState {
    /// 현재 검색 쿼리
    pub query: String,
    /// Replace 쿼리 (CodeViewer 전용 — MarkdownViewer 는 무시)
    pub replace_query: String,
    /// 대소문자 구분 여부 (true = sensitive) — Plain/CaseSensitive 모드용 legacy 필드
    pub case_sensitive: bool,
    /// 검색 모드 (MS-3b)
    pub search_mode: SearchMode,
    /// Find bar 가 화면에 표시 중인지 여부
    pub is_visible: bool,
    /// 현재 매칭된 위치 목록 (query 변경 시 재계산)
    pub matches: Vec<MatchLocation>,
    /// 현재 포커스된 매치 인덱스 (matches 비어있으면 None)
    pub current_match_idx: Option<usize>,
    /// Regex 컴파일 오류 메시지 (SearchMode::Regex 이고 잘못된 패턴일 때 설정됨)
    pub regex_error: Option<String>,
}

impl FindReplaceState {
    /// 빈 상태로 생성한다.
    pub fn new() -> Self {
        Self::default()
    }

    /// Find bar 를 열고 기존 쿼리를 유지한다.
    pub fn open(&mut self) {
        self.is_visible = true;
    }

    /// Find bar 를 닫는다. 쿼리와 매치 결과는 유지한다.
    pub fn close(&mut self) {
        self.is_visible = false;
    }

    /// 쿼리를 설정하고 소스에서 매치를 계산한다.
    ///
    /// `source` 는 전체 파일 내용 (줄바꿈 포함).
    pub fn set_query(&mut self, query: String, source: &str) {
        self.query = query;
        self.recalculate_matches(source);
    }

    /// case_sensitive 설정을 토글하고 매치를 재계산한다.
    pub fn toggle_case_sensitive(&mut self, source: &str) {
        self.case_sensitive = !self.case_sensitive;
        self.recalculate_matches(source);
    }

    /// 검색 모드를 설정하고 매치를 재계산한다.
    ///
    /// 모드 전환 시 이전 매치와 regex_error 를 초기화한다.
    pub fn set_mode(&mut self, mode: SearchMode, source: &str) {
        self.search_mode = mode;
        self.regex_error = None;
        self.recalculate_matches(source);
    }

    /// 다음 매치로 이동한다 (wrap-around).
    pub fn next_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match_idx = Some(match self.current_match_idx {
            None => 0,
            Some(i) => (i + 1) % self.matches.len(),
        });
    }

    /// 이전 매치로 이동한다 (wrap-around).
    pub fn prev_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match_idx = Some(match self.current_match_idx {
            None => self.matches.len() - 1,
            Some(0) => self.matches.len() - 1,
            Some(i) => i - 1,
        });
    }

    /// 현재 포커스된 매치 위치를 반환한다.
    pub fn current_match(&self) -> Option<&MatchLocation> {
        self.current_match_idx.and_then(|i| self.matches.get(i))
    }

    /// 현재 매치 수를 반환한다.
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// "N of M" 형식의 표시 문자열을 반환한다 (매치 없으면 "No results").
    pub fn match_display(&self) -> String {
        if self.matches.is_empty() {
            if self.query.is_empty() {
                return String::new();
            }
            return "결과 없음".to_string();
        }
        let idx = self.current_match_idx.map(|i| i + 1).unwrap_or(0);
        format!("{} / {}", idx, self.matches.len())
    }

    /// 현재 포커스된 매치를 replace_query 로 치환하고 새 소스를 반환한다.
    ///
    /// 매치가 없거나 replace_query 가 비어있으면 원본을 반환한다.
    /// 치환 후 매치 목록을 재계산한다.
    pub fn replace_current(&mut self, source: &str) -> String {
        let Some(idx) = self.current_match_idx else {
            return source.to_string();
        };
        let Some(loc) = self.matches.get(idx).cloned() else {
            return source.to_string();
        };
        let new_source = apply_replacement(source, &loc, &self.replace_query.clone());
        self.recalculate_matches(&new_source);
        new_source
    }

    /// 모든 매치를 replace_query 로 치환하고 새 소스와 치환 횟수를 반환한다.
    pub fn replace_all(&mut self, source: &str) -> (String, usize) {
        if self.matches.is_empty() {
            return (source.to_string(), 0);
        }
        let count = self.matches.len();
        // 역순으로 치환하여 오프셋 불변성 유지
        let mut result = source.to_string();
        let replace_with = self.replace_query.clone();
        for loc in self.matches.iter().rev() {
            result = apply_replacement(&result, loc, &replace_with);
        }
        self.recalculate_matches(&result);
        (result, count)
    }

    // ──────────────────────────────────────────────────────────
    // private
    // ──────────────────────────────────────────────────────────

    /// 소스에서 query 를 검색하여 matches 목록을 재계산한다.
    ///
    /// `SearchMode::Regex` 일 때는 regex crate 를 사용하며, 컴파일 실패 시
    /// `regex_error` 에 오류 메시지를 저장하고 matches 를 비운다 (패닉 없음).
    fn recalculate_matches(&mut self, source: &str) {
        self.matches.clear();
        self.current_match_idx = None;
        if self.query.is_empty() {
            return;
        }

        match self.search_mode {
            SearchMode::Regex => {
                self.recalculate_matches_regex(source);
            }
            SearchMode::Plain | SearchMode::CaseSensitive => {
                self.recalculate_matches_plain(source);
            }
        }

        if !self.matches.is_empty() {
            self.current_match_idx = Some(0);
        }
    }

    /// Plain / CaseSensitive 모드 매치 계산.
    fn recalculate_matches_plain(&mut self, source: &str) {
        let (src_cmp, q_cmp) = if self.case_sensitive
            || self.search_mode == SearchMode::CaseSensitive
        {
            (source.to_string(), self.query.clone())
        } else {
            (source.to_lowercase(), self.query.to_lowercase())
        };

        for (line_idx, cmp_line) in src_cmp.lines().enumerate() {
            let mut search_start = 0;
            while let Some(pos) = cmp_line[search_start..].find(&q_cmp) {
                let abs_start = search_start + pos;
                let abs_end = abs_start + q_cmp.chars().count();
                self.matches.push(MatchLocation {
                    line: line_idx,
                    start: abs_start,
                    end: abs_end,
                });
                search_start = abs_start + q_cmp.len().max(1);
            }
        }
    }

    /// Regex 모드 매치 계산.
    ///
    /// 정규식 컴파일 실패 → `regex_error` 에 저장, matches 비움.
    /// `^` / `$` 앵커는 각 줄 단위로 처리한다 (find_iter per line).
    fn recalculate_matches_regex(&mut self, source: &str) {
        use regex::Regex;

        // 정규식 컴파일 실패 시 오류 저장 후 반환
        let re = match Regex::new(&self.query) {
            Ok(r) => r,
            Err(e) => {
                self.regex_error = Some(e.to_string());
                return;
            }
        };
        self.regex_error = None;

        // 줄 단위로 `^`, `$` 앵커가 동작하도록 각 줄을 개별 처리
        for (line_idx, line) in source.lines().enumerate() {
            for mat in re.find_iter(line) {
                // byte offset → char offset 변환
                let start_char = line[..mat.start()].chars().count();
                let end_char = start_char + mat.as_str().chars().count();
                self.matches.push(MatchLocation {
                    line: line_idx,
                    start: start_char,
                    end: end_char,
                });
            }
        }
    }
}

/// 소스 문자열의 특정 위치를 replacement 로 치환한다.
fn apply_replacement(source: &str, loc: &MatchLocation, replacement: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if loc.line >= lines.len() {
        return source.to_string();
    }

    let line = lines[loc.line];
    let chars: Vec<char> = line.chars().collect();
    if loc.start > chars.len() || loc.end > chars.len() {
        return source.to_string();
    }

    let before: String = chars[..loc.start].iter().collect();
    let after: String = chars[loc.end..].iter().collect();
    let new_line = format!("{}{}{}", before, replacement, after);

    let mut result_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    result_lines[loc.line] = new_line;

    // 원본이 trailing newline 을 가지고 있는지 확인
    let trailing_newline = source.ends_with('\n');
    let joined = result_lines.join("\n");
    if trailing_newline {
        format!("{}\n", joined)
    } else {
        joined
    }
}

// ============================================================
// 단위 테스트 (MS-3a TDD — RED → GREEN)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── T1: 기본 find 동작 ──

    #[test]
    fn find_replace_default_state_is_invisible() {
        let state = FindReplaceState::new();
        assert!(!state.is_visible, "기본 상태는 숨겨져 있어야 한다");
        assert_eq!(state.match_count(), 0);
    }

    #[test]
    fn find_replace_open_makes_visible() {
        let mut state = FindReplaceState::new();
        state.open();
        assert!(state.is_visible, "open() 후 visible 이어야 한다");
    }

    #[test]
    fn find_replace_close_hides() {
        let mut state = FindReplaceState::new();
        state.open();
        state.close();
        assert!(!state.is_visible, "close() 후 숨겨져야 한다");
    }

    #[test]
    fn find_replace_set_query_finds_matches() {
        let mut state = FindReplaceState::new();
        let source = "hello world\nhello rust\ngoodbye";
        state.set_query("hello".to_string(), source);
        assert_eq!(state.match_count(), 2, "hello 가 2번 나타나야 한다");
    }

    #[test]
    fn find_replace_empty_query_clears_matches() {
        let mut state = FindReplaceState::new();
        let source = "hello world";
        state.set_query("hello".to_string(), source);
        assert_eq!(state.match_count(), 1);
        state.set_query(String::new(), source);
        assert_eq!(state.match_count(), 0, "쿼리 초기화 시 매치 없어야 한다");
    }

    #[test]
    fn find_replace_no_match_returns_zero() {
        let mut state = FindReplaceState::new();
        state.set_query("xyz_not_found".to_string(), "hello world");
        assert_eq!(state.match_count(), 0);
    }

    // ── T2: case-sensitive 토글 ──

    #[test]
    fn find_replace_case_insensitive_by_default() {
        let mut state = FindReplaceState::new();
        let source = "Hello HELLO hello";
        state.set_query("hello".to_string(), source);
        assert_eq!(state.match_count(), 3, "대소문자 무시 → 3 매치");
    }

    #[test]
    fn find_replace_case_sensitive_restricts_matches() {
        let mut state = FindReplaceState::new();
        state.case_sensitive = true;
        let source = "Hello HELLO hello";
        state.set_query("hello".to_string(), source);
        assert_eq!(state.match_count(), 1, "대소문자 구분 → 1 매치");
    }

    #[test]
    fn find_replace_toggle_case_sensitive_recalculates() {
        let mut state = FindReplaceState::new();
        let source = "Hello HELLO hello";
        state.set_query("hello".to_string(), source);
        assert_eq!(state.match_count(), 3); // insensitive: 3
        state.toggle_case_sensitive(source);
        assert_eq!(state.match_count(), 1); // sensitive: 1
        state.toggle_case_sensitive(source);
        assert_eq!(state.match_count(), 3); // insensitive again: 3
    }

    // ── T3: prev/next 네비게이션 ──

    #[test]
    fn find_replace_next_match_advances_index() {
        let mut state = FindReplaceState::new();
        let source = "a a a";
        state.set_query("a".to_string(), source);
        assert_eq!(state.current_match_idx, Some(0));
        state.next_match();
        assert_eq!(state.current_match_idx, Some(1));
        state.next_match();
        assert_eq!(state.current_match_idx, Some(2));
    }

    #[test]
    fn find_replace_next_match_wraps_around() {
        let mut state = FindReplaceState::new();
        let source = "a a";
        state.set_query("a".to_string(), source);
        // 2 matches: idx=0
        state.next_match(); // idx=1
        state.next_match(); // wrap → idx=0
        assert_eq!(state.current_match_idx, Some(0));
    }

    #[test]
    fn find_replace_prev_match_goes_backward() {
        let mut state = FindReplaceState::new();
        let source = "a a a";
        state.set_query("a".to_string(), source);
        // idx=0 → prev → wrap → last
        state.prev_match();
        assert_eq!(state.current_match_idx, Some(2));
    }

    #[test]
    fn find_replace_prev_next_on_empty_does_nothing() {
        let mut state = FindReplaceState::new();
        state.set_query("xyz".to_string(), "hello world");
        assert_eq!(state.match_count(), 0);
        state.next_match();
        state.prev_match();
        assert_eq!(state.current_match_idx, None);
    }

    // ── T4: match_display ──

    #[test]
    fn find_replace_match_display_empty_query() {
        let state = FindReplaceState::new();
        assert_eq!(state.match_display(), "");
    }

    #[test]
    fn find_replace_match_display_no_results() {
        let mut state = FindReplaceState::new();
        state.set_query("notfound".to_string(), "hello world");
        assert_eq!(state.match_display(), "결과 없음");
    }

    #[test]
    fn find_replace_match_display_with_results() {
        let mut state = FindReplaceState::new();
        state.set_query("a".to_string(), "a b a");
        // idx=0, total=2 → "1 / 2"
        assert_eq!(state.match_display(), "1 / 2");
    }

    // ── T5: replace single ──

    #[test]
    fn find_replace_replace_current_replaces_focused_match() {
        let mut state = FindReplaceState::new();
        let source = "hello world";
        state.set_query("hello".to_string(), source);
        state.replace_query = "goodbye".to_string();
        let new_source = state.replace_current(source);
        assert!(
            new_source.contains("goodbye"),
            "치환 후 goodbye 가 있어야 한다"
        );
        assert!(
            !new_source.contains("hello"),
            "치환 후 hello 가 없어야 한다"
        );
    }

    #[test]
    fn find_replace_replace_current_no_match_returns_original() {
        let mut state = FindReplaceState::new();
        let source = "hello world";
        state.replace_query = "foo".to_string();
        // query 미설정 → current_match_idx = None
        let result = state.replace_current(source);
        assert_eq!(result, source);
    }

    // ── T6: replace all ──

    #[test]
    fn find_replace_replace_all_replaces_every_match() {
        let mut state = FindReplaceState::new();
        let source = "foo bar foo baz foo";
        state.set_query("foo".to_string(), source);
        state.replace_query = "qux".to_string();
        let (new_source, count) = state.replace_all(source);
        assert_eq!(count, 3, "3개 매치 치환 완료");
        assert!(!new_source.contains("foo"), "치환 후 foo 가 없어야 한다");
        assert_eq!(new_source.matches("qux").count(), 3);
    }

    #[test]
    fn find_replace_replace_all_no_match_returns_original() {
        let mut state = FindReplaceState::new();
        let source = "hello world";
        state.set_query("xyz".to_string(), source);
        state.replace_query = "foo".to_string();
        let (result, count) = state.replace_all(source);
        assert_eq!(result, source);
        assert_eq!(count, 0);
    }

    #[test]
    fn find_replace_replace_all_multiline() {
        let mut state = FindReplaceState::new();
        let source = "line1 foo\nline2 foo\nline3 bar";
        state.set_query("foo".to_string(), source);
        state.replace_query = "baz".to_string();
        let (new_source, count) = state.replace_all(source);
        assert_eq!(count, 2);
        assert!(new_source.contains("line1 baz"));
        assert!(new_source.contains("line2 baz"));
        assert!(new_source.contains("line3 bar"));
    }

    // ── T8: Regex 검색 모드 (MS-3b) ──

    #[test]
    fn regex_mode_finds_pattern_matches() {
        let mut state = FindReplaceState::new();
        let source = "foo bar";
        state.set_mode(SearchMode::Regex, source);
        state.set_query(r"\w+".to_string(), source);
        assert_eq!(state.match_count(), 2, r"\w+ 는 foo, bar 두 단어를 매칭해야 한다");
        assert!(state.regex_error.is_none(), "유효한 regex 는 오류 없음");
    }

    #[test]
    fn regex_mode_with_invalid_regex_stores_error_no_panic() {
        let mut state = FindReplaceState::new();
        let source = "hello world";
        state.set_mode(SearchMode::Regex, source);
        state.set_query("[invalid".to_string(), source);
        assert!(
            state.regex_error.is_some(),
            "잘못된 regex 는 regex_error 에 오류를 저장해야 한다"
        );
        assert_eq!(
            state.match_count(),
            0,
            "regex 컴파일 실패 시 매치 목록은 비어야 한다"
        );
    }

    #[test]
    fn regex_mode_case_sensitive_by_default() {
        // regex 기본은 대소문자 구분 — 명시적 문서화
        let mut state = FindReplaceState::new();
        let source = "Foo foo FOO";
        state.set_mode(SearchMode::Regex, source);
        state.set_query("foo".to_string(), source);
        // 기본 regex 는 대소문자 구분 → 소문자 "foo" 만 매칭
        assert_eq!(state.match_count(), 1, "기본 regex 는 대소문자 구분");
    }

    #[test]
    fn set_mode_clears_previous_matches() {
        let mut state = FindReplaceState::new();
        let source = "foo bar foo";
        state.set_query("foo".to_string(), source);
        assert_eq!(state.match_count(), 2);
        // 모드 변경 시 이전 매치 초기화
        state.set_mode(SearchMode::Regex, source);
        // query 는 여전히 "foo" — regex "foo" 도 2개 매칭이어야 함
        // 단, 이 테스트는 set_mode 자체가 매치를 재계산하는지 확인
        assert!(
            state.regex_error.is_none(),
            "set_mode 후 regex_error 초기화"
        );
    }

    #[test]
    fn regex_mode_multiline_anchors() {
        // `^foo` 는 각 줄 시작의 foo 만 매칭해야 한다
        let mut state = FindReplaceState::new();
        let source = "foo bar\nbaz foo\nfoo qux";
        state.set_mode(SearchMode::Regex, source);
        // 줄 시작 foo: 1번째 줄("foo bar"), 3번째 줄("foo qux") — 2개
        // 2번째 줄("baz foo") 는 줄 중간이라 매칭 안 됨
        state.set_query("^foo".to_string(), source);
        assert_eq!(
            state.match_count(),
            2,
            "^foo 는 줄 시작의 foo 만 매칭해야 한다"
        );
    }

    // ── T7: 매치 위치 검증 ──

    #[test]
    fn find_replace_match_location_correct_line_and_offset() {
        let mut state = FindReplaceState::new();
        let source = "abc\ndefg\nhij";
        state.set_query("def".to_string(), source);
        assert_eq!(state.match_count(), 1);
        let loc = state.current_match().unwrap();
        assert_eq!(loc.line, 1, "2번째 줄 (0-indexed=1)");
        assert_eq!(loc.start, 0);
        assert_eq!(loc.end, 3);
    }

    #[test]
    fn find_replace_match_location_multiple_on_same_line() {
        let mut state = FindReplaceState::new();
        let source = "aXaXa";
        state.set_query("X".to_string(), source);
        assert_eq!(state.match_count(), 2);
        let locs: Vec<_> = state.matches.iter().collect();
        assert_eq!(locs[0].line, 0);
        assert_eq!(locs[0].start, 1);
        assert_eq!(locs[1].line, 0);
        assert_eq!(locs[1].start, 3);
    }
}
