//! MCP 서버 모듈 — rmcp + axum 기반 Streamable HTTP 서버

use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

/// echo 도구의 입력 파라미터
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EchoParams {
    /// 에코할 메시지
    pub msg: String,
}

/// MoAI IDE MCP 서버
///
/// Claude 가 `--mcp-config` 로 연결하는 MCP 서버.
/// Streamable HTTP 전송을 통해 tool 호출을 처리한다.
///
/// # @MX:ANCHOR: [AUTO] MCP 서버 진입점 — ServerHandler 구현체
/// # @MX:REASON: tool_router(server_handler) 매크로가 list_tools/call_tool 을 자동 생성함
#[derive(Debug, Clone)]
pub struct MoaiIdeServer;

/// tool_router(server_handler) — list_tools/call_tool 구현을 자동으로 ServerHandler 에 위임
#[tool_router(server_handler)]
impl MoaiIdeServer {
    /// echo 도구: 입력 메시지를 그대로 반환한다
    #[tool(description = "입력 메시지를 그대로 에코한다")]
    fn echo(&self, Parameters(EchoParams { msg }): Parameters<EchoParams>) -> String {
        msg
    }
}

/// 서버 시작 결과
pub struct ServerHandle {
    /// 실제로 바인딩된 포트
    pub port: u16,
    /// 서버 종료 토큰
    pub cancellation_token: CancellationToken,
}

impl MoaiIdeServer {
    /// 새 서버 인스턴스 생성
    pub fn new() -> Self {
        Self
    }
}

impl Default for MoaiIdeServer {
    fn default() -> Self {
        Self::new()
    }
}

/// 서버를 임의 포트에 바인딩하고 백그라운드 태스크로 실행한다.
///
/// 반환된 `ServerHandle` 의 `cancellation_token.cancel()` 을 호출하면 서버가 종료된다.
///
/// # @MX:ANCHOR: [AUTO] 서버 시작 진입점 — 외부 호출자(Claude 프로세스)가 참조
/// # @MX:REASON: moai-claude-host 에서 이 함수를 통해 MCP 서버를 생성함
pub async fn start_server() -> anyhow::Result<ServerHandle> {
    let ct = CancellationToken::new();

    // StreamableHTTP 서비스 생성 (stateless + JSON 응답 모드)
    let config = StreamableHttpServerConfig::default()
        .with_stateful_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_cancellation_token(ct.child_token());

    let service: StreamableHttpService<MoaiIdeServer, LocalSessionManager> =
        StreamableHttpService::new(|| Ok(MoaiIdeServer::new()), Default::default(), config);

    // OS가 임의 포트를 할당하도록 :0 으로 바인딩
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    let router = axum::Router::new().nest_service("/mcp", service);

    tokio::spawn({
        let ct = ct.clone();
        async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                .await;
        }
    });

    Ok(ServerHandle {
        port,
        cancellation_token: ct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 서버가 임의 포트에 바인딩되고 포트 번호를 반환하는지 검증한다
    #[tokio::test]
    async fn test_mcp_server_starts() {
        let handle = start_server().await.expect("서버 시작 실패");
        assert!(handle.port > 0, "포트는 0보다 커야 한다");
        handle.cancellation_token.cancel();
    }

    /// 두 서버 인스턴스가 서로 다른 포트를 할당받는지 검증한다
    #[tokio::test]
    async fn test_mcp_server_unique_ports() {
        let handle1 = start_server().await.expect("첫 번째 서버 시작 실패");
        let handle2 = start_server().await.expect("두 번째 서버 시작 실패");
        assert_ne!(
            handle1.port, handle2.port,
            "두 서버는 서로 다른 포트를 사용해야 한다"
        );
        handle1.cancellation_token.cancel();
        handle2.cancellation_token.cancel();
    }

    /// tools/list 요청 시 echo 도구가 목록에 포함되는지 검증한다 (통합 테스트)
    #[tokio::test]
    async fn test_mcp_server_echo_tool() {
        let handle = start_server().await.expect("서버 시작 실패");
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/mcp", handle.port);

        // MCP initialize 요청
        let init_body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(init_body)
            .send()
            .await
            .expect("initialize 요청 실패");
        assert_eq!(resp.status(), 200, "initialize 응답이 200이어야 한다");

        // MCP tools/list 요청
        let list_body = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(list_body)
            .send()
            .await
            .expect("tools/list 요청 실패");

        assert_eq!(resp.status(), 200, "tools/list 응답이 200이어야 한다");

        let body: serde_json::Value = resp.json().await.expect("JSON 파싱 실패");
        let tools = body["result"]["tools"]
            .as_array()
            .expect("tools 배열이 존재해야 한다");

        let has_echo = tools.iter().any(|t| t["name"] == "echo");
        assert!(
            has_echo,
            "echo 도구가 tools/list 에 포함되어야 한다: {:?}",
            tools
        );

        handle.cancellation_token.cancel();
    }
}
