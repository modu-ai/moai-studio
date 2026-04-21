//! T-016: workspace별 HTTP hook endpoint + EventBus 발행.
//!
//! 기존 `HookServer`에 workspace-scoped 인증 토큰과 EventBus broadcaster를
//! 연결하여, Claude가 hook POST를 보낼 때마다 이벤트가 UI 레이어로 즉시
//! 전파되도록 한다.
//!
//! @MX:ANCHOR [AUTO] workspace-scoped hook endpoint 단일 진입점
//!   fan_in>=2 (supervisor/lifecycle + integration-tests/hook_roundtrip)
//! @MX:WARN [AUTO] 인증 토큰은 워크스페이스 생명주기 동안 유효 — 노출 시 즉시 재생성 필요.
//! @MX:REASON [AUTO] Claude Code가 settings.json에 토큰을 기록하므로 부주의한 공유는 RCE 벡터.

use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
};
use ring::rand::SecureRandom;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::error::HookServerError;
use crate::types::{HookEventRequest, HookResponse, HookSpecificOutput};

/// workspace-scoped hook endpoint 실행 핸들.
pub struct HookEndpoint {
    /// 바인딩된 실제 포트
    pub port: u16,
    /// Claude 요청 시 사용할 인증 토큰 (hex)
    pub auth_token: String,
    /// 백그라운드 axum 태스크
    _task: JoinHandle<()>,
}

#[derive(Clone)]
struct EndpointState {
    token: Arc<String>,
    bus: broadcast::Sender<String>,
    workspace_id: u64,
}

impl HookEndpoint {
    /// 워크스페이스별 endpoint를 바인딩한다 (port=0 → OS assigned).
    pub async fn spawn(
        workspace_id: u64,
        bus: broadcast::Sender<String>,
    ) -> Result<Self, HookServerError> {
        let token = generate_token();
        let state = EndpointState {
            token: Arc::new(token.clone()),
            bus,
            workspace_id,
        };

        let app = Router::new()
            .route("/hooks/{event}", post(hook_handler))
            .layer(middleware::from_fn_with_state(state.clone(), auth_mw))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        Ok(Self {
            port,
            auth_token: token,
            _task: task,
        })
    }

    /// endpoint base URL.
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

/// 32-byte ring::SystemRandom 토큰 (hex 64자).
fn generate_token() -> String {
    let mut buf = [0u8; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut buf)
        .expect("ring SystemRandom::fill 실패");
    hex::encode(buf)
}

async fn auth_mw(State(s): State<EndpointState>, req: Request, next: Next) -> Response {
    let tok = req
        .headers()
        .get("x-auth-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if tok != s.token.as_str() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(req).await
}

async fn hook_handler(
    State(s): State<EndpointState>,
    Path(event): Path<String>,
    Json(body): Json<HookEventRequest>,
) -> impl IntoResponse {
    // 1) EventBus 발행 (<10ms 목표: serialize + send 만 수행)
    let payload = serde_json::json!({
        "kind": "hook",
        "workspace_id": s.workspace_id,
        "event": &event,
        "session_id": body.session_id,
        "tool_name": body.tool_name,
    });
    let _ = s.bus.send(payload.to_string());

    // 2) Claude 에 적절한 응답 반환
    let resp = match event.as_str() {
        "PreToolUse" => HookResponse {
            hook_specific_output: Some(HookSpecificOutput {
                permission_decision: "allow".to_string(),
                updated_input: None,
            }),
        },
        _ => HookResponse {
            hook_specific_output: None,
        },
    };
    (StatusCode::OK, Json(resp))
}
