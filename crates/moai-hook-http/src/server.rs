//! axum 기반 HTTP 훅 수신 서버

use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{
    error::HookServerError,
    types::{HookEventRequest, HookResponse, HookSpecificOutput},
};

/// 서버 공유 상태 (인증 토큰 보관)
#[derive(Clone)]
struct AppState {
    auth_token: Arc<String>,
}

/// HTTP 훅 수신 서버
pub struct HookServer {
    /// 바인딩할 포트 (0 이면 OS 가 자동 할당)
    port: u16,
    /// 인증 토큰
    auth_token: String,
}

impl HookServer {
    /// 새 HookServer 를 생성 (포트는 OS 자동 할당)
    pub fn new(auth_token: String) -> Self {
        Self {
            port: 0,
            auth_token,
        }
    }

    /// 지정된 포트로 HookServer 를 생성
    pub fn with_port(auth_token: String, port: u16) -> Self {
        Self { port, auth_token }
    }

    /// 서버를 시작하고 (실제 바인딩 포트, JoinHandle) 을 반환
    pub async fn start(&self) -> Result<(u16, JoinHandle<()>), HookServerError> {
        let state = AppState {
            auth_token: Arc::new(self.auth_token.clone()),
        };

        let app = Router::new()
            .route("/hooks/{event}", post(hook_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        // OS 에 포트 자동 할당 요청 (port = 0)
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;
        let actual_port = listener.local_addr()?.port();

        info!("훅 수신 서버 시작: 127.0.0.1:{}", actual_port);

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        Ok((actual_port, handle))
    }
}

// ─── 인증 미들웨어 ───────────────────────────────────────────────────────────

/// X-Auth-Token 헤더를 검증하는 미들웨어
async fn auth_middleware(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get("x-auth-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if token != state.auth_token.as_str() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    next.run(request).await
}

// ─── 훅 이벤트 핸들러 ───────────────────────────────────────────────────────

/// 훅 이벤트를 수신하고 적절한 응답을 반환
async fn hook_handler(
    Path(event): Path<String>,
    Json(body): Json<HookEventRequest>,
) -> impl IntoResponse {
    info!(
        event = %event,
        session_id = %body.session_id,
        tool_name = ?body.tool_name,
        "훅 이벤트 수신"
    );

    let response = build_response(&event);
    (StatusCode::OK, Json(response)).into_response()
}

/// 이벤트 유형에 따른 응답을 생성
/// PreToolUse: permissionDecision = "allow"
/// 나머지: hookSpecificOutput 없음
fn build_response(event: &str) -> HookResponse {
    match event {
        "PreToolUse" => HookResponse {
            hook_specific_output: Some(HookSpecificOutput {
                permission_decision: "allow".to_string(),
                updated_input: None,
            }),
        },
        _ => HookResponse {
            hook_specific_output: None,
        },
    }
}
