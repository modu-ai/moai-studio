//! SDKMessage 파싱 통합 테스트
//! Claude CLI stream-json 프로토콜의 모든 메시지 타입을 검증

use moai_stream_json::{SDKMessage, ContentBlock, UserContentBlock, StreamEventData};

// ───────────────────────────────────────────────
// system/init 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_system_init() {
    let json = r#"{"type":"system","subtype":"init","session_id":"abc123","tools":[{"name":"Bash"},{"name":"Read"}],"mcp_servers":[{"name":"moai","status":"connected"}]}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::System(sys) => match sys {
            moai_stream_json::SystemMessage::Init(init) => {
                assert_eq!(init.session_id, "abc123");
                assert_eq!(init.tools.len(), 2);
                assert_eq!(init.tools[0].name, "Bash");
                assert_eq!(init.mcp_servers.len(), 1);
                assert_eq!(init.mcp_servers[0].name, "moai");
            }
            _ => panic!("예상: Init 서브타입"),
        },
        _ => panic!("예상: System 메시지"),
    }
}

// ───────────────────────────────────────────────
// assistant/text 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_assistant_text() {
    let json = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hello, world!"}]}}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::Assistant(a) => {
            assert_eq!(a.message.role, "assistant");
            assert_eq!(a.message.content.len(), 1);
            match &a.message.content[0] {
                ContentBlock::Text(t) => assert_eq!(t.text, "Hello, world!"),
                _ => panic!("예상: Text 블록"),
            }
        }
        _ => panic!("예상: Assistant 메시지"),
    }
}

// ───────────────────────────────────────────────
// assistant/tool_use 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_assistant_tool_use() {
    let json = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","id":"tu_1","name":"Bash","input":{"command":"ls"}}]}}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::Assistant(a) => {
            match &a.message.content[0] {
                ContentBlock::ToolUse(tu) => {
                    assert_eq!(tu.id, "tu_1");
                    assert_eq!(tu.name, "Bash");
                }
                _ => panic!("예상: ToolUse 블록"),
            }
        }
        _ => panic!("예상: Assistant 메시지"),
    }
}

// ───────────────────────────────────────────────
// assistant/thinking 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_assistant_thinking() {
    let json = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"Let me think...","signature":"sig_xyz"}]}}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::Assistant(a) => {
            match &a.message.content[0] {
                ContentBlock::Thinking(th) => {
                    assert_eq!(th.thinking, "Let me think...");
                    assert_eq!(th.signature, "sig_xyz");
                }
                _ => panic!("예상: Thinking 블록"),
            }
        }
        _ => panic!("예상: Assistant 메시지"),
    }
}

// ───────────────────────────────────────────────
// user/tool_result 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_user_tool_result() {
    let json = r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"tu_1","content":"file1.txt\nfile2.txt"}]}}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::User(u) => {
            assert_eq!(u.message.role, "user");
            match &u.message.content[0] {
                UserContentBlock::ToolResult(tr) => {
                    assert_eq!(tr.tool_use_id, "tu_1");
                    assert_eq!(tr.content, "file1.txt\nfile2.txt");
                }
            }
        }
        _ => panic!("예상: User 메시지"),
    }
}

// ───────────────────────────────────────────────
// rate_limit_event 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_rate_limit_event() {
    let json = r#"{"type":"rate_limit_event","usage":{"input_tokens":100,"output_tokens":50}}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::RateLimitEvent(r) => {
            assert_eq!(r.usage.input_tokens, 100);
            assert_eq!(r.usage.output_tokens, 50);
        }
        _ => panic!("예상: RateLimitEvent 메시지"),
    }
}

// ───────────────────────────────────────────────
// result/success 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_result_success() {
    let json = r#"{"type":"result","subtype":"success","result":"Hello, world!","cost_usd":0.001,"total_cost_usd":0.001,"duration_ms":1234,"duration_api_ms":1000}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::Result(r) => match r {
            moai_stream_json::ResultMessage::Success(s) => {
                assert_eq!(s.result, "Hello, world!");
                assert!((s.cost_usd - 0.001).abs() < 1e-9);
                assert_eq!(s.duration_ms, 1234);
            }
        },
        _ => panic!("예상: Result 메시지"),
    }
}

// ───────────────────────────────────────────────
// stream_event/content_block_delta 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_stream_event_content_block_delta() {
    let json = r#"{"type":"stream_event","event":"content_block_delta","data":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::StreamEvent(se) => match se {
            StreamEventData::ContentBlockDelta(cbd) => {
                assert_eq!(cbd.index, 0);
            }
            _ => panic!("예상: ContentBlockDelta"),
        },
        _ => panic!("예상: StreamEvent 메시지"),
    }
}

// ───────────────────────────────────────────────
// Unknown 변형 테스트: 알 수 없는 type은 Unknown으로 보존
// ───────────────────────────────────────────────

#[test]
fn test_parse_unknown_preserves_raw_value() {
    let json = r#"{"type":"future_type","data":123}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::Unknown(v) => {
            // 원본 JSON 값이 보존되어야 함
            assert_eq!(v["type"], "future_type");
            assert_eq!(v["data"], 123);
        }
        _ => panic!("예상: Unknown 변형"),
    }
}

// ───────────────────────────────────────────────
// system/hook_started 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_system_hook_started() {
    let json = r#"{"type":"system","subtype":"hook_started","hook_type":"PreToolUse","tool_name":"Bash"}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::System(sys) => match sys {
            moai_stream_json::SystemMessage::HookStarted(h) => {
                assert_eq!(h.hook_type, "PreToolUse");
            }
            _ => panic!("예상: HookStarted 서브타입"),
        },
        _ => panic!("예상: System 메시지"),
    }
}

// ───────────────────────────────────────────────
// system/hook_response 파싱 테스트
// ───────────────────────────────────────────────

#[test]
fn test_parse_system_hook_response() {
    let json = r#"{"type":"system","subtype":"hook_response","hook_type":"PreToolUse","decision":"allow"}"#;
    let msg: SDKMessage = serde_json::from_str(json).unwrap();
    match msg {
        SDKMessage::System(sys) => match sys {
            moai_stream_json::SystemMessage::HookResponse(h) => {
                assert_eq!(h.hook_type, "PreToolUse");
                assert_eq!(h.decision, "allow");
            }
            _ => panic!("예상: HookResponse 서브타입"),
        },
        _ => panic!("예상: System 메시지"),
    }
}
