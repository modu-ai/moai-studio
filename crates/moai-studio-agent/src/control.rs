//! Agent control envelope writer — pause / resume / kill (RG-AD-5, AC-AD-9/10)
//!
//! USER-DECISION-AD-C C1: stdin envelope `MOAI-CTRL: {json}\n` 라인 포맷.
//! agent process 의 stdin 에 한 줄 단위로 작성하여 prompt 와 명확히 구분한다.
//!
//! SPEC-V3-010 REQ-AD-024/025/026/027/029.

// @MX:ANCHOR: [AUTO] control-envelope-writer
// @MX:REASON: [AUTO] agent control IPC 단일 진입점. fan_in >= 3:
//   AgentControlBar UI, integration test, future supervisor crate.
//   SPEC: SPEC-V3-010 RG-AD-5, USER-DECISION-AD-C C1

use std::io::{self, Write};

use serde::{Deserialize, Serialize};

use crate::events::AgentRunId;

/// agent control 액션 (REQ-AD-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlAction {
    /// 일시정지 (REQ-AD-025)
    Pause,
    /// 재개 (REQ-AD-026)
    Resume,
    /// 강제 종료 (REQ-AD-027)
    Kill,
}

impl ControlAction {
    /// JSON 직렬화 시 사용되는 액션 식별자 ("pause" / "resume" / "kill").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Kill => "kill",
        }
    }
}

/// stdin 에 작성될 control envelope (REQ-AD-025/026/027).
///
/// 직렬화 결과 예시:
/// ```text
/// MOAI-CTRL: {"action":"pause","run_id":"run-abc"}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlEnvelope {
    /// 어떤 동작을 수행할지
    pub action: ControlAction,
    /// 대상 run 의 식별자 (REQ-AD-025/026/027 의 run_id 필드)
    pub run_id: AgentRunId,
}

impl ControlEnvelope {
    /// 새 envelope 를 만든다.
    pub fn new(action: ControlAction, run_id: AgentRunId) -> Self {
        Self { action, run_id }
    }

    /// pause envelope 단축 생성자 (REQ-AD-025).
    pub fn pause(run_id: AgentRunId) -> Self {
        Self::new(ControlAction::Pause, run_id)
    }

    /// resume envelope 단축 생성자 (REQ-AD-026).
    pub fn resume(run_id: AgentRunId) -> Self {
        Self::new(ControlAction::Resume, run_id)
    }

    /// kill envelope 단축 생성자 (REQ-AD-027).
    pub fn kill(run_id: AgentRunId) -> Self {
        Self::new(ControlAction::Kill, run_id)
    }

    /// envelope 를 한 줄 문자열로 직렬화한다 (newline 종결자 포함).
    ///
    /// 결과: `MOAI-CTRL: {"action":"...","run_id":"..."}\n`
    pub fn to_line(&self) -> String {
        // serde_json 직렬화는 무한 안전 — 자료형이 모두 직렬화 가능.
        let payload = serde_json::to_string(self).expect("ControlEnvelope 는 항상 직렬화 가능");
        format!("MOAI-CTRL: {}\n", payload)
    }
}

/// envelope 를 임의의 io::Write 핸들에 작성한다 (REQ-AD-025/026/027).
///
/// 호출자가 ChildStdin 또는 임의의 buffer 를 넘긴다.
/// 작성 직후 flush 까지 수행하여 envelope 가 즉시 전달되도록 한다.
pub fn write_envelope<W: Write>(writer: &mut W, envelope: &ControlEnvelope) -> io::Result<()> {
    let line = envelope.to_line();
    writer.write_all(line.as_bytes())?;
    writer.flush()?;
    Ok(())
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn run_id(s: &str) -> AgentRunId {
        AgentRunId(s.to_string())
    }

    /// AC-AD-9: pause envelope 가 정확한 prefix + JSON 으로 직렬화된다.
    #[test]
    fn pause_envelope_serialization() {
        let env = ControlEnvelope::pause(run_id("run-abc"));
        let line = env.to_line();

        assert!(line.starts_with("MOAI-CTRL: "), "prefix 누락: {}", line);
        assert!(line.ends_with('\n'), "newline 종결자 누락");
        assert!(line.contains(r#""action":"pause""#));
        assert!(line.contains(r#""run_id":"run-abc""#));
    }

    /// resume envelope 직렬화.
    #[test]
    fn resume_envelope_serialization() {
        let env = ControlEnvelope::resume(run_id("run-xyz"));
        let line = env.to_line();
        assert!(line.contains(r#""action":"resume""#));
        assert!(line.contains(r#""run_id":"run-xyz""#));
    }

    /// AC-AD-10: kill envelope 직렬화 (confirm dialog 는 UI 책임).
    #[test]
    fn kill_envelope_serialization() {
        let env = ControlEnvelope::kill(run_id("run-42"));
        let line = env.to_line();
        assert!(line.contains(r#""action":"kill""#));
        assert!(line.contains(r#""run_id":"run-42""#));
    }

    /// envelope 는 newline 으로 종결되어야 한다 (line-based protocol).
    #[test]
    fn envelope_writes_newline_terminator() {
        let env = ControlEnvelope::pause(run_id("r"));
        let line = env.to_line();
        let trailing = line.chars().last();
        assert_eq!(trailing, Some('\n'));
    }

    /// AC-AD-9: write_envelope 가 buffer 에 정확히 작성한다.
    #[test]
    fn write_envelope_to_buffer() {
        let env = ControlEnvelope::pause(run_id("r1"));
        let mut buf: Vec<u8> = Vec::new();
        write_envelope(&mut buf, &env).expect("write 실패");

        let written = String::from_utf8(buf).expect("UTF-8 디코드 실패");
        assert!(written.starts_with("MOAI-CTRL: "));
        assert!(written.ends_with('\n'));
        assert!(written.contains(r#""action":"pause""#));
    }

    /// 다중 envelope 작성 시 각 envelope 가 별도 라인이어야 한다.
    #[test]
    fn multiple_envelopes_are_separate_lines() {
        let mut buf: Vec<u8> = Vec::new();
        write_envelope(&mut buf, &ControlEnvelope::pause(run_id("r"))).unwrap();
        write_envelope(&mut buf, &ControlEnvelope::resume(run_id("r"))).unwrap();

        let written = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = written.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("pause"));
        assert!(lines[1].contains("resume"));
    }

    /// ControlAction::as_str 매핑이 정확해야 한다.
    #[test]
    fn action_as_str_matches_serde() {
        assert_eq!(ControlAction::Pause.as_str(), "pause");
        assert_eq!(ControlAction::Resume.as_str(), "resume");
        assert_eq!(ControlAction::Kill.as_str(), "kill");

        // serde rename_all = lowercase 와 일치하는지 라운드트립 검증
        let env = ControlEnvelope::kill(run_id("r"));
        let line = env.to_line();
        let json_part = line
            .strip_prefix("MOAI-CTRL: ")
            .and_then(|s| s.strip_suffix('\n'))
            .unwrap();
        let decoded: ControlEnvelope = serde_json::from_str(json_part).expect("역직렬화 실패");
        assert_eq!(decoded.action, ControlAction::Kill);
    }
}
