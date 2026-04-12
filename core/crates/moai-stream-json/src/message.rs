//! SDKMessage: Claude CLI stream-json 프로토콜의 모든 메시지 타입 정의
//!
//! 2단계 역직렬화 전략을 사용:
//! 1. RawMessage로 type/subtype 필드와 원본 JSON 값을 캡처
//! 2. type+subtype 조합으로 올바른 SDKMessage 변형을 생성

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ───────────────────────────────────────────────
// 보조 타입들
// ───────────────────────────────────────────────

/// 도구 정보 (system/init에서 사용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
}

/// MCP 서버 정보 (system/init에서 사용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    #[serde(default)]
    pub status: String,
}

/// 토큰 사용량 정보 (rate_limit_event에서 사용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

// ───────────────────────────────────────────────
// system 메시지 서브타입
// ───────────────────────────────────────────────

/// system/init 페이로드
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInit {
    pub session_id: String,
    #[serde(default)]
    pub tools: Vec<ToolInfo>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerInfo>,
}

/// system/hook_started 페이로드
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHookStarted {
    pub hook_type: String,
    #[serde(default)]
    pub tool_name: Option<String>,
}

/// system/hook_response 페이로드
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHookResponse {
    pub hook_type: String,
    pub decision: String,
}

/// system 메시지의 서브타입 열거형
#[derive(Debug, Clone)]
pub enum SystemMessage {
    /// 세션 초기화 메시지
    Init(SystemInit),
    /// 훅 시작 알림
    HookStarted(SystemHookStarted),
    /// 훅 응답 메시지
    HookResponse(SystemHookResponse),
}

// ───────────────────────────────────────────────
// assistant 메시지 콘텐츠 블록
// ───────────────────────────────────────────────

/// 텍스트 블록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
}

/// 도구 사용 블록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: Value,
}

/// 사고 과정 블록 (extended thinking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    pub thinking: String,
    pub signature: String,
}

/// assistant 메시지의 콘텐츠 블록 타입
#[derive(Debug, Clone)]
pub enum ContentBlock {
    /// 텍스트 응답
    Text(TextBlock),
    /// 도구 호출
    ToolUse(ToolUseBlock),
    /// 확장 사고 과정
    Thinking(ThinkingBlock),
}

impl<'de> Deserialize<'de> for ContentBlock {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // type 필드 기반으로 구분
        let v = Value::deserialize(deserializer)?;
        let block_type = v.get("type").and_then(Value::as_str).unwrap_or("");
        match block_type {
            "text" => {
                let b: TextBlock = serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                Ok(ContentBlock::Text(b))
            }
            "tool_use" => {
                let b: ToolUseBlock =
                    serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                Ok(ContentBlock::ToolUse(b))
            }
            "thinking" => {
                let b: ThinkingBlock =
                    serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                Ok(ContentBlock::Thinking(b))
            }
            other => Err(serde::de::Error::custom(format!(
                "알 수 없는 콘텐츠 블록 타입: {other}"
            ))),
        }
    }
}

/// assistant 메시지의 message 필드
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessageBody {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

/// assistant 메시지 전체
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessage {
    pub message: AssistantMessageBody,
}

// ───────────────────────────────────────────────
// user 메시지 콘텐츠 블록
// ───────────────────────────────────────────────

/// 도구 실행 결과 블록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    pub content: String,
}

/// user 메시지의 콘텐츠 블록 타입
#[derive(Debug, Clone)]
pub enum UserContentBlock {
    /// 도구 실행 결과
    ToolResult(ToolResultBlock),
}

impl<'de> Deserialize<'de> for UserContentBlock {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        let block_type = v.get("type").and_then(Value::as_str).unwrap_or("");
        match block_type {
            "tool_result" => {
                let b: ToolResultBlock =
                    serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                Ok(UserContentBlock::ToolResult(b))
            }
            other => Err(serde::de::Error::custom(format!(
                "알 수 없는 user 콘텐츠 블록 타입: {other}"
            ))),
        }
    }
}

/// user 메시지의 message 필드
#[derive(Debug, Clone, Deserialize)]
pub struct UserMessageBody {
    pub role: String,
    pub content: Vec<UserContentBlock>,
}

/// user 메시지 전체
#[derive(Debug, Clone, Deserialize)]
pub struct UserMessage {
    pub message: UserMessageBody,
}

// ───────────────────────────────────────────────
// rate_limit_event 메시지
// ───────────────────────────────────────────────

/// 요청 속도 제한 이벤트
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitEvent {
    pub usage: UsageInfo,
}

// ───────────────────────────────────────────────
// result 메시지 서브타입
// ───────────────────────────────────────────────

/// result/success 페이로드
#[derive(Debug, Clone, Deserialize)]
pub struct ResultSuccess {
    pub result: String,
    pub cost_usd: f64,
    pub total_cost_usd: f64,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
}

/// result 메시지 서브타입 열거형
#[derive(Debug, Clone)]
pub enum ResultMessage {
    /// 성공적으로 완료된 결과
    Success(ResultSuccess),
}

// ───────────────────────────────────────────────
// stream_event 메시지 서브타입
// ───────────────────────────────────────────────

/// message_start 이벤트 데이터
#[derive(Debug, Clone, Deserialize)]
pub struct MessageStartData {
    #[serde(flatten)]
    pub raw: Value,
}

/// content_block_start 이벤트 데이터
#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlockStartData {
    pub index: u32,
    #[serde(flatten)]
    pub raw: Value,
}

/// content_block_delta 이벤트 데이터
#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlockDeltaData {
    pub index: u32,
    pub delta: Value,
}

/// content_block_stop 이벤트 데이터
#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlockStopData {
    pub index: u32,
}

/// message_delta 이벤트 데이터
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaData {
    #[serde(flatten)]
    pub raw: Value,
}

/// message_stop 이벤트 데이터
#[derive(Debug, Clone, Deserialize)]
pub struct MessageStopData {
    #[serde(flatten)]
    pub raw: Value,
}

/// stream_event 메시지의 6가지 서브타입
#[derive(Debug, Clone)]
pub enum StreamEventData {
    /// 메시지 시작
    MessageStart(MessageStartData),
    /// 콘텐츠 블록 시작
    ContentBlockStart(ContentBlockStartData),
    /// 콘텐츠 블록 델타 (스트리밍 텍스트)
    ContentBlockDelta(ContentBlockDeltaData),
    /// 콘텐츠 블록 종료
    ContentBlockStop(ContentBlockStopData),
    /// 메시지 델타
    MessageDelta(MessageDeltaData),
    /// 메시지 종료
    MessageStop(MessageStopData),
}

// ───────────────────────────────────────────────
// 2단계 역직렬화를 위한 내부 타입
// ───────────────────────────────────────────────

/// 1단계: type/subtype 필드와 원본 JSON 값을 캡처하는 임시 구조체
#[derive(Deserialize)]
struct RawMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    data: Option<Value>,
    /// 나머지 필드 전체를 보존
    #[serde(flatten)]
    rest: Value,
}

// ───────────────────────────────────────────────
// SDKMessage: 최상위 메시지 열거형
// ───────────────────────────────────────────────

/// Claude CLI stream-json 프로토콜의 모든 메시지 타입
#[derive(Debug, Clone)]
pub enum SDKMessage {
    /// 시스템 메시지 (init, hook_started, hook_response)
    System(SystemMessage),
    /// 어시스턴트 응답 메시지
    Assistant(AssistantMessage),
    /// 사용자 메시지 (tool_result 포함)
    User(UserMessage),
    /// 속도 제한 이벤트
    RateLimitEvent(RateLimitEvent),
    /// 실행 결과 (success 서브타입)
    Result(ResultMessage),
    /// 스트리밍 이벤트 (6가지 서브타입)
    StreamEvent(StreamEventData),
    /// 알 수 없는 메시지 타입 (원본 JSON 값 보존)
    Unknown(Value),
}

impl<'de> Deserialize<'de> for SDKMessage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // 1단계: RawMessage로 type과 subtype을 추출
        let raw = RawMessage::deserialize(deserializer)?;

        // 원본 Value 재조립 (type, subtype 포함)
        let full_value = {
            let mut map = match raw.rest {
                Value::Object(m) => m,
                _ => serde_json::Map::new(),
            };
            map.insert("type".to_string(), Value::String(raw.msg_type.clone()));
            if let Some(ref st) = raw.subtype {
                map.insert("subtype".to_string(), Value::String(st.clone()));
            }
            if let Some(ref ev) = raw.event {
                map.insert("event".to_string(), Value::String(ev.clone()));
            }
            if let Some(ref d) = raw.data {
                map.insert("data".to_string(), d.clone());
            }
            Value::Object(map)
        };

        // 2단계: type+subtype으로 올바른 변형으로 디스패치
        match raw.msg_type.as_str() {
            "system" => {
                let subtype = raw.subtype.as_deref().unwrap_or("");
                match subtype {
                    "init" => {
                        let init: SystemInit = serde_json::from_value(full_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::System(SystemMessage::Init(init)))
                    }
                    "hook_started" => {
                        let h: SystemHookStarted = serde_json::from_value(full_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::System(SystemMessage::HookStarted(h)))
                    }
                    "hook_response" => {
                        let h: SystemHookResponse = serde_json::from_value(full_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::System(SystemMessage::HookResponse(h)))
                    }
                    other => {
                        tracing::warn!("알 수 없는 system 서브타입: {other}");
                        Ok(SDKMessage::Unknown(full_value))
                    }
                }
            }
            "assistant" => {
                let msg: AssistantMessage = serde_json::from_value(full_value)
                    .map_err(serde::de::Error::custom)?;
                Ok(SDKMessage::Assistant(msg))
            }
            "user" => {
                let msg: UserMessage =
                    serde_json::from_value(full_value).map_err(serde::de::Error::custom)?;
                Ok(SDKMessage::User(msg))
            }
            "rate_limit_event" => {
                let msg: RateLimitEvent = serde_json::from_value(full_value)
                    .map_err(serde::de::Error::custom)?;
                Ok(SDKMessage::RateLimitEvent(msg))
            }
            "result" => {
                let subtype = raw.subtype.as_deref().unwrap_or("");
                match subtype {
                    "success" => {
                        let s: ResultSuccess = serde_json::from_value(full_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::Result(ResultMessage::Success(s)))
                    }
                    other => {
                        tracing::warn!("알 수 없는 result 서브타입: {other}");
                        Ok(SDKMessage::Unknown(full_value))
                    }
                }
            }
            "stream_event" => {
                let event_type = raw.event.as_deref().unwrap_or("");
                // data 필드가 있으면 사용, 없으면 전체 값 사용
                let data_value = raw.data.unwrap_or(full_value);
                match event_type {
                    "message_start" => {
                        let d: MessageStartData = serde_json::from_value(data_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::StreamEvent(StreamEventData::MessageStart(d)))
                    }
                    "content_block_start" => {
                        let d: ContentBlockStartData = serde_json::from_value(data_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::StreamEvent(StreamEventData::ContentBlockStart(
                            d,
                        )))
                    }
                    "content_block_delta" => {
                        let d: ContentBlockDeltaData = serde_json::from_value(data_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::StreamEvent(StreamEventData::ContentBlockDelta(
                            d,
                        )))
                    }
                    "content_block_stop" => {
                        let d: ContentBlockStopData = serde_json::from_value(data_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::StreamEvent(StreamEventData::ContentBlockStop(d)))
                    }
                    "message_delta" => {
                        let d: MessageDeltaData = serde_json::from_value(data_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::StreamEvent(StreamEventData::MessageDelta(d)))
                    }
                    "message_stop" => {
                        let d: MessageStopData = serde_json::from_value(data_value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(SDKMessage::StreamEvent(StreamEventData::MessageStop(d)))
                    }
                    other => {
                        tracing::warn!("알 수 없는 stream_event 이벤트 타입: {other}");
                        Ok(SDKMessage::Unknown(data_value))
                    }
                }
            }
            other => {
                // 알 수 없는 메시지 타입은 원본 JSON 값을 보존하고 경고 로그 출력
                tracing::warn!("알 수 없는 SDKMessage 타입: {other}");
                Ok(SDKMessage::Unknown(full_value))
            }
        }
    }
}
