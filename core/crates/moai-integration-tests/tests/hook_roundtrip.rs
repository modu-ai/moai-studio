//! HTTP 훅 서버 라운드트립 통합 테스트
//!
//! moai-hook-http 서버가 훅 이벤트를 올바르게 처리하는지 검증한다.
//! 인증, PreToolUse 응답 형식, 응답에 hookEventName 미포함(Errata E6) 등을 테스트한다.

use moai_hook_http::HookServer;

/// 테스트용 고정 인증 토큰
const TEST_TOKEN: &str = "test-auth-token-1234";

/// 서버를 시작하고 (포트, JoinHandle) 반환하는 헬퍼
async fn start_hook_server() -> (u16, tokio::task::JoinHandle<()>) {
    let server = HookServer::new(TEST_TOKEN.to_string());
    server.start().await.expect("훅 서버 시작 실패")
}

/// PreToolUse 훅에 유효한 인증으로 POST 시 permissionDecision: "allow" 반환 검증
#[tokio::test]
async fn test_hook_pre_tool_use_allow_with_valid_auth() {
    let (port, _handle) = start_hook_server().await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hooks/PreToolUse", port);

    // Claude가 전송하는 훅 이벤트 본문 형식
    let body = serde_json::json!({
        "session_id": "test-session-001",
        "cwd": "/tmp/test",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "echo hello"},
        "tool_use_id": "tool-use-001"
    });

    let resp = client
        .post(&url)
        .header("X-Auth-Token", TEST_TOKEN)
        .json(&body)
        .send()
        .await
        .expect("PreToolUse 요청 실패");

    assert_eq!(
        resp.status(),
        200,
        "유효한 인증으로 PreToolUse 요청은 200이어야 한다"
    );

    let response_body: serde_json::Value = resp.json().await.expect("응답 JSON 파싱 실패");

    // permissionDecision이 "allow"인지 확인
    assert_eq!(
        response_body["hookSpecificOutput"]["permissionDecision"], "allow",
        "PreToolUse 응답의 permissionDecision은 'allow'여야 한다"
    );
}

/// 잘못된 인증 토큰으로 요청 시 401 반환 검증
#[tokio::test]
async fn test_hook_invalid_auth_returns_401() {
    let (port, _handle) = start_hook_server().await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hooks/PreToolUse", port);

    let body = serde_json::json!({
        "session_id": "test-session-002",
        "cwd": "/tmp/test",
        "hook_event_name": "PreToolUse",
        "tool_name": "Read",
        "tool_input": {},
        "tool_use_id": "tool-use-002"
    });

    let resp = client
        .post(&url)
        .header("X-Auth-Token", "wrong-token-abcdef")
        .json(&body)
        .send()
        .await
        .expect("잘못된 토큰으로 요청 실패");

    assert_eq!(
        resp.status(),
        401,
        "잘못된 인증 토큰은 401 Unauthorized를 반환해야 한다"
    );
}

/// 인증 토큰이 없는 요청 시 401 반환 검증
#[tokio::test]
async fn test_hook_missing_auth_returns_401() {
    let (port, _handle) = start_hook_server().await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hooks/PreToolUse", port);

    let body = serde_json::json!({
        "session_id": "test-session-003",
        "cwd": "/tmp/test",
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {},
        "tool_use_id": "tool-use-003"
    });

    // X-Auth-Token 헤더 없이 요청
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .expect("헤더 없는 요청 실패");

    assert_eq!(
        resp.status(),
        401,
        "인증 토큰이 없는 요청은 401 Unauthorized를 반환해야 한다"
    );
}

/// 응답에 hookEventName 필드가 포함되지 않는지 검증 (Errata E6)
#[tokio::test]
async fn test_hook_response_does_not_contain_hook_event_name() {
    let (port, _handle) = start_hook_server().await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hooks/PreToolUse", port);

    let body = serde_json::json!({
        "session_id": "test-session-004",
        "cwd": "/tmp/test",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"},
        "tool_use_id": "tool-use-004"
    });

    let resp = client
        .post(&url)
        .header("X-Auth-Token", TEST_TOKEN)
        .json(&body)
        .send()
        .await
        .expect("요청 실패");

    assert_eq!(resp.status(), 200);

    let response_body: serde_json::Value = resp.json().await.expect("응답 JSON 파싱 실패");

    // Errata E6: 응답에 hookEventName 필드가 없어야 한다
    assert!(
        response_body.get("hookEventName").is_none(),
        "응답에 hookEventName 필드가 포함되면 안 된다 (Errata E6). 실제 응답: {:?}",
        response_body
    );
}

/// PostToolUse 이벤트는 hookSpecificOutput 없이 응답하는지 검증
#[tokio::test]
async fn test_hook_post_tool_use_returns_empty_output() {
    let (port, _handle) = start_hook_server().await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hooks/PostToolUse", port);

    let body = serde_json::json!({
        "session_id": "test-session-005",
        "cwd": "/tmp/test",
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "echo done"},
        "tool_use_id": "tool-use-005"
    });

    let resp = client
        .post(&url)
        .header("X-Auth-Token", TEST_TOKEN)
        .json(&body)
        .send()
        .await
        .expect("PostToolUse 요청 실패");

    assert_eq!(resp.status(), 200, "PostToolUse 응답은 200이어야 한다");

    let response_body: serde_json::Value = resp.json().await.expect("응답 JSON 파싱 실패");

    // PostToolUse는 hookSpecificOutput이 null이어야 한다
    assert!(
        response_body["hookSpecificOutput"].is_null(),
        "PostToolUse 응답의 hookSpecificOutput은 null이어야 한다. 실제: {:?}",
        response_body
    );
}

/// 두 서버 인스턴스가 서로 다른 포트에서 독립적으로 동작하는지 검증
#[tokio::test]
async fn test_hook_server_port_isolation() {
    let server1 = HookServer::new("token-a".to_string());
    let server2 = HookServer::new("token-b".to_string());

    let (port1, _h1) = server1.start().await.expect("서버1 시작 실패");
    let (port2, _h2) = server2.start().await.expect("서버2 시작 실패");

    assert_ne!(port1, port2, "두 서버는 서로 다른 포트를 사용해야 한다");

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "session_id": "s",
        "cwd": "/",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {},
        "tool_use_id": "t"
    });

    // 서버1은 token-a로만 접근 가능
    let resp1 = client
        .post(format!("http://127.0.0.1:{}/hooks/PreToolUse", port1))
        .header("X-Auth-Token", "token-a")
        .json(&body)
        .send()
        .await
        .expect("서버1 요청 실패");
    assert_eq!(resp1.status(), 200, "서버1에 올바른 토큰으로 요청 시 200");

    // 서버1에 token-b로 요청하면 401
    let resp1_wrong = client
        .post(format!("http://127.0.0.1:{}/hooks/PreToolUse", port1))
        .header("X-Auth-Token", "token-b")
        .json(&body)
        .send()
        .await
        .expect("서버1 잘못된 토큰 요청 실패");
    assert_eq!(
        resp1_wrong.status(),
        401,
        "서버1에 잘못된 토큰으로 요청 시 401"
    );
}
