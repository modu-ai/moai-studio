use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::process::{ChildStdin, ChildStdout, Command};
use tokio_util::codec::FramedRead;

use moai_stream_json::{SdkMessageCodec, SdkMessageStream};
use serde::Serialize;

use crate::error::ProcessError;

/// Claude CLI 서브프로세스 실행을 위한 설정 구조체
///
/// `build_command()`를 호출하면 실제로 스폰(spawn) 가능한
/// `tokio::process::Command`를 반환한다.
/// 스폰 자체는 이 구조체의 책임이 아니다 (Task T-6에서 처리).
pub struct ClaudeProcessConfig {
    /// claude CLI 실행 파일 경로 (기본값: "claude")
    pub claude_path: PathBuf,

    /// Anthropic API 키 — 환경 변수로 주입됨 (Errata E3)
    pub api_key: String,

    /// Claude Code 설정 파일 경로 (선택적)
    pub settings_path: Option<PathBuf>,

    /// MCP 서버 설정 파일 경로 (선택적)
    pub mcp_config_path: Option<PathBuf>,

    /// 플러그인 디렉터리 경로 (선택적)
    pub plugin_dir: Option<PathBuf>,

    /// 허용할 도구 목록 — --tools 플래그로 쉼표 구분 전달 (Errata E2)
    pub tools: Vec<String>,

    /// 권한 모드 (기본값: "acceptEdits")
    pub permission_mode: String,

    /// 클로드가 실행될 작업 디렉터리
    pub working_dir: PathBuf,
}

// ─────────────────────────────────────────────────────────────────────────────
// SDKUserMessage: Claude CLI stdin으로 전송하는 사용자 메시지 JSON 형식
// ─────────────────────────────────────────────────────────────────────────────

/// stdin 전송용 텍스트 콘텐츠 블록
#[derive(Debug, Serialize)]
struct UserTextContent {
    #[serde(rename = "type")]
    content_type: &'static str,
    text: String,
}

/// stdin 전송용 message 필드
#[derive(Debug, Serialize)]
struct UserMessageBody {
    role: &'static str,
    content: Vec<UserTextContent>,
}

/// Claude CLI stdin으로 전송하는 사용자 메시지
///
/// JSON 형식:
/// ```json
/// {"type":"user","message":{"role":"user","content":[{"type":"text","text":"<text>"}]}}
/// ```
#[derive(Debug, Serialize)]
pub struct SDKUserMessage {
    #[serde(rename = "type")]
    msg_type: &'static str,
    message: UserMessageBody,
}

impl SDKUserMessage {
    /// 텍스트로 새 사용자 메시지를 생성한다.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            msg_type: "user",
            message: UserMessageBody {
                role: "user",
                content: vec![UserTextContent {
                    content_type: "text",
                    text: text.into(),
                }],
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ClaudeProcess: 스폰된 Claude 서브프로세스 핸들
// ─────────────────────────────────────────────────────────────────────────────

/// 스폰된 Claude CLI 서브프로세스 핸들
///
/// stdout을 NDJSON 스트림으로 래핑하고,
/// stdin을 통해 사용자 메시지를 전송할 수 있다.
pub struct ClaudeProcess {
    // @MX:NOTE: tokio::process::Child는 Debug를 구현하지 않으므로 수동 구현 필요
    /// 스폰된 자식 프로세스
    child: tokio::process::Child,
    /// stdout → NDJSON 스트림 변환을 위한 FramedRead
    stdout: ChildStdout,
    /// stdin 버퍼 라이터
    stdin: BufWriter<ChildStdin>,
}

/// tokio::process::Child는 Debug를 구현하지 않으므로 수동으로 구현한다.
impl std::fmt::Debug for ClaudeProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeProcess")
            .field("child_id", &self.child.id())
            .finish_non_exhaustive()
    }
}

impl ClaudeProcess {
    /// 테스트용: 이미 스폰된 Child와 분리된 stdout/stdin으로 ClaudeProcess를 생성한다.
    pub fn from_parts(
        child: tokio::process::Child,
        stdout: ChildStdout,
        stdin: ChildStdin,
    ) -> Self {
        Self {
            child,
            stdout,
            stdin: BufWriter::new(stdin),
        }
    }

    /// stdout을 SDKMessage 스트림으로 반환한다.
    ///
    /// `SdkMessageCodec`으로 NDJSON 라인을 `SDKMessage`로 디코딩한다.
    /// 호출 후 stdout의 소유권이 이동하므로 한 번만 호출할 수 있다.
    pub fn message_stream(self) -> SdkMessageStream<ChildStdout> {
        FramedRead::new(self.stdout, SdkMessageCodec::new())
    }

    /// stdin으로 사용자 메시지를 전송한다.
    ///
    /// `SDKUserMessage`를 JSON 직렬화하여 `\n`과 함께 stdin에 기록한다.
    pub async fn send_user_message(&mut self, text: &str) -> Result<(), ProcessError> {
        let msg = SDKUserMessage::new(text);
        let mut json = serde_json::to_string(&msg).map_err(|e| ProcessError::SpawnFailed {
            source: std::io::Error::other(e),
        })?;
        json.push('\n');
        self.stdin
            .write_all(json.as_bytes())
            .await
            .map_err(|e| ProcessError::SpawnFailed { source: e })?;
        self.stdin
            .flush()
            .await
            .map_err(|e| ProcessError::SpawnFailed { source: e })?;
        Ok(())
    }

    /// 서브프로세스 종료를 기다린다.
    ///
    /// 정상 종료 시 `Ok(())`를 반환하고,
    /// 비정상 종료 시 `ProcessError::ProcessCrashed`를 반환한다.
    pub async fn wait(&mut self) -> Result<(), ProcessError> {
        let status = self
            .child
            .wait()
            .await
            .map_err(|e| ProcessError::SpawnFailed { source: e })?;

        if status.success() {
            Ok(())
        } else {
            Err(ProcessError::ProcessCrashed {
                exit_code: status.code(),
            })
        }
    }

    /// 서브프로세스를 강제 종료한다.
    pub async fn shutdown(&mut self) {
        let _ = self.child.kill().await;
    }
}

impl ClaudeProcessConfig {
    /// 설정을 기반으로 `tokio::process::Command`를 구성하여 반환한다.
    ///
    /// 실제 프로세스 스폰은 수행하지 않으며, 인수·환경 변수·stdio 설정만 담당한다.
    pub fn build_command(&self) -> Command {
        let mut cmd = Command::new(&self.claude_path);

        // 도구 목록을 쉼표 구분 문자열로 변환
        let tools_csv = self.tools.join(",");

        // 필수 플래그 설정
        cmd.args([
            "--bare",
            "-p",
            "",
            "--output-format",
            "stream-json",
            "--include-partial-messages",
            "--verbose",
            "--permission-mode",
            &self.permission_mode,
            "--tools",
            &tools_csv,
        ]);

        // 선택적 플래그: --settings
        if let Some(ref settings) = self.settings_path {
            cmd.arg("--settings").arg(settings);
        }

        // 선택적 플래그: --mcp-config
        if let Some(ref mcp) = self.mcp_config_path {
            cmd.arg("--mcp-config").arg(mcp);
        }

        // 선택적 플래그: --plugin-dir
        if let Some(ref plugin) = self.plugin_dir {
            cmd.arg("--plugin-dir").arg(plugin);
        }

        // 환경 변수 설정
        // Errata E4: 환경 변수 스크러빙 비활성화
        cmd.env("CLAUDE_CODE_SUBPROCESS_ENV_SCRUB", "0");
        // Errata E3: API 키를 환경 변수로 주입
        cmd.env("ANTHROPIC_API_KEY", &self.api_key);

        // 작업 디렉터리 설정
        cmd.current_dir(&self.working_dir);

        // stdio를 모두 파이프로 설정 (스트리밍 I/O 용)
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        cmd
    }

    /// 설정을 기반으로 Claude CLI 서브프로세스를 스폰하고 `ClaudeProcess`를 반환한다.
    ///
    /// # 에러
    /// - `ProcessError::ApiKeyMissing`: api_key가 비어 있는 경우
    /// - `ProcessError::ClaudeNotFound`: claude_path가 존재하지 않는 경우
    /// - `ProcessError::SpawnFailed`: 프로세스 스폰 자체가 실패한 경우
    pub async fn spawn(self) -> Result<ClaudeProcess, ProcessError> {
        // T-7: API 키 빈값 가드
        if self.api_key.is_empty() {
            return Err(ProcessError::ApiKeyMissing);
        }

        // T-7: claude CLI 존재 여부 확인
        // 절대 경로인 경우 파일 존재 여부를 직접 확인하고,
        // 상대 경로/바이너리 이름인 경우 which를 사용하여 확인한다.
        let path_str = self.claude_path.to_string_lossy().to_string();
        let exists = if self.claude_path.is_absolute() {
            self.claude_path.exists()
        } else {
            // which 명령으로 PATH에서 바이너리를 찾는다
            std::process::Command::new("which")
                .arg(&path_str)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };

        if !exists {
            return Err(ProcessError::ClaudeNotFound { path: path_str });
        }

        // 커맨드를 빌드하고 스폰
        let mut cmd = self.build_command();
        let mut child = cmd.spawn().map_err(|e| ProcessError::SpawnFailed { source: e })?;

        let stdout = child.stdout.take().expect("stdout 파이프 없음 (build_command에서 설정됨)");
        let stdin = child.stdin.take().expect("stdin 파이프 없음 (build_command에서 설정됨)");

        Ok(ClaudeProcess::from_parts(child, stdout, stdin))
    }
}
