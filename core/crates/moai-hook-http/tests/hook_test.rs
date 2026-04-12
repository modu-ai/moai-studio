//! moai-hook-http 통합 테스트
//! TDD RED 단계: 구현 전 실패하는 테스트를 먼저 작성

use moai_hook_http::{
    HookEventRequest, HookResponse, HookSpecificOutput, HookServer,
};

// ─── test 1: PreToolUse JSON 역직렬화 ───────────────────────────────────────

/// Claude 가 전송하는 PreToolUse 이벤트를 올바르게 파싱하는지 검증
#[test]
fn test_hook_event_deserialization() {
    let json = r#"{
        "session_id": "sess-abc123",
        "cwd": "/Users/user/project",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls -la"},
        "tool_use_id": "tool-xyz"
    }"#;

    let event: HookEventRequest = serde_json::from_str(json).expect("역직렬화 실패");

    assert_eq!(event.session_id, "sess-abc123");
    assert_eq!(event.cwd, "/Users/user/project");
    assert_eq!(event.hook_event_name, "PreToolUse");
    assert_eq!(event.tool_name.as_deref(), Some("Bash"));
    assert!(event.tool_input.is_some());
    assert_eq!(event.tool_use_id.as_deref(), Some("tool-xyz"));
}

// ─── test 2: Errata E6 — hookEventName 필드 포함 금지 ────────────────────────

/// HookResponse 직렬화 시 hookEventName 필드가 절대 포함되지 않아야 함 (Errata E6)
#[test]
fn test_hook_response_no_event_name() {
    let response = HookResponse {
        hook_specific_output: Some(HookSpecificOutput {
            permission_decision: "allow".to_string(),
            updated_input: None,
        }),
    };

    let json_str = serde_json::to_string(&response).expect("직렬화 실패");
    let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // hookSpecificOutput 내부에도 hookEventName 없어야 함
    assert!(
        json_val.get("hookEventName").is_none(),
        "최상위에 hookEventName 이 있으면 안 됨"
    );
    if let Some(output) = json_val.get("hookSpecificOutput") {
        assert!(
            output.get("hookEventName").is_none(),
            "hookSpecificOutput 내부에 hookEventName 이 있으면 안 됨"
        );
    }
}

// ─── test 3: updatedInput 포함 응답 직렬화 ──────────────────────────────────

/// updatedInput 이 있을 때 응답 구조가 올바른지 검증
#[test]
fn test_hook_response_with_updated_input() {
    let updated = serde_json::json!({"command": "echo hello"});
    let response = HookResponse {
        hook_specific_output: Some(HookSpecificOutput {
            permission_decision: "allow".to_string(),
            updated_input: Some(updated.clone()),
        }),
    };

    let json_val: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&response).unwrap()).unwrap();

    let output = json_val.get("hookSpecificOutput").expect("hookSpecificOutput 없음");
    assert_eq!(output["permissionDecision"], "allow");
    assert_eq!(output["updatedInput"], updated);
}

// ─── test 4: 서버가 POST 요청을 수신하고 200 OK 반환 ─────────────────────────

/// HookServer 를 시작하고 /hooks/PreToolUse 에 POST 요청을 보냈을 때 200 OK 를 반환해야 함
#[tokio::test]
async fn test_hook_server_receives_post() {
    let token = "test-secret-token".to_string();
    let server = HookServer::new(token.clone());
    let (port, _handle) = server.start().await.expect("서버 시작 실패");

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "session_id": "sess-001",
        "cwd": "/tmp",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    });

    let response = client
        .post(format!("http://127.0.0.1:{}/hooks/PreToolUse", port))
        .header("X-Auth-Token", &token)
        .json(&body)
        .send()
        .await
        .expect("요청 전송 실패");

    assert_eq!(response.status(), 200);
}

// ─── test 5: 잘못된 토큰 → 401 Unauthorized ──────────────────────────────────

/// 잘못된 X-Auth-Token 헤더로 요청 시 401 Unauthorized 를 반환해야 함
#[tokio::test]
async fn test_hook_server_rejects_bad_token() {
    let server = HookServer::new("correct-token".to_string());
    let (port, _handle) = server.start().await.expect("서버 시작 실패");

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "session_id": "sess-002",
        "cwd": "/tmp",
        "hook_event_name": "PreToolUse"
    });

    let response = client
        .post(format!("http://127.0.0.1:{}/hooks/PreToolUse", port))
        .header("X-Auth-Token", "wrong-token")
        .json(&body)
        .send()
        .await
        .expect("요청 전송 실패");

    assert_eq!(response.status(), 401);
}

// ─── test 6: PreToolUse → permissionDecision: "allow" ────────────────────────

/// PreToolUse 이벤트에 대해 기본 응답은 permissionDecision: "allow" 여야 함
#[tokio::test]
async fn test_hook_server_pre_tool_use_allow() {
    let token = "allow-test-token".to_string();
    let server = HookServer::new(token.clone());
    let (port, _handle) = server.start().await.expect("서버 시작 실패");

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "session_id": "sess-003",
        "cwd": "/tmp",
        "hook_event_name": "PreToolUse",
        "tool_name": "Read",
        "tool_input": {"file_path": "/tmp/test.txt"}
    });

    let response = client
        .post(format!("http://127.0.0.1:{}/hooks/PreToolUse", port))
        .header("X-Auth-Token", &token)
        .json(&body)
        .send()
        .await
        .expect("요청 전송 실패");

    assert_eq!(response.status(), 200);

    let json_val: serde_json::Value = response.json().await.expect("응답 JSON 파싱 실패");

    // hookEventName 필드가 없어야 함 (Errata E6)
    assert!(json_val.get("hookEventName").is_none(), "Errata E6 위반: hookEventName 존재");

    let output = json_val.get("hookSpecificOutput").expect("hookSpecificOutput 없음");
    assert_eq!(
        output["permissionDecision"], "allow",
        "PreToolUse 기본 응답은 allow 여야 함"
    );
}
