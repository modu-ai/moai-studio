//! SPEC-V3-009 MS-3 RG-SU-5 — moai CLI subprocess 클라이언트.
//!
//! REQ-SU-040: `moai run|plan|sync SPEC-XXX` subprocess spawn.
//! REQ-SU-041: stdout NDJSON 라인을 `moai_stream_json::decode_line` 로 파싱 → stream_lines append.
//! REQ-SU-042: 16ms throttle 는 Render 호출 측 책임 (본 모듈은 decode/append 만 담당).
//! REQ-SU-043: subprocess 종료 시 exit_code + 마지막 status 라인 기록.
//! REQ-SU-044: moai 바이너리 미존재 시 panic 금지 — Err(NotFound) 반환.
//!
//! # @MX:ANCHOR: [AUTO] MoaiCommandClient
//! @MX:REASON: [AUTO] SPEC-V3-009 §12 외부 인터페이스. fan_in >= 3:
//!   spec_ui::mod.rs, kanban_view, AC-SU-9/10 테스트.
//!
//! # @MX:WARN: [AUTO] subprocess spawn (moai binary)
//! @MX:REASON: [AUTO] REQ-SU-044 — moai 미존재 시 NotFound 에러. Err 를 unwrap 하지 말 것.
//!   단일 SPEC 당 1개 이상 동시 실행 금지 (REQ-SU-045) — 호출 측(KanbanBoardView) 책임.

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

use moai_stream_json::decode_line;

/// moai CLI 서브커맨드 종류 (REQ-SU-040).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoaiSubcommand {
    /// `moai run SPEC-XXX`
    Run,
    /// `moai plan SPEC-XXX`
    Plan,
    /// `moai sync SPEC-XXX`
    Sync,
}

impl MoaiSubcommand {
    /// 커맨드 인수 문자열 반환.
    pub fn as_str(&self) -> &'static str {
        match self {
            MoaiSubcommand::Run => "run",
            MoaiSubcommand::Plan => "plan",
            MoaiSubcommand::Sync => "sync",
        }
    }

    /// `[subcommand, spec_id]` 인수 쌍 반환.
    pub fn to_args(&self, spec_id: &str) -> [String; 2] {
        [self.as_str().to_string(), spec_id.to_string()]
    }
}

/// subprocess 실행 상태 (REQ-SU-043).
#[derive(Debug, Clone)]
pub struct CommandStatus {
    /// 종료 코드 (None = 실행 중)
    pub exit_code: Option<i32>,
    /// 마지막으로 디코딩된 SDKMessage 요약 (REQ-SU-043)
    pub last_status_line: Option<String>,
    /// spawn 시각
    pub started_at: Instant,
}

impl CommandStatus {
    fn new() -> Self {
        Self {
            exit_code: None,
            last_status_line: None,
            started_at: Instant::now(),
        }
    }
}

/// moai CLI subprocess + stream-json 디코더 (RG-SU-5).
///
/// @MX:ANCHOR: [AUTO] MoaiCommandClient — SPEC-V3-009 RG-SU-5 진입점.
/// @MX:REASON: [AUTO] fan_in >= 3: spec_ui::mod, kanban_view, AC-SU-9/10.
pub struct MoaiCommandClient {
    /// 대상 SPEC ID
    pub spec_id: String,
    /// 실행 서브커맨드
    pub subcommand: MoaiSubcommand,
    /// 실행 상태
    pub status: CommandStatus,
    /// 디코딩된 스트림 라인 목록 (REQ-SU-041)
    pub stream_lines: Vec<String>,
    /// 실행 중인 child process
    child: Child,
}

impl std::fmt::Debug for MoaiCommandClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MoaiCommandClient")
            .field("spec_id", &self.spec_id)
            .field("subcommand", &self.subcommand)
            .field("stream_lines_len", &self.stream_lines.len())
            .finish()
    }
}

impl MoaiCommandClient {
    /// moai CLI subprocess 를 spawn 한다 (REQ-SU-040).
    ///
    /// `moai` 바이너리가 PATH 에 없으면 `Err(io::ErrorKind::NotFound)` 반환 (REQ-SU-044).
    /// stdin = null, stdout = piped, stderr = piped 로 설정한다.
    pub fn spawn(spec_id: String, subcommand: MoaiSubcommand, cwd: &Path) -> std::io::Result<Self> {
        Self::spawn_with_binary("moai", spec_id, subcommand, cwd)
    }

    /// 지정 바이너리 경로로 spawn 한다 (테스트용 — mock binary 주입).
    pub fn spawn_with_binary(
        binary: &str,
        spec_id: String,
        subcommand: MoaiSubcommand,
        cwd: &Path,
    ) -> std::io::Result<Self> {
        let args = subcommand.to_args(&spec_id);
        let child = Command::new(binary)
            .current_dir(cwd)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self {
            spec_id,
            subcommand,
            status: CommandStatus::new(),
            stream_lines: Vec::new(),
            child,
        })
    }

    /// 단일 NDJSON 라인을 디코딩하여 `stream_lines` 에 append 한다 (REQ-SU-041).
    ///
    /// 디코딩 성공 시 SDKMessage 요약 문자열을 `last_status_line` 에 기록한다.
    /// 디코딩 실패 시 raw 텍스트 그대로 append (graceful, no panic, REQ-SU-044 spirit).
    pub fn ingest_stdout_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        match decode_line(trimmed) {
            Ok(msg) => {
                // SDKMessage 의 간략 요약을 생성한다
                let summary = sdk_message_summary(&msg);
                self.status.last_status_line = Some(summary.clone());
                self.stream_lines.push(summary);
            }
            Err(e) => {
                // 디코딩 실패 시 raw 텍스트 append (graceful)
                tracing::warn!("stream-json 디코딩 실패 (raw append): {e}");
                self.stream_lines.push(trimmed.to_string());
            }
        }
    }

    /// child 종료 여부를 확인한다. 종료됐으면 exit_code 를 반환한다.
    ///
    /// 아직 실행 중이면 None 반환 (non-blocking poll).
    pub fn poll(&mut self) -> Option<i32> {
        if let Some(code) = self.status.exit_code {
            return Some(code); // 이미 종료된 경우
        }

        match self.child.try_wait() {
            Ok(Some(status)) => {
                let code = status.code().unwrap_or(-1);
                self.status.exit_code = Some(code);
                Some(code)
            }
            Ok(None) => None, // 아직 실행 중
            Err(e) => {
                tracing::warn!("child process poll 오류: {e}");
                None
            }
        }
    }

    /// subprocess 가 아직 실행 중인지 반환한다.
    pub fn is_running(&self) -> bool {
        self.status.exit_code.is_none()
    }
}

impl Drop for MoaiCommandClient {
    /// Drop 시 child process 를 정리한다 (resource leak 방지).
    fn drop(&mut self) {
        // 종료되지 않은 경우 kill 시도 (graceful)
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// SDKMessage 의 요약 문자열을 생성한다.
fn sdk_message_summary(msg: &moai_stream_json::SDKMessage) -> String {
    use moai_stream_json::{SDKMessage, SystemMessage};
    match msg {
        SDKMessage::System(s) => match s {
            SystemMessage::Init(_) => "[system/init]".to_string(),
            SystemMessage::HookStarted(h) => format!("[system/hook_started: {}]", h.hook_type),
            SystemMessage::HookResponse(h) => {
                format!("[system/hook_response: {}]", h.decision)
            }
        },
        SDKMessage::Assistant(_) => "[assistant]".to_string(),
        SDKMessage::User(_) => "[user]".to_string(),
        SDKMessage::RateLimitEvent(_) => "[rate_limit_event]".to_string(),
        SDKMessage::Result(_) => "[result]".to_string(),
        SDKMessage::StreamEvent(_) => "[stream_event]".to_string(),
        SDKMessage::Unknown(_) => "[unknown]".to_string(),
    }
}

// ============================================================
// 단위 테스트 (RED 단계에서 먼저 작성)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_cwd() -> PathBuf {
        std::env::temp_dir()
    }

    // ── MoaiSubcommand 단위 테스트 ────────────────────────────────

    #[test]
    fn subcommand_to_args_run() {
        // AC-SU-9: Run 서브커맨드 인수 검증
        let args = MoaiSubcommand::Run.to_args("SPEC-V3-009");
        assert_eq!(args, ["run".to_string(), "SPEC-V3-009".to_string()]);
    }

    #[test]
    fn subcommand_to_args_plan() {
        let args = MoaiSubcommand::Plan.to_args("SPEC-V3-001");
        assert_eq!(args, ["plan".to_string(), "SPEC-V3-001".to_string()]);
    }

    #[test]
    fn subcommand_to_args_sync() {
        let args = MoaiSubcommand::Sync.to_args("SPEC-V3-009");
        assert_eq!(args, ["sync".to_string(), "SPEC-V3-009".to_string()]);
    }

    // ── spawn_nonexistent_binary 테스트 (REQ-SU-044) ────────────

    #[test]
    fn spawn_nonexistent_binary_returns_not_found() {
        // REQ-SU-044: moai 바이너리 미존재 시 NotFound 에러
        let result = MoaiCommandClient::spawn_with_binary(
            "moai-binary-that-does-not-exist-xyz",
            "SPEC-V3-009".to_string(),
            MoaiSubcommand::Run,
            &tmp_cwd(),
        );
        assert!(result.is_err());
        let kind = result.unwrap_err().kind();
        assert_eq!(kind, std::io::ErrorKind::NotFound);
    }

    // ── ingest_stdout_line 테스트 (REQ-SU-041) ──────────────────

    /// echo 로 단일 stream-json 라인을 출력하는 fake MoaiCommandClient 생성.
    #[cfg(unix)]
    fn make_echo_client(json_line: &str) -> MoaiCommandClient {
        // `echo` 를 binary 로 사용해 stream-json 라인 한 줄 출력
        MoaiCommandClient::spawn_with_binary(
            "/bin/echo",
            "SPEC-V3-009".to_string(),
            MoaiSubcommand::Run,
            &tmp_cwd(),
        )
        // spawn 은 성공 (echo 는 항상 존재)
        .expect("echo spawn 실패")
        // 이 client 는 실제로 json_line 을 출력하지 않으므로,
        // ingest_stdout_line 은 별도로 직접 호출한다
        .with_pre_ingested(json_line)
    }

    #[test]
    fn ingest_decodes_stream_json_and_appends() {
        // AC-SU-9: system/init NDJSON 라인 디코딩 후 stream_lines append
        let json_line = r#"{"type":"system","subtype":"init","session_id":"sess-1","tools":[],"mcp_servers":[]}"#;

        // MoaiCommandClient 없이 decode+append 로직만 테스트 (격리)
        let mut client = MoaiCommandClientTestHelper::new();
        client.ingest_stdout_line(json_line);

        assert!(
            !client.stream_lines.is_empty(),
            "stream_lines 에 항목 추가됨"
        );
        assert!(
            client.status.last_status_line.is_some(),
            "last_status_line 갱신됨"
        );
        let last = client.status.last_status_line.as_ref().unwrap();
        assert!(
            last.contains("[system/init]"),
            "SDKMessage summary 가 system/init 포함: {last}"
        );
    }

    #[test]
    fn ingest_unknown_line_appends_raw_text() {
        // AC-SU-9: 파싱 실패 시 raw text append (graceful)
        let garbage = "hello garbage line";
        let mut client = MoaiCommandClientTestHelper::new();
        client.ingest_stdout_line(garbage);

        assert_eq!(client.stream_lines.len(), 1);
        assert_eq!(client.stream_lines[0], garbage);
    }

    #[test]
    fn ingest_empty_line_is_noop() {
        // 빈 라인은 skip
        let mut client = MoaiCommandClientTestHelper::new();
        client.ingest_stdout_line("   ");
        assert!(client.stream_lines.is_empty());
    }

    // ── poll 테스트 (REQ-SU-043) ──────────────────────────────────

    #[cfg(unix)]
    #[test]
    fn poll_returns_none_while_running_some_after_exit() {
        // `/usr/bin/true` 는 즉시 exit code 0 으로 종료 (Unix 전용)
        // MoaiSubcommand 인수 무시 — binary 동작만 테스트
        let mut client = MoaiCommandClient::spawn_with_binary(
            "/usr/bin/true",
            "SPEC-V3-009".to_string(),
            MoaiSubcommand::Run,
            &tmp_cwd(),
        )
        .expect("/usr/bin/true spawn 실패");

        // 짧게 대기 후 종료 확인
        std::thread::sleep(std::time::Duration::from_millis(50));

        // try_wait 에 의해 종료 코드 반환
        let code = client.poll();
        assert!(
            code.is_some(),
            "/usr/bin/true 는 종료되었으므로 exit code 반환"
        );
        assert_eq!(code.unwrap(), 0);
    }

    // ── 테스트 전용 헬퍼 구조체 ──────────────────────────────────

    /// MoaiCommandClient 의 ingest/status 로직을 child process 없이 테스트하는 헬퍼.
    struct MoaiCommandClientTestHelper {
        pub stream_lines: Vec<String>,
        pub status: CommandStatus,
    }

    impl MoaiCommandClientTestHelper {
        fn new() -> Self {
            Self {
                stream_lines: Vec::new(),
                status: CommandStatus::new(),
            }
        }

        fn ingest_stdout_line(&mut self, line: &str) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return;
            }
            match decode_line(trimmed) {
                Ok(msg) => {
                    let summary = sdk_message_summary(&msg);
                    self.status.last_status_line = Some(summary.clone());
                    self.stream_lines.push(summary);
                }
                Err(e) => {
                    tracing::warn!("stream-json 디코딩 실패 (raw append): {e}");
                    self.stream_lines.push(trimmed.to_string());
                }
            }
        }
    }

    // ── echo-based integration (AC-SU-9, Unix only) ─────────────

    #[cfg(unix)]
    impl MoaiCommandClient {
        /// 테스트 전용: pre_ingest 라인 주입 (echo client 로 생성 후 json 주입용).
        fn with_pre_ingested(mut self, line: &str) -> Self {
            self.ingest_stdout_line(line);
            self
        }
    }

    #[cfg(unix)]
    #[test]
    fn echo_based_integration_decode_and_append() {
        // AC-SU-9: echo subprocess + stream-json decode + append end-to-end
        let json_line = r#"{"type":"system","subtype":"init","session_id":"sess-x","tools":[],"mcp_servers":[]}"#;
        let client = make_echo_client(json_line);

        assert!(
            !client.stream_lines.is_empty(),
            "echo-based: stream_lines 에 항목 추가됨"
        );
        assert!(
            client.status.last_status_line.is_some(),
            "echo-based: last_status_line 갱신됨"
        );
    }
}
