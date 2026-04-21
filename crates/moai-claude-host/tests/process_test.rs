// moai-claude-host 프로세스 빌더 통합 테스트
// 실제 프로세스를 스폰하지 않음 — 커맨드 구성만 검증

use moai_claude_host::ClaudeProcessConfig;
use std::path::PathBuf;

/// 테스트용 기본 설정을 생성하는 헬퍼
fn make_config() -> ClaudeProcessConfig {
    ClaudeProcessConfig {
        claude_path: PathBuf::from("claude"),
        api_key: "test-api-key".to_string(),
        settings_path: None,
        mcp_config_path: None,
        plugin_dir: None,
        tools: vec!["Bash".to_string(), "Read".to_string()],
        permission_mode: "acceptEdits".to_string(),
        working_dir: PathBuf::from("/tmp"),
    }
}

/// --bare, --output-format stream-json, --tools, --permission-mode 등 필수 인수가 포함되었는지 검증
#[test]
fn test_build_command_has_required_args() {
    let config = make_config();
    let cmd = config.build_command();

    // tokio::process::Command → 내부 std::process::Command 접근
    let std_cmd = cmd.as_std();
    let args: Vec<&str> = std_cmd.get_args().map(|a| a.to_str().unwrap()).collect();

    assert!(args.contains(&"--bare"), "args에 --bare가 없음: {:?}", args);
    assert!(
        args.contains(&"--output-format"),
        "args에 --output-format이 없음: {:?}",
        args
    );
    assert!(
        args.contains(&"stream-json"),
        "args에 stream-json이 없음: {:?}",
        args
    );
    assert!(
        args.contains(&"--tools"),
        "args에 --tools가 없음: {:?}",
        args
    );
    assert!(
        args.contains(&"--permission-mode"),
        "args에 --permission-mode가 없음: {:?}",
        args
    );
    assert!(
        args.contains(&"--include-partial-messages"),
        "args에 --include-partial-messages가 없음: {:?}",
        args
    );
    assert!(
        args.contains(&"--verbose"),
        "args에 --verbose가 없음: {:?}",
        args
    );
}

/// CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0 환경 변수가 설정되어 있는지 검증
#[test]
fn test_build_command_has_env_scrub() {
    let config = make_config();
    let cmd = config.build_command();

    let std_cmd = cmd.as_std();
    let envs: Vec<(&std::ffi::OsStr, Option<&std::ffi::OsStr>)> = std_cmd.get_envs().collect();

    let found = envs.iter().any(|(key, val)| {
        key.to_str() == Some("CLAUDE_CODE_SUBPROCESS_ENV_SCRUB")
            && val.and_then(|v| v.to_str()) == Some("0")
    });

    assert!(
        found,
        "CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0이 환경 변수에 없음: {:?}",
        envs
    );
}

/// ANTHROPIC_API_KEY가 환경 변수에 포함되어 있는지 검증
#[test]
fn test_build_command_has_api_key() {
    let config = make_config();
    let cmd = config.build_command();

    let std_cmd = cmd.as_std();
    let envs: Vec<(&std::ffi::OsStr, Option<&std::ffi::OsStr>)> = std_cmd.get_envs().collect();

    let found = envs.iter().any(|(key, val)| {
        key.to_str() == Some("ANTHROPIC_API_KEY")
            && val.and_then(|v| v.to_str()) == Some("test-api-key")
    });

    assert!(found, "ANTHROPIC_API_KEY가 환경 변수에 없음: {:?}", envs);
}

/// --tools 플래그가 쉼표로 구분된 도구 목록과 함께 사용되는지 검증
/// --allowedTools 가 아닌 --tools 여야 함 (Errata E2)
#[test]
fn test_build_command_tools_flag() {
    let config = make_config(); // tools: ["Bash", "Read"]
    let cmd = config.build_command();

    let std_cmd = cmd.as_std();
    let args: Vec<&str> = std_cmd.get_args().map(|a| a.to_str().unwrap()).collect();

    // --allowedTools 가 아닌 --tools 사용
    assert!(
        !args.contains(&"--allowedTools"),
        "--allowedTools는 사용하면 안 됨 (Errata E2): {:?}",
        args
    );
    assert!(
        args.contains(&"--tools"),
        "args에 --tools가 없음: {:?}",
        args
    );

    // --tools 다음 값이 쉼표로 구분된 목록인지 확인
    let tools_idx = args.iter().position(|&a| a == "--tools").unwrap();
    let tools_value = args[tools_idx + 1];
    assert!(
        tools_value.contains(','),
        "--tools 값이 쉼표 구분 목록이 아님: {}",
        tools_value
    );
    assert!(
        tools_value.contains("Bash"),
        "--tools 값에 Bash가 없음: {}",
        tools_value
    );
    assert!(
        tools_value.contains("Read"),
        "--tools 값에 Read가 없음: {}",
        tools_value
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Task T-6 / T-7 테스트: ClaudeProcess 파이프 통신 및 에러 처리
// ─────────────────────────────────────────────────────────────────────────────

use moai_claude_host::{ClaudeProcess, ProcessError, SDKUserMessage};
use moai_stream_json::SDKMessage;
use tokio_stream::StreamExt as _;

/// T-6: SDKUserMessage가 올바른 JSON 구조로 직렬화되는지 검증
#[test]
fn test_sdk_user_message_serialization() {
    let msg = SDKUserMessage::new("hello world");
    let json = serde_json::to_string(&msg).expect("직렬화 실패");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON 파싱 실패");

    // 최상위 type 필드 검증
    assert_eq!(value["type"], "user", "type 필드가 'user'가 아님: {}", json);

    // message.role 검증
    assert_eq!(
        value["message"]["role"], "user",
        "message.role이 'user'가 아님: {}",
        json
    );

    // message.content[0].type 검증
    assert_eq!(
        value["message"]["content"][0]["type"], "text",
        "content[0].type이 'text'가 아님: {}",
        json
    );

    // message.content[0].text 검증
    assert_eq!(
        value["message"]["content"][0]["text"], "hello world",
        "content[0].text 불일치: {}",
        json
    );
}

/// T-7: api_key가 빈 문자열이면 ApiKeyMissing 에러를 반환해야 함
#[tokio::test]
async fn test_spawn_with_empty_api_key() {
    let config = ClaudeProcessConfig {
        claude_path: std::path::PathBuf::from("claude"),
        api_key: "".to_string(), // 빈 API 키
        settings_path: None,
        mcp_config_path: None,
        plugin_dir: None,
        tools: vec![],
        permission_mode: "acceptEdits".to_string(),
        working_dir: std::path::PathBuf::from("/tmp"),
    };

    let result = config.spawn().await;
    assert!(
        matches!(result, Err(ProcessError::ApiKeyMissing)),
        "빈 api_key에서 ApiKeyMissing이 아님: {:?}",
        result
    );
}

/// T-7: 존재하지 않는 바이너리 경로를 지정하면 에러를 반환해야 함
#[tokio::test]
async fn test_spawn_with_nonexistent_binary() {
    let config = ClaudeProcessConfig {
        claude_path: std::path::PathBuf::from("/nonexistent/claude"),
        api_key: "test-key".to_string(),
        settings_path: None,
        mcp_config_path: None,
        plugin_dir: None,
        tools: vec![],
        permission_mode: "acceptEdits".to_string(),
        working_dir: std::path::PathBuf::from("/tmp"),
    };

    let result = config.spawn().await;
    assert!(
        matches!(
            result,
            Err(ProcessError::ClaudeNotFound { .. }) | Err(ProcessError::SpawnFailed { .. })
        ),
        "존재하지 않는 바이너리에서 예상 에러가 아님: {:?}",
        result
    );
}

/// T-6: mock 서브프로세스(printf)가 NDJSON을 stdout으로 출력하면
///      message_stream()이 SDKMessage로 올바르게 디코딩해야 함
#[tokio::test]
async fn test_mock_subprocess_stdout_decoding() {
    // mock 출력: 실제 Claude CLI가 출력하는 형식의 NDJSON 픽스처
    let mock_ndjson = concat!(
        r#"{"type":"system","subtype":"init","session_id":"test-session","tools":[],"mcp_servers":[]}"#,
        "\n",
        r#"{"type":"result","subtype":"success","result":"done","cost_usd":0.0,"total_cost_usd":0.0,"duration_ms":0,"duration_api_ms":0}"#,
        "\n"
    );

    // printf로 NDJSON을 stdout에 출력하는 mock 프로세스 스폰
    let mut child = tokio::process::Command::new("printf")
        .arg(mock_ndjson)
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("printf 스폰 실패");

    let stdout = child.stdout.take().expect("stdout 없음");
    let stdin = child.stdin.take().expect("stdin 없음");

    // ClaudeProcess로 래핑하여 message_stream() 사용
    let process = ClaudeProcess::from_parts(child, stdout, stdin);
    let mut stream = process.message_stream();

    // 첫 번째 메시지: SystemInit
    let first = stream
        .next()
        .await
        .expect("첫 번째 메시지 없음")
        .expect("첫 번째 메시지 디코딩 실패");

    use moai_stream_json::SystemMessage;
    assert!(
        matches!(first, SDKMessage::System(SystemMessage::Init(_))),
        "첫 번째 메시지가 System::Init이 아님: {:?}",
        first
    );

    // 두 번째 메시지: Result::Success
    let second = stream
        .next()
        .await
        .expect("두 번째 메시지 없음")
        .expect("두 번째 메시지 디코딩 실패");

    use moai_stream_json::ResultMessage;
    assert!(
        matches!(second, SDKMessage::Result(ResultMessage::Success(_))),
        "두 번째 메시지가 Result::Success가 아님: {:?}",
        second
    );
}

/// T-7: wait()이 정상 종료 시 Ok(())를 반환해야 함
#[tokio::test]
async fn test_process_wait_success() {
    // true 명령은 exit code 0으로 즉시 종료
    let mut child = tokio::process::Command::new("true")
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("true 스폰 실패");

    let stdout = child.stdout.take().expect("stdout 없음");
    let stdin = child.stdin.take().expect("stdin 없음");

    let mut process = ClaudeProcess::from_parts(child, stdout, stdin);
    let result = process.wait().await;

    assert!(result.is_ok(), "정상 종료 시 Ok가 아님: {:?}", result);
}

/// T-7: wait()이 비정상 종료 시 ProcessCrashed를 반환해야 함
#[tokio::test]
async fn test_process_wait_crash() {
    // `false` 명령은 exit code 1로 즉시 종료
    let mut child = tokio::process::Command::new("false")
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("false 스폰 실패");

    let stdout = child.stdout.take().expect("stdout 없음");
    let stdin = child.stdin.take().expect("stdin 없음");

    let mut process = ClaudeProcess::from_parts(child, stdout, stdin);
    let result = process.wait().await;

    assert!(
        matches!(result, Err(ProcessError::ProcessCrashed { .. })),
        "비정상 종료 시 ProcessCrashed가 아님: {:?}",
        result
    );
}

/// mcp_config_path가 Some일 때 --mcp-config 인수가 포함되고,
/// None일 때 제외되는지 검증
#[test]
fn test_build_command_optional_mcp_config() {
    // Some 케이스: --mcp-config 포함
    let mut config = make_config();
    config.mcp_config_path = Some(PathBuf::from("/tmp/mcp.json"));
    let cmd = config.build_command();
    let std_cmd = cmd.as_std();
    let args: Vec<&str> = std_cmd.get_args().map(|a| a.to_str().unwrap()).collect();
    assert!(
        args.contains(&"--mcp-config"),
        "mcp_config_path가 Some일 때 --mcp-config가 없음: {:?}",
        args
    );

    // None 케이스: --mcp-config 제외
    let config_none = make_config();
    let cmd_none = config_none.build_command();
    let std_cmd_none = cmd_none.as_std();
    let args_none: Vec<&str> = std_cmd_none
        .get_args()
        .map(|a| a.to_str().unwrap())
        .collect();
    assert!(
        !args_none.contains(&"--mcp-config"),
        "mcp_config_path가 None일 때 --mcp-config가 있으면 안 됨: {:?}",
        args_none
    );
}
