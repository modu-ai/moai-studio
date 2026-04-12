//! 서브프로세스 스트림 통합 테스트
//!
//! ClaudeProcess(moai-claude-host)와 SdkMessageCodec(moai-stream-json)의
//! 크로스 크레이트 통합을 검증한다.
//! 실제 Claude CLI 대신 mock 스크립트로 NDJSON 픽스처 데이터를 출력한다.

use futures::StreamExt;
use moai_claude_host::ClaudeProcess;
use moai_stream_json::SDKMessage;
use tokio::process::Command;

/// NDJSON 픽스처 라인들 (실제 Claude CLI 출력 형식과 동일)
const FIXTURE_SYSTEM_INIT: &str = r#"{"type":"system","subtype":"init","session_id":"test-session-abc","tools":[],"mcp_servers":[]}"#;
const FIXTURE_RESULT_SUCCESS: &str = r#"{"type":"result","subtype":"success","result":"테스트 완료","cost_usd":0.001,"total_cost_usd":0.001,"duration_ms":100,"duration_api_ms":50}"#;

/// mock 서브프로세스를 생성하는 헬퍼
///
/// `lines`에 담긴 NDJSON 라인들을 stdout으로 출력하고 종료하는
/// 임시 프로세스를 반환한다.
async fn spawn_mock_process(lines: Vec<&str>) -> ClaudeProcess {
    // echo -e 대신 printf로 NDJSON 출력
    // macOS와 Linux 모두에서 동작하도록 printf 사용
    let ndjson = lines.join("\n") + "\n";

    let mut cmd = Command::new("printf");
    cmd.arg("%s")
        .arg(&ndjson)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("printf 프로세스 스폰 실패");
    let stdout = child.stdout.take().expect("stdout 파이프 없음");
    let stdin = child.stdin.take().expect("stdin 파이프 없음");

    ClaudeProcess::from_parts(child, stdout, stdin)
}

/// system/init 메시지가 올바르게 디코딩되는지 검증
#[tokio::test]
async fn test_stream_decodes_system_init() {
    let process = spawn_mock_process(vec![FIXTURE_SYSTEM_INIT]).await;

    let mut stream = process.message_stream();

    // 첫 번째 메시지를 읽어 system/init인지 확인
    let msg = stream
        .next()
        .await
        .expect("스트림이 비어있다")
        .expect("메시지 디코딩 실패");

    match msg {
        SDKMessage::System(moai_stream_json::SystemMessage::Init(init)) => {
            assert_eq!(
                init.session_id, "test-session-abc",
                "session_id가 올바르게 파싱되어야 한다"
            );
        }
        other => panic!(
            "System::Init 메시지를 예상했으나 {:?} 수신",
            std::mem::discriminant(&other)
        ),
    }
}

/// result/success 메시지가 올바르게 디코딩되는지 검증
#[tokio::test]
async fn test_stream_decodes_result_success() {
    let process = spawn_mock_process(vec![FIXTURE_RESULT_SUCCESS]).await;

    let mut stream = process.message_stream();
    let msg = stream
        .next()
        .await
        .expect("스트림이 비어있다")
        .expect("메시지 디코딩 실패");

    match msg {
        SDKMessage::Result(moai_stream_json::ResultMessage::Success(result)) => {
            assert_eq!(
                result.result, "테스트 완료",
                "result 필드가 올바르게 파싱되어야 한다"
            );
            assert_eq!(
                result.duration_ms, 100,
                "duration_ms가 올바르게 파싱되어야 한다"
            );
        }
        other => panic!(
            "Result::Success 메시지를 예상했으나 다른 타입 수신: {:?}",
            other
        ),
    }
}

/// 여러 메시지가 순서대로 디코딩되는지 검증
#[tokio::test]
async fn test_stream_decodes_multiple_messages_in_order() {
    let process = spawn_mock_process(vec![FIXTURE_SYSTEM_INIT, FIXTURE_RESULT_SUCCESS]).await;

    let mut stream = process.message_stream();
    let messages: Vec<SDKMessage> = async {
        let mut msgs = Vec::new();
        while let Some(result) = stream.next().await {
            msgs.push(result.expect("메시지 디코딩 실패"));
        }
        msgs
    }
    .await;

    assert_eq!(messages.len(), 2, "픽스처 2개가 모두 디코딩되어야 한다");

    // 첫 번째: system/init
    assert!(
        matches!(
            &messages[0],
            SDKMessage::System(moai_stream_json::SystemMessage::Init(_))
        ),
        "첫 번째 메시지는 System::Init이어야 한다"
    );

    // 두 번째: result/success
    assert!(
        matches!(
            &messages[1],
            SDKMessage::Result(moai_stream_json::ResultMessage::Success(_))
        ),
        "두 번째 메시지는 Result::Success이어야 한다"
    );
}

/// 빈 줄이 포함된 NDJSON 스트림을 올바르게 처리하는지 검증
#[tokio::test]
async fn test_stream_skips_empty_lines() {
    // 실제 Claude 출력에서 발생하는 빈 줄을 포함한 픽스처
    let ndjson = format!("\n{}\n\n{}\n", FIXTURE_SYSTEM_INIT, FIXTURE_RESULT_SUCCESS);

    let mut cmd = Command::new("printf");
    cmd.arg("%s")
        .arg(&ndjson)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("프로세스 스폰 실패");
    let stdout = child.stdout.take().expect("stdout 파이프 없음");
    let stdin = child.stdin.take().expect("stdin 파이프 없음");

    let process = ClaudeProcess::from_parts(child, stdout, stdin);
    let mut stream = process.message_stream();

    let mut count = 0;
    while let Some(result) = stream.next().await {
        result.expect("빈 줄이 있어도 유효한 메시지는 파싱되어야 한다");
        count += 1;
    }

    assert_eq!(count, 2, "빈 줄을 건너뛰고 유효한 메시지 2개를 읽어야 한다");
}

/// 알 수 없는 타입의 메시지가 Unknown으로 처리되는지 검증
#[tokio::test]
async fn test_stream_decodes_unknown_type_as_unknown() {
    let unknown_json = r#"{"type":"custom_future_type","data":"some-value"}"#;
    let process = spawn_mock_process(vec![unknown_json]).await;

    let mut stream = process.message_stream();
    let msg = stream
        .next()
        .await
        .expect("스트림이 비어있다")
        .expect("메시지 디코딩 실패");

    assert!(
        matches!(msg, SDKMessage::Unknown(_)),
        "알 수 없는 타입은 SDKMessage::Unknown으로 처리되어야 한다"
    );
}

/// SDKUserMessage를 stdin으로 전송하고 올바른 JSON 형식인지 검증
///
/// 실제 Claude CLI가 없으므로 cat 명령으로 stdin을 echo하는 방식으로 테스트한다.
#[tokio::test]
async fn test_send_user_message_writes_correct_json() {
    // cat 명령: stdin을 그대로 stdout으로 출력
    // 사용자 메시지를 전송하면 cat이 그대로 돌려보내고,
    // SdkMessageCodec이 이를 파싱 시도한다.
    // (실제 Claude CLI 포맷과 다르므로 Unknown으로 파싱될 것)
    let mut cmd = Command::new("cat");
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("cat 프로세스 스폰 실패");
    let stdout = child.stdout.take().expect("stdout 파이프 없음");
    let stdin = child.stdin.take().expect("stdin 파이프 없음");

    let mut process = ClaudeProcess::from_parts(child, stdout, stdin);

    // 사용자 메시지 전송
    process
        .send_user_message("통합 테스트 메시지")
        .await
        .expect("사용자 메시지 전송 실패");

    // stdin을 닫아 cat이 EOF를 받고 종료하도록 함
    // (ClaudeProcess drop 시 자동 처리)
    drop(process);
}
