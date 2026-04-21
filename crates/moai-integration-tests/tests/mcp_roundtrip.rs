//! MCP 서버 라운드트립 통합 테스트
//!
//! moai-ide-server가 실제 HTTP 요청에 올바르게 응답하는지 검증한다.
//! initialize → tools/list → tools/call 전체 흐름을 단계별로 테스트한다.

use moai_ide_server::server::start_server;

/// MCP 서버가 임의 포트에서 시작되고 포트를 반환하는지 확인
#[tokio::test]
async fn test_mcp_server_binds_random_port() {
    let handle = start_server().await.expect("서버 시작 실패");

    // 유효한 포트 번호인지 확인 (1024 이상)
    assert!(handle.port > 0, "포트는 0보다 커야 한다");
    assert!(handle.port >= 1024, "포트는 1024 이상이어야 한다");

    handle.cancellation_token.cancel();
}

/// MCP initialize 요청 후 200 응답을 받는지 검증
#[tokio::test]
async fn test_mcp_initialize_returns_200() {
    let handle = start_server().await.expect("서버 시작 실패");
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/mcp", handle.port);

    // MCP 프로토콜 initialize 요청
    let init_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "integration-test",
                "version": "1.0"
            }
        }
    });

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body)
        .send()
        .await
        .expect("initialize HTTP 요청 실패");

    assert_eq!(
        resp.status(),
        200,
        "initialize 응답 상태 코드는 200이어야 한다"
    );

    handle.cancellation_token.cancel();
}

/// tools/list 요청 시 echo 도구가 목록에 포함되는지 검증
#[tokio::test]
async fn test_mcp_tools_list_contains_echo() {
    let handle = start_server().await.expect("서버 시작 실패");
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/mcp", handle.port);

    // 1단계: initialize
    let init_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "integration-test", "version": "1.0"}
        }
    });

    client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body)
        .send()
        .await
        .expect("initialize 요청 실패");

    // 2단계: tools/list
    let list_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&list_body)
        .send()
        .await
        .expect("tools/list 요청 실패");

    assert_eq!(resp.status(), 200, "tools/list 응답은 200이어야 한다");

    let body: serde_json::Value = resp.json().await.expect("응답 JSON 파싱 실패");

    // echo 도구 존재 여부 확인
    let tools = body["result"]["tools"]
        .as_array()
        .expect("tools 배열이 응답에 있어야 한다");

    let echo_tool = tools.iter().find(|t| t["name"] == "echo");
    assert!(
        echo_tool.is_some(),
        "echo 도구가 tools/list 목록에 있어야 한다. 실제 도구 목록: {:?}",
        tools
    );

    handle.cancellation_token.cancel();
}

/// tools/call로 echo 도구를 호출하고 메시지가 그대로 반환되는지 검증
#[tokio::test]
async fn test_mcp_echo_tool_call_roundtrip() {
    let handle = start_server().await.expect("서버 시작 실패");
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/mcp", handle.port);

    // 1단계: initialize
    let init_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "integration-test", "version": "1.0"}
        }
    });

    client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body)
        .send()
        .await
        .expect("initialize 요청 실패");

    // 2단계: tools/call echo
    let test_message = "안녕하세요, 통합 테스트입니다";
    let call_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "echo",
            "arguments": {
                "msg": test_message
            }
        }
    });

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&call_body)
        .send()
        .await
        .expect("tools/call 요청 실패");

    assert_eq!(resp.status(), 200, "tools/call 응답은 200이어야 한다");

    let body: serde_json::Value = resp.json().await.expect("응답 JSON 파싱 실패");

    // 결과에 에코된 메시지가 포함되어 있는지 확인
    let result_str = body["result"].to_string();
    assert!(
        result_str.contains(test_message),
        "echo 도구 응답에 입력 메시지가 포함되어야 한다. 실제 응답: {:?}",
        body
    );

    handle.cancellation_token.cancel();
}

/// 두 서버 인스턴스가 서로 다른 포트를 사용하는지 검증 (포트 격리)
#[tokio::test]
async fn test_mcp_server_port_isolation() {
    let handle1 = start_server().await.expect("첫 번째 서버 시작 실패");
    let handle2 = start_server().await.expect("두 번째 서버 시작 실패");

    assert_ne!(
        handle1.port, handle2.port,
        "두 서버 인스턴스는 서로 다른 포트를 사용해야 한다"
    );

    handle1.cancellation_token.cancel();
    handle2.cancellation_token.cancel();
}
