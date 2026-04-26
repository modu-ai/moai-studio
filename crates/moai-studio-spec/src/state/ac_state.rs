//! SPEC-V3-009 RG-SU-2 — AcState enum + AcRecord + AcSummary.
//!
//! REQ-SU-010: AcState 5 변형 정의.
//! REQ-SU-011: progress.md 의 두 패턴 인식.
//! REQ-SU-012: 명시 status 없으면 Pending default.
//! REQ-SU-013: UI 색상 매핑 (token 이름으로 노출).
//! REQ-SU-014: SpecRecord::ac_summary() 노출 → AcSummary.

use regex::Regex;

/// AC (Acceptance Criteria) 진행 상태 5 분류 (REQ-SU-010).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AcState {
    /// 미완료 / 상태 미기록 (default)
    Pending,
    /// 일부 완료
    Partial,
    /// 연기
    Deferred,
    /// 실패
    Fail,
    /// 완전 통과
    Full,
}

impl AcState {
    /// progress.md 의 status 라벨 → AcState 변환 (REQ-SU-011, REQ-SU-012).
    ///
    /// 인식 가능한 라벨: PASS/FULL, PARTIAL, DEFERRED/DEFER, FAIL.
    /// 그 외는 모두 `Pending` (REQ-SU-012).
    pub fn from_progress_label(label: &str) -> Self {
        match label.trim().to_uppercase().as_str() {
            "PASS" | "FULL" => AcState::Full,
            "PARTIAL" => AcState::Partial,
            "DEFERRED" | "DEFER" => AcState::Deferred,
            "FAIL" | "FAILED" => AcState::Fail,
            _ => AcState::Pending,
        }
    }

    /// 이 AcState 에 대응하는 design token 이름 (REQ-SU-013).
    ///
    /// 반환값은 token 이름 문자열 (실제 색상 적용은 UI 레이어 담당).
    pub fn token_name(&self) -> &'static str {
        match self {
            AcState::Full => "status.success",
            AcState::Partial => "status.warning",
            AcState::Deferred => "text.tertiary",
            AcState::Fail => "status.error",
            AcState::Pending => "status.info",
        }
    }
}

/// 단일 AC 기록.
#[derive(Debug, Clone, PartialEq)]
pub struct AcRecord {
    /// AC ID (예: "AC-SU-1")
    pub id: String,
    /// 현재 상태 (default: Pending)
    pub state: AcState,
    /// 원본 라벨 (progress.md 에서 읽은 그대로)
    pub raw_label: Option<String>,
}

impl AcRecord {
    /// 새 AcRecord 생성 (state = Pending).
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            state: AcState::Pending,
            raw_label: None,
        }
    }

    /// 상태 + 라벨 설정.
    pub fn with_label(mut self, label: &str) -> Self {
        self.state = AcState::from_progress_label(label);
        self.raw_label = Some(label.to_string());
        self
    }
}

/// AC 요약 카운트 (REQ-SU-014).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AcSummary {
    pub full: u32,
    pub partial: u32,
    pub deferred: u32,
    pub fail: u32,
    pub pending: u32,
}

impl AcSummary {
    /// `records` 로부터 AcSummary 를 계산한다.
    pub fn from_records(records: &[AcRecord]) -> Self {
        let mut s = AcSummary::default();
        for r in records {
            match r.state {
                AcState::Full => s.full += 1,
                AcState::Partial => s.partial += 1,
                AcState::Deferred => s.deferred += 1,
                AcState::Fail => s.fail += 1,
                AcState::Pending => s.pending += 1,
            }
        }
        s
    }

    /// UI 표시용 짧은 문자열 (예: "12/15 PASS, 2 PENDING, 1 FAIL").
    pub fn display(&self) -> String {
        let total = self.full + self.partial + self.deferred + self.fail + self.pending;
        format!(
            "{}/{} PASS, {} PENDING, {} FAIL",
            self.full, total, self.pending, self.fail
        )
    }
}

/// progress.md 본문에서 AC 상태를 파싱하여 `Vec<AcRecord>` 를 반환한다 (REQ-SU-011).
///
/// 두 패턴 인식:
/// - 라인 패턴: `AC-SU-3: PASS`
/// - 표 패턴: `| AC-SU-3 | ... | PASS |`
pub fn parse_ac_states_from_progress(text: &str) -> Vec<AcRecord> {
    let line_re = Regex::new(r"(?m)^(AC-[\w-]+):\s*(\w+)").expect("line_re must compile");
    let table_re = Regex::new(r"(?m)^\|\s*(AC-[\w-]+)\s*\|(?:[^|]*\|){0,8}\s*(\w+)\s*\|")
        .expect("table_re must compile");

    let mut records: Vec<AcRecord> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 라인 패턴 먼저
    for cap in line_re.captures_iter(text) {
        let id = cap[1].to_string();
        let label = cap[2].to_string();
        if seen.insert(id.clone()) {
            records.push(AcRecord::new(id).with_label(&label));
        }
    }

    // 표 패턴 (아직 발견되지 않은 ID 만)
    for cap in table_re.captures_iter(text) {
        let id = cap[1].trim().to_string();
        let label = cap[2].trim().to_string();
        // 표 헤더 행 제외
        if id.to_uppercase().starts_with("AC")
            && !id.to_uppercase().contains("ID")
            && seen.insert(id.clone())
        {
            records.push(AcRecord::new(id).with_label(&label));
        }
    }

    records
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AcState tests ──

    #[test]
    fn ac_state_from_pass() {
        assert_eq!(AcState::from_progress_label("PASS"), AcState::Full);
        assert_eq!(AcState::from_progress_label("pass"), AcState::Full);
        assert_eq!(AcState::from_progress_label("FULL"), AcState::Full);
    }

    #[test]
    fn ac_state_from_partial() {
        assert_eq!(AcState::from_progress_label("PARTIAL"), AcState::Partial);
    }

    #[test]
    fn ac_state_from_deferred() {
        assert_eq!(AcState::from_progress_label("DEFERRED"), AcState::Deferred);
        assert_eq!(AcState::from_progress_label("DEFER"), AcState::Deferred);
    }

    #[test]
    fn ac_state_from_fail() {
        assert_eq!(AcState::from_progress_label("FAIL"), AcState::Fail);
        assert_eq!(AcState::from_progress_label("FAILED"), AcState::Fail);
    }

    #[test]
    fn ac_state_unknown_is_pending() {
        assert_eq!(AcState::from_progress_label("UNKNOWN"), AcState::Pending);
        assert_eq!(AcState::from_progress_label(""), AcState::Pending);
        assert_eq!(
            AcState::from_progress_label("IN_PROGRESS"),
            AcState::Pending
        );
    }

    #[test]
    fn ac_state_ordering() {
        // Pending < Partial < Full など ordering 체크
        assert!(AcState::Pending < AcState::Full);
        assert!(AcState::Fail < AcState::Full);
    }

    #[test]
    fn ac_state_token_names() {
        assert_eq!(AcState::Full.token_name(), "status.success");
        assert_eq!(AcState::Partial.token_name(), "status.warning");
        assert_eq!(AcState::Deferred.token_name(), "text.tertiary");
        assert_eq!(AcState::Fail.token_name(), "status.error");
        assert_eq!(AcState::Pending.token_name(), "status.info");
    }

    // ── AcSummary tests ──

    #[test]
    fn ac_summary_counts() {
        let records = vec![
            AcRecord::new("AC-1").with_label("PASS"),
            AcRecord::new("AC-2").with_label("PASS"),
            AcRecord::new("AC-3").with_label("PARTIAL"),
            AcRecord::new("AC-4").with_label("FAIL"),
            AcRecord::new("AC-5"), // Pending
        ];
        let s = AcSummary::from_records(&records);
        assert_eq!(s.full, 2);
        assert_eq!(s.partial, 1);
        assert_eq!(s.fail, 1);
        assert_eq!(s.pending, 1);
        assert_eq!(s.deferred, 0);
    }

    #[test]
    fn ac_summary_display() {
        let records = vec![
            AcRecord::new("AC-1").with_label("PASS"),
            AcRecord::new("AC-2").with_label("FAIL"),
            AcRecord::new("AC-3"),
        ];
        let s = AcSummary::from_records(&records);
        let d = s.display();
        assert!(d.contains("1/3 PASS"), "display: {d}");
        assert!(d.contains("1 PENDING"), "display: {d}");
        assert!(d.contains("1 FAIL"), "display: {d}");
    }

    // ── parse_ac_states_from_progress tests ──

    #[test]
    fn parse_line_pattern() {
        let text = "AC-SU-1: PASS\nAC-SU-2: FAIL\nAC-SU-3: PARTIAL\n";
        let records = parse_ac_states_from_progress(text);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].state, AcState::Full);
        assert_eq!(records[1].state, AcState::Fail);
        assert_eq!(records[2].state, AcState::Partial);
    }

    #[test]
    fn parse_table_pattern() {
        let text = "| AC ID | 설명 | 상태 |\n|---|---|---|\n| AC-P-8 | FULL | `some_test` |\n";
        let records = parse_ac_states_from_progress(text);
        // 표 패턴: AC-P-8 인식
        assert!(records.iter().any(|r| r.id == "AC-P-8"));
    }

    #[test]
    fn dedup_line_pattern_takes_priority() {
        // 라인과 표 두 곳에 동일 ID → 라인 패턴 우선 (먼저 seen 에 등록)
        let text = "AC-X-1: PASS\n| AC-X-1 | desc | FAIL |\n";
        let records = parse_ac_states_from_progress(text);
        let r = records.iter().find(|r| r.id == "AC-X-1").unwrap();
        assert_eq!(r.state, AcState::Full);
    }

    #[test]
    fn all_five_states_recognized() {
        let text = "AC-1: PASS\nAC-2: PARTIAL\nAC-3: DEFERRED\nAC-4: FAIL\nAC-5: UNKNOWN\n";
        let records = parse_ac_states_from_progress(text);
        let states: Vec<_> = records.iter().map(|r| r.state).collect();
        assert!(states.contains(&AcState::Full));
        assert!(states.contains(&AcState::Partial));
        assert!(states.contains(&AcState::Deferred));
        assert!(states.contains(&AcState::Fail));
        assert!(states.contains(&AcState::Pending)); // UNKNOWN → Pending
    }
}
