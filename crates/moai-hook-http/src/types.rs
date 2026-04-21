//! Claude 훅 이벤트 타입 정의

use serde::{Deserialize, Serialize};

/// Claude 가 전송하는 훅 이벤트 요청 본문
#[derive(Debug, Deserialize)]
pub struct HookEventRequest {
    pub session_id: String,
    pub cwd: String,
    pub hook_event_name: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_use_id: Option<String>,
    // 추가 필드는 무시 (하위 호환성)
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// 훅 응답 형식
/// [Errata E6: hookEventName 필드를 절대 포함하지 않음]
#[derive(Debug, Serialize)]
pub struct HookResponse {
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

/// hookSpecificOutput 내부 구조
#[derive(Debug, Serialize)]
pub struct HookSpecificOutput {
    /// 권한 결정: "allow", "deny", "ask"
    #[serde(rename = "permissionDecision")]
    pub permission_decision: String,
    /// 수정된 입력값 (선택적)
    #[serde(rename = "updatedInput", skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
}
