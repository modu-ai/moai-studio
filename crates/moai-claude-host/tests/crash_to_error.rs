//! T-013: 서브프로세스가 비정상 종료하면 `ExitOutcome::Crashed` 로 분류되고,
//! 정상 종료 시 `ExitOutcome::Normal` 로 분류되는지 검증한다.
//!
//! 상위 레이어(RootSupervisor)는 이 결과를 받아 `WorkspaceStatus::Error` 로
//! 전이시킨다 — 해당 전이 자체는 moai-supervisor 테스트에서 다룬다.

use moai_claude_host::monitor::{ExitOutcome, wait_for_exit};
use moai_claude_host::{ClaudeProcess, SDKUserMessage};

fn mock_child(cmd: &str) -> ClaudeProcess {
    let mut child = tokio::process::Command::new(cmd)
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("mock spawn 실패");
    let stdout = child.stdout.take().unwrap();
    let stdin = child.stdin.take().unwrap();
    ClaudeProcess::from_parts(child, stdout, stdin)
}

#[tokio::test]
async fn crashing_subprocess_returns_crashed_outcome() {
    // `false` 는 항상 exit code 1 로 종료한다 → Crashed 로 분류되어야 한다.
    let proc = mock_child("false");
    let outcome = wait_for_exit(proc).await.expect("wait 실패");
    assert!(
        matches!(outcome, ExitOutcome::Crashed { exit_code: Some(1) }),
        "비정상 종료는 Crashed(1) 이어야 함: {outcome:?}"
    );
}

#[tokio::test]
async fn clean_exit_returns_normal_outcome() {
    let proc = mock_child("true");
    let outcome = wait_for_exit(proc).await.expect("wait 실패");
    assert!(
        matches!(outcome, ExitOutcome::Normal),
        "정상 종료는 Normal 이어야 함: {outcome:?}"
    );
}

#[test]
fn sdk_user_message_stdin_json_shape() {
    // T-013: stdin 메시지 JSON 형태 검증 (role=user + content[0].type=text)
    let m = SDKUserMessage::new("안녕");
    let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    assert_eq!(v["type"], "user");
    assert_eq!(v["message"]["role"], "user");
    assert_eq!(v["message"]["content"][0]["type"], "text");
    assert_eq!(v["message"]["content"][0]["text"], "안녕");
}
