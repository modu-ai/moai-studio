//! stream-json 수신 경로 (REQ-AD-002, AC-AD-1)
//!
//! Claude Code subprocess stdout 의 NDJSON 라인을 AgentEvent 로 변환하여 mpsc channel 로 발행.
//! 파싱 실패 시 Unknown fallback (REQ-AD-005).

use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;

use crate::events::{AgentEvent, StreamJsonEvent, TokenUsage};

/// 전역 이벤트 ID 시퀀스 카운터.
static EVENT_ID: AtomicU64 = AtomicU64::new(0);

fn next_event_id() -> u64 {
    EVENT_ID.fetch_add(1, Ordering::Relaxed)
}

/// stream-json 수신 에러.
#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    /// mpsc channel 이 닫혔다.
    #[error("event channel 이 닫혔다")]
    Send,
}

/// stream-json 라인 단위 수신기.
pub struct StreamIngestor {
    tx: mpsc::Sender<AgentEvent>,
}

impl StreamIngestor {
    /// 새 StreamIngestor 를 생성한다.
    pub fn new(tx: mpsc::Sender<AgentEvent>) -> Self {
        Self { tx }
    }

    /// NDJSON 한 줄을 수신하여 AgentEvent 로 변환한 뒤 channel 로 전송한다 (REQ-AD-002).
    ///
    /// 파싱 실패 시 `EventKind::Unknown` 으로 fallback (REQ-AD-005).
    pub async fn ingest_line(&self, line: &str) -> Result<(), IngestError> {
        let id = next_event_id();
        let raw = line.to_string();

        let ev = parse_stream_json_line(id, raw);

        self.tx.send(ev).await.map_err(|_| IngestError::Send)
    }
}

/// NDJSON 한 줄을 AgentEvent 로 파싱한다 (내부 공용 함수).
///
/// 파싱 실패 시 Unknown fallback (REQ-AD-005).
pub(crate) fn parse_stream_json_line(id: u64, raw: String) -> AgentEvent {
    // moai-stream-json decode_line 로 유효성 검사, raw JSON 은 별도 파싱
    match moai_stream_json::decode_line(&raw) {
        Ok(_msg) => {
            // decode_line 성공 → raw JSON 을 Value 로 재파싱하여 payload/type 추출
            let payload: serde_json::Value =
                serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);

            // usage 추출 시도 — rate_limit_event 의 usage 필드
            let usage = extract_usage_from_payload(&payload);

            let type_ = payload
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            AgentEvent::from_stream_json(
                id,
                raw,
                StreamJsonEvent {
                    type_,
                    payload,
                    usage,
                },
            )
        }
        Err(_) => {
            // 파싱 실패 — Unknown fallback (REQ-AD-005)
            tracing::warn!(
                "stream-json 파싱 실패, Unknown fallback: {}",
                &raw[..raw.len().min(80)]
            );
            AgentEvent::unknown(id, raw)
        }
    }
}

/// JSON payload 에서 TokenUsage 를 추출한다 (B1 self-report).
fn extract_usage_from_payload(payload: &serde_json::Value) -> Option<TokenUsage> {
    let usage = payload.get("usage")?;
    let input_tokens = usage.get("input_tokens")?.as_u64()?;
    let output_tokens = usage.get("output_tokens")?.as_u64()?;
    Some(TokenUsage {
        input_tokens,
        output_tokens,
        cost_usd: None, // B1: cost_usd 는 API self-report 없으면 None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventKind;

    /// AC-AD-1: 유효한 stream-json 라인이 StreamJson kind 로 파싱되어야 한다.
    #[test]
    fn ingest_valid_stream_json_line() {
        // system/init 메시지 (moai-stream-json 이 인식하는 형식)
        let line = r#"{"type":"system","subtype":"init","session_id":"test-123","tools":[],"mcp_servers":[]}"#;
        let ev = parse_stream_json_line(0, line.to_string());

        match &ev.kind {
            EventKind::StreamJson(s) => {
                assert_eq!(s.type_, "system");
            }
            other => panic!("StreamJson kind 예상, 실제: {:?}", other),
        }
        assert_eq!(ev.raw, line);
    }

    /// AC-AD-1: 파싱 불가 라인은 Unknown 으로 fallback 되어야 한다 (REQ-AD-005).
    #[test]
    fn ingest_unknown_falls_back_to_unknown_kind() {
        let line = "this is not json at all !!!";
        let ev = parse_stream_json_line(1, line.to_string());

        match ev.kind {
            EventKind::Unknown(_) => {} // 정상 fallback
            other => panic!("Unknown kind 예상, 실제: {:?}", other),
        }
        assert_eq!(ev.raw, line);
    }
}
