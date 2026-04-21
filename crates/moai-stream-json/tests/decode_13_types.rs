//! T-014: SDKMessage 13종 실시간 디코딩 + EventBus 발행 검증.
//!
//! 13종 = System{Init, HookStarted, HookResponse}
//!       + Assistant + User + RateLimitEvent + Result::Success
//!       + StreamEvent{MessageStart, ContentBlockStart, ContentBlockDelta,
//!                     ContentBlockStop, MessageDelta, MessageStop}

use moai_stream_json::{
    ContentBlock, SDKMessage, StreamEventData, SystemMessage, UserContentBlock, decode_and_publish,
    decode_line,
};
use tokio::sync::broadcast;

/// 13종 NDJSON 픽스처 생성 — 순서 고정.
fn thirteen_lines() -> Vec<&'static str> {
    vec![
        // 1. system/init
        r#"{"type":"system","subtype":"init","session_id":"s1","tools":[],"mcp_servers":[]}"#,
        // 2. system/hook_started
        r#"{"type":"system","subtype":"hook_started","hook_type":"PreToolUse","tool_name":"Bash"}"#,
        // 3. system/hook_response
        r#"{"type":"system","subtype":"hook_response","hook_type":"PreToolUse","decision":"allow"}"#,
        // 4. assistant
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"hi"}]}}"#,
        // 5. user (tool_result 포함)
        r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"u1","content":"ok"}]}}"#,
        // 6. rate_limit_event
        r#"{"type":"rate_limit_event","usage":{"input_tokens":10,"output_tokens":5}}"#,
        // 7. result/success
        r#"{"type":"result","subtype":"success","result":"done","cost_usd":0.0,"total_cost_usd":0.0,"duration_ms":1,"duration_api_ms":1}"#,
        // 8. stream_event/message_start
        r#"{"type":"stream_event","event":"message_start","data":{"message":{"id":"m1","role":"assistant"}}}"#,
        // 9. stream_event/content_block_start
        r#"{"type":"stream_event","event":"content_block_start","data":{"index":0,"content_block":{"type":"text","text":""}}}"#,
        // 10. stream_event/content_block_delta
        r#"{"type":"stream_event","event":"content_block_delta","data":{"index":0,"delta":{"type":"text_delta","text":"he"}}}"#,
        // 11. stream_event/content_block_stop
        r#"{"type":"stream_event","event":"content_block_stop","data":{"index":0}}"#,
        // 12. stream_event/message_delta
        r#"{"type":"stream_event","event":"message_delta","data":{"delta":{"stop_reason":"end_turn"}}}"#,
        // 13. stream_event/message_stop
        r#"{"type":"stream_event","event":"message_stop","data":{}}"#,
    ]
}

#[test]
fn decode_line_covers_all_thirteen_variants() {
    let lines = thirteen_lines();
    assert_eq!(lines.len(), 13, "13 종 픽스처 길이 불일치");

    let mut stream_events_seen = std::collections::HashSet::new();
    let mut system_events_seen = std::collections::HashSet::new();

    for (i, line) in lines.iter().enumerate() {
        let msg =
            decode_line(line).unwrap_or_else(|e| panic!("line {i} 디코딩 실패: {e} | {line}"));
        let label = match &msg {
            SDKMessage::System(SystemMessage::Init(_)) => {
                system_events_seen.insert("init");
                "system/init"
            }
            SDKMessage::System(SystemMessage::HookStarted(_)) => {
                system_events_seen.insert("hook_started");
                "system/hook_started"
            }
            SDKMessage::System(SystemMessage::HookResponse(_)) => {
                system_events_seen.insert("hook_response");
                "system/hook_response"
            }
            SDKMessage::Assistant(a) => {
                match &a.message.content[0] {
                    ContentBlock::Text(_) => {}
                    _ => panic!("assistant content[0] != text"),
                }
                "assistant"
            }
            SDKMessage::User(u) => {
                let UserContentBlock::ToolResult(_) = &u.message.content[0];
                "user"
            }
            SDKMessage::RateLimitEvent(_) => "rate_limit_event",
            SDKMessage::Result(_) => "result/success",
            SDKMessage::StreamEvent(d) => {
                let key = match d {
                    StreamEventData::MessageStart(_) => "message_start",
                    StreamEventData::ContentBlockStart(_) => "content_block_start",
                    StreamEventData::ContentBlockDelta(_) => "content_block_delta",
                    StreamEventData::ContentBlockStop(_) => "content_block_stop",
                    StreamEventData::MessageDelta(_) => "message_delta",
                    StreamEventData::MessageStop(_) => "message_stop",
                };
                stream_events_seen.insert(key);
                key
            }
            SDKMessage::Unknown(_) => panic!("line {i}: Unknown variant — 분류 실패"),
        };
        println!("#{i}: {label}");
    }

    // System 3종, StreamEvent 6종 모두 커버됐는지 확인
    assert_eq!(
        system_events_seen.len(),
        3,
        "System 3종 미커버: {system_events_seen:?}"
    );
    assert_eq!(
        stream_events_seen.len(),
        6,
        "StreamEvent 6종 미커버: {stream_events_seen:?}"
    );
}

#[tokio::test]
async fn decode_and_publish_broadcasts_all_messages_to_eventbus() {
    let (tx, mut rx) = broadcast::channel::<String>(64);
    let payload = thirteen_lines().join("\n");
    let reader = std::io::Cursor::new(payload.into_bytes());

    let stats = decode_and_publish(reader, &tx).await;

    assert_eq!(stats.decoded, 13, "디코딩 카운트 불일치: {stats:?}");
    assert_eq!(stats.errors, 0, "파싱 에러 있음: {stats:?}");
    assert_eq!(stats.publish_errors, 0, "발행 에러 있음: {stats:?}");

    // EventBus 수신 검증
    let mut received = 0usize;
    while let Ok(msg) = rx.try_recv() {
        assert!(
            msg.contains(r#""type""#),
            "발행된 페이로드에 type 없음: {msg}"
        );
        received += 1;
    }
    assert_eq!(received, 13, "EventBus 수신 수 불일치: {received}");
}

#[tokio::test]
async fn decoder_skips_malformed_lines_and_continues() {
    let (tx, _rx) = broadcast::channel::<String>(16);
    let payload = concat!(
        r#"{"type":"system","subtype":"init","session_id":"s","tools":[],"mcp_servers":[]}"#,
        "\n",
        r#"{not json"#,
        "\n",
        r#"{"type":"result","subtype":"success","result":"ok","cost_usd":0.0,"total_cost_usd":0.0,"duration_ms":1,"duration_api_ms":1}"#,
        "\n",
    );
    let reader = std::io::Cursor::new(payload.as_bytes().to_vec());
    let stats = decode_and_publish(reader, &tx).await;
    assert_eq!(stats.decoded, 2, "유효한 2건만 디코딩되어야 함: {stats:?}");
    assert_eq!(
        stats.errors, 1,
        "깨진 1건은 errors 로 집계되어야 함: {stats:?}"
    );
}
