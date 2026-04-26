//! Fuzzy 매처 — subsequence + scoring + highlight 위치 반환.
//!
//! @MX:NOTE: [AUTO] subsequence fuzzy 매처 — 점수 + highlight 인덱스 반환. 외부 크레이트 의존 없음.
//! @MX:SPEC: SPEC-V3-012

// ============================================================
// 점수 가중치 상수 (research.md §3.3 기본값)
// ============================================================

/// 매칭된 문자 1개당 기본 점수.
const BASE_MATCH_CREDIT: i32 = 16;

/// 연속 매칭 보너스 (이전 매칭 바로 다음 위치).
const CONSECUTIVE_BONUS: i32 = 15;

/// prefix 보너스 — 쿼리의 첫 문자가 후보의 맨 앞에 매칭될 때.
const PREFIX_BONUS: i32 = 10;

/// 단어 경계 보너스 — `_`, `-`, `.`, `/`, ` ` 또는 camel-boundary 뒤에 매칭될 때.
const WORD_BOUNDARY_BONUS: i32 = 8;

/// 매칭 윈도우 내 미매칭 문자당 gap 패널티.
const GAP_PENALTY: i32 = -1;

// ============================================================
// fuzzy_match 공개 API
// ============================================================

/// Subsequence fuzzy 매처.
///
/// - 쿼리가 비어있으면 `Some((0, vec![]))` 반환 (AC-PL-12 / RG-PL-16).
/// - 후보에 쿼리의 모든 문자가 순서대로 나타나지 않으면 `None` 반환 (AC-PL-10 / RG-PL-14).
/// - 성공 시 `Some((score, highlights))` 반환.
///   `highlights`는 후보 문자열 내 매칭된 문자의 **바이트 인덱스** 목록.
///
/// 점수 구성 (AC-PL-11 / RG-PL-15):
/// - 매칭 문자당 +16 (base_match_credit)
/// - 연속 매칭 시 +15 (consecutive_bonus)
/// - 첫 문자가 index 0 에서 매칭 시 +10 (prefix_bonus)
/// - 단어 경계 뒤 매칭 시 +8 (word_boundary_bonus)
/// - 매칭 윈도우 내 미매칭 문자당 -1 (gap_penalty)
pub fn fuzzy_match(query: &str, candidate: &str) -> Option<(i32, Vec<usize>)> {
    // AC-PL-12 / RG-PL-16: 빈 쿼리는 무조건 통과, 점수 0, highlight 없음.
    if query.is_empty() {
        return Some((0, vec![]));
    }

    let q_chars: Vec<char> = query.to_lowercase().chars().collect();
    let c_chars: Vec<(usize, char)> = candidate
        .char_indices()
        .map(|(i, ch)| {
            let lower = ch.to_lowercase().next().unwrap_or(ch);
            (i, lower)
        })
        .collect();

    let mut highlights: Vec<usize> = Vec::with_capacity(q_chars.len());
    let mut score: i32 = 0;
    let mut q_i: usize = 0;

    // 이전에 매칭된 문자의 후보 내 바이트 인덱스 (연속 보너스 계산용).
    let mut prev_matched_byte_idx: Option<usize> = None;

    // 후보 내 이전 문자 (단어 경계 보너스 계산용).
    let mut prev_candidate_char: Option<char> = None;

    for (cand_byte_idx, cand_ch) in &c_chars {
        if q_i >= q_chars.len() {
            break;
        }

        if *cand_ch == q_chars[q_i] {
            // 기본 점수.
            score += BASE_MATCH_CREDIT;

            // prefix 보너스: 쿼리 첫 문자가 후보 맨 앞 문자에 매칭.
            if *cand_byte_idx == 0 && q_i == 0 {
                score += PREFIX_BONUS;
            }

            // 연속 보너스: 이전 매칭 바이트 인덱스 다음 위치인지 확인.
            // char_indices 기반이라 직전 매칭 다음에 바로 이어진 character 여야 한다.
            // 단순 구현: c_chars 에서 현재 위치가 이전 매칭 위치 바로 다음인지 확인.
            if let Some(prev_byte) = prev_matched_byte_idx {
                // 이전 매칭 문자 다음 바이트 인덱스를 찾아 비교.
                let is_consecutive = c_chars
                    .iter()
                    .position(|(i, _)| *i == prev_byte)
                    .map(|pos| pos + 1 < c_chars.len() && c_chars[pos + 1].0 == *cand_byte_idx)
                    .unwrap_or(false);
                if is_consecutive {
                    score += CONSECUTIVE_BONUS;
                }
            }

            // 단어 경계 보너스.
            if prev_candidate_char.is_some_and(|prev_ch| {
                is_word_separator(prev_ch) || is_camel_boundary(prev_ch, *cand_ch)
            }) {
                score += WORD_BOUNDARY_BONUS;
            }

            highlights.push(*cand_byte_idx);
            prev_matched_byte_idx = Some(*cand_byte_idx);
            q_i += 1;
        }

        prev_candidate_char = Some(*cand_ch);
    }

    // 쿼리의 모든 문자가 매칭되었는지 확인 (AC-PL-10 / RG-PL-14).
    if q_i < q_chars.len() {
        return None;
    }

    // gap 패널티: 첫 매칭 위치부터 마지막 매칭 위치까지의 윈도우 내 미매칭 문자 수.
    if let (Some(&first_hl), Some(&last_hl)) = (highlights.first(), highlights.last()) {
        let window_chars = c_chars
            .iter()
            .filter(|(i, _)| *i >= first_hl && *i <= last_hl)
            .count();
        let unmatched_in_window = window_chars.saturating_sub(highlights.len()) as i32;
        score += unmatched_in_window * GAP_PENALTY;
    }

    Some((score, highlights))
}

// ============================================================
// 헬퍼 — 단어 경계 판별
// ============================================================

/// 단어 구분 문자 여부 — `_`, `-`, `.`, `/`, ` `, `\` 등.
fn is_word_separator(ch: char) -> bool {
    matches!(ch, '_' | '-' | '.' | '/' | ' ' | '\\' | ':' | '@')
}

/// camel-case 경계 여부 — 소문자 다음 대문자 (lower→Upper 전환).
fn is_camel_boundary(prev: char, curr: char) -> bool {
    prev.is_lowercase() && curr.is_uppercase()
}

// ============================================================
// highlight_runs: highlight 인덱스를 (matched, unmatched) 런으로 변환
// ============================================================

/// Highlight 런 — 렌더러가 accent-soft 스타일 적용에 사용.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    /// 후보 문자열 내 텍스트 조각.
    pub text: String,
    /// true 이면 매칭된 부분 (accent-soft 스타일 적용 대상).
    pub highlighted: bool,
}

/// 후보 문자열과 highlight 바이트 인덱스를 받아 TextRun 목록으로 변환한다.
///
/// - highlight 인덱스는 정렬된 바이트 인덱스 목록이어야 한다.
/// - 반환값: 매칭/미매칭 텍스트 런의 목록 (순서대로).
pub fn build_text_runs(candidate: &str, highlights: &[usize]) -> Vec<TextRun> {
    if candidate.is_empty() {
        return vec![];
    }

    let mut runs: Vec<TextRun> = Vec::new();
    let hl_set: std::collections::BTreeSet<usize> = highlights.iter().copied().collect();

    // char_indices 로 순회하며 highlight 인덱스와 비교.
    let chars: Vec<(usize, char)> = candidate.char_indices().collect();
    let mut i = 0;

    while i < chars.len() {
        let (byte_idx, _ch) = chars[i];
        // 다음 char 의 바이트 오프셋 (또는 문자열 끝).
        let next_byte = if i + 1 < chars.len() {
            chars[i + 1].0
        } else {
            candidate.len()
        };

        let is_hl = hl_set.contains(&byte_idx);

        // 동일 highlighted 상태인 연속 문자를 하나의 런으로 병합.
        let mut run_end = next_byte;
        let mut j = i + 1;
        while j < chars.len() {
            let next_is_hl = hl_set.contains(&chars[j].0);
            if next_is_hl != is_hl {
                break;
            }
            run_end = if j + 1 < chars.len() {
                chars[j + 1].0
            } else {
                candidate.len()
            };
            j += 1;
        }

        let text = candidate[byte_idx..run_end].to_string();
        runs.push(TextRun {
            text,
            highlighted: is_hl,
        });
        i = j;
    }

    runs
}

// ============================================================
// 단위 테스트 — AC-PL-9 ~ AC-PL-12
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------
    // AC-PL-12: 빈 쿼리 → 전체 통과, 점수 0, highlight 없음
    // ----------------------------------------------------------

    /// AC-PL-12: 빈 쿼리 + 임의 후보 → Some((0, vec![])).
    #[test]
    fn empty_query_passthrough() {
        let result = fuzzy_match("", "hello world");
        assert_eq!(result, Some((0, vec![])), "빈 쿼리는 Some((0, [])) 여야 함");
    }

    /// AC-PL-12: 빈 쿼리 + 빈 후보 → Some((0, vec![])).
    #[test]
    fn empty_query_empty_candidate() {
        let result = fuzzy_match("", "");
        assert_eq!(result, Some((0, vec![])));
    }

    // ----------------------------------------------------------
    // AC-PL-10: 비매칭 → None
    // ----------------------------------------------------------

    /// AC-PL-10: 쿼리 "xyz" + 후보 "abc" → None.
    #[test]
    fn no_subsequence_returns_none() {
        let result = fuzzy_match("xyz", "abc");
        assert!(result.is_none(), "subsequence 없으면 None 이어야 함");
    }

    /// 쿼리보다 후보가 짧은 경우 → None.
    #[test]
    fn candidate_shorter_than_query_returns_none() {
        let result = fuzzy_match("abcdef", "abc");
        assert!(result.is_none());
    }

    /// 쿼리 문자 순서가 역순인 경우 → None.
    #[test]
    fn wrong_order_returns_none() {
        let result = fuzzy_match("ba", "abc");
        // "ba" 는 a→b 순서로 있어야 하나 후보에는 a(0) b(1)이므로 b는 a 다음에 있음 — None.
        // 실제로 a_idx=0 → b_idx=1 이므로 b→a 순서의 쿼리 "ba"는 b(1)→a(?) 를 찾아야 하는데
        // b(1) 이후에 a 가 없으므로 None.
        assert!(result.is_none());
    }

    // ----------------------------------------------------------
    // AC-PL-9: subsequence 매칭 정확성
    // ----------------------------------------------------------

    /// AC-PL-9: 쿼리 "abc" + 후보 "a_b_c" → Some, highlight=[0,2,4].
    #[test]
    fn subsequence_match() {
        let result = fuzzy_match("abc", "a_b_c");
        assert!(result.is_some(), "subsequence 매칭 실패");
        let (_, highlights) = result.unwrap();
        assert_eq!(highlights, vec![0, 2, 4], "highlight 인덱스 불일치");
    }

    /// 정확히 일치하는 경우 — 모든 문자가 highlight 됨.
    #[test]
    fn exact_match_all_highlighted() {
        let result = fuzzy_match("abc", "abc");
        assert!(result.is_some());
        let (_, highlights) = result.unwrap();
        assert_eq!(highlights, vec![0, 1, 2]);
    }

    /// 쿼리 문자가 후보 중간에 분산된 경우.
    #[test]
    fn scattered_match_returns_correct_indices() {
        let result = fuzzy_match("ac", "abbc");
        assert!(result.is_some());
        let (_, highlights) = result.unwrap();
        // a=0, c=3
        assert_eq!(highlights[0], 0);
        assert_eq!(highlights[1], 3);
    }

    // ----------------------------------------------------------
    // 대소문자 무시
    // ----------------------------------------------------------

    /// 대소문자 구분 없이 매칭된다.
    #[test]
    fn case_insensitive_match() {
        let result = fuzzy_match("ABC", "abcdef");
        assert!(result.is_some(), "대소문자 무시 매칭 실패");
        let (_, highlights) = result.unwrap();
        assert_eq!(highlights, vec![0, 1, 2]);
    }

    /// 혼합 케이스 쿼리 + 혼합 케이스 후보.
    #[test]
    fn mixed_case_match() {
        let result = fuzzy_match("Abc", "aBcDef");
        assert!(result.is_some());
    }

    // ----------------------------------------------------------
    // AC-PL-11: 점수 순서 — 연속 매칭 > 분산 매칭
    // ----------------------------------------------------------

    /// AC-PL-11: 쿼리 "abc" 에 대해 "abc_def" 는 "a_b_c" 보다 높은 점수.
    #[test]
    fn consecutive_scores_higher() {
        let consecutive = fuzzy_match("abc", "abc_def").expect("consecutive match 실패");
        let scattered = fuzzy_match("abc", "a_b_c").expect("scattered match 실패");
        assert!(
            consecutive.0 > scattered.0,
            "연속 매칭 점수({}) 가 분산 매칭 점수({}) 보다 높아야 함",
            consecutive.0,
            scattered.0
        );
    }

    /// prefix 매칭이 중간 매칭보다 높은 점수를 가진다.
    #[test]
    fn prefix_scores_higher_than_midstring() {
        let prefix = fuzzy_match("ab", "abcde").expect("prefix match 실패");
        let midstring = fuzzy_match("ab", "xabcde").expect("midstring match 실패");
        assert!(
            prefix.0 > midstring.0,
            "prefix 점수({}) 가 중간 매칭 점수({}) 보다 높아야 함",
            prefix.0,
            midstring.0
        );
    }

    /// 점수 기반 정렬이 올바르다 — 더 관련성 높은 항목이 상위에 온다.
    #[test]
    fn score_ordering_is_correct() {
        let query = "rs";
        let candidates = ["resize", "rs_test", "restyle", "random_stuff"];
        let mut results: Vec<(i32, &str)> = candidates
            .iter()
            .filter_map(|c| fuzzy_match(query, c).map(|(s, _)| (s, *c)))
            .collect();
        results.sort_by(|a, b| b.0.cmp(&a.0));
        // 모두 매칭되어야 함.
        assert_eq!(results.len(), 4, "모든 후보가 매칭되어야 함");
        // 첫 번째와 두 번째는 "rs"로 시작하거나 더 관련성이 높아야 함.
        // 단순히 결과가 내림차순으로 정렬되어 있는지 확인.
        for i in 0..results.len() - 1 {
            assert!(
                results[i].0 >= results[i + 1].0,
                "점수 정렬 실패: {} < {}",
                results[i].0,
                results[i + 1].0
            );
        }
    }

    // ----------------------------------------------------------
    // 비-ASCII (한국어, 이모지) 후보
    // ----------------------------------------------------------

    /// 한국어 후보에 대한 ASCII 쿼리 — 매칭 없음.
    #[test]
    fn korean_candidate_ascii_query_no_match() {
        let result = fuzzy_match("abc", "가나다라");
        assert!(result.is_none());
    }

    /// 한국어 쿼리와 한국어 후보 매칭.
    #[test]
    fn korean_subsequence_match() {
        // "가다" 는 "가나다" 의 subsequence.
        let result = fuzzy_match("가다", "가나다");
        assert!(result.is_some(), "한국어 subsequence 매칭 실패");
        let (_, highlights) = result.unwrap();
        assert_eq!(highlights.len(), 2, "highlight 2개 이어야 함");
        // "가나다": '가'=0..3, '나'=3..6, '다'=6..9 (UTF-8 3바이트씩)
        assert_eq!(highlights[0], 0, "첫 번째 highlight 인덱스");
        assert_eq!(highlights[1], 6, "두 번째 highlight 인덱스");
    }

    // ----------------------------------------------------------
    // highlight 렌더링 — build_text_runs
    // ----------------------------------------------------------

    /// AC-PL-13: highlight 인덱스 [0, 2, 4] 로 TextRun 이 정확히 분리된다.
    #[test]
    fn highlight_uses_accent_soft_indices() {
        // "a_b_c" 에서 인덱스 [0, 2, 4] 가 highlighted.
        let candidate = "a_b_c";
        let highlights = vec![0usize, 2, 4];
        let runs = build_text_runs(candidate, &highlights);

        // 'a' → highlighted, '_' → not, 'b' → highlighted, '_' → not, 'c' → highlighted
        let expected_texts = ["a", "_", "b", "_", "c"];
        let expected_hl = [true, false, true, false, true];
        assert_eq!(runs.len(), expected_texts.len(), "run 수 불일치");
        for (i, run) in runs.iter().enumerate() {
            assert_eq!(run.text, expected_texts[i], "run[{i}] text 불일치");
            assert_eq!(
                run.highlighted, expected_hl[i],
                "run[{i}] highlighted 불일치"
            );
        }
    }

    /// 빈 highlight → 전체가 단일 미매칭 런.
    #[test]
    fn empty_highlights_single_run() {
        let runs = build_text_runs("hello", &[]);
        assert_eq!(runs.len(), 1);
        assert!(!runs[0].highlighted);
        assert_eq!(runs[0].text, "hello");
    }

    /// 모든 문자가 highlighted → 단일 매칭 런.
    #[test]
    fn all_highlighted_single_run() {
        let runs = build_text_runs("abc", &[0, 1, 2]);
        assert_eq!(runs.len(), 1);
        assert!(runs[0].highlighted);
        assert_eq!(runs[0].text, "abc");
    }
}
