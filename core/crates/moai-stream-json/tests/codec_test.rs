//! NDJSON 코덱 통합 테스트
//! 스트리밍 JSON 라인 처리와 SdkMessageStream 검증

use moai_stream_json::{SDKMessage, SdkMessageCodec, SdkMessageStream};
use tokio_util::codec::FramedRead;
use futures::StreamExt;

// ───────────────────────────────────────────────
// 다중 라인 NDJSON 스트림 → Vec<SDKMessage> 테스트
// ───────────────────────────────────────────────

#[tokio::test]
async fn test_multiline_ndjson_stream() {
    // 여러 JSON 메시지를 개행으로 구분
    let ndjson = concat!(
        r#"{"type":"system","subtype":"init","session_id":"s1","tools":[],"mcp_servers":[]}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"hi"}]}}"#,
        "\n",
        r#"{"type":"result","subtype":"success","result":"done","cost_usd":0.0,"total_cost_usd":0.0,"duration_ms":100,"duration_api_ms":80}"#,
        "\n",
    );

    let cursor = std::io::Cursor::new(ndjson.as_bytes().to_vec());
    let tokio_reader = tokio::io::BufReader::new(cursor);
    let mut stream: SdkMessageStream<_> = FramedRead::new(tokio_reader, SdkMessageCodec::new());

    let mut messages = Vec::new();
    while let Some(result) = stream.next().await {
        messages.push(result.expect("파싱 실패"));
    }

    assert_eq!(messages.len(), 3);

    // 첫 번째: system/init
    assert!(matches!(&messages[0], SDKMessage::System(_)));

    // 두 번째: assistant
    assert!(matches!(&messages[1], SDKMessage::Assistant(_)));

    // 세 번째: result/success
    assert!(matches!(&messages[2], SDKMessage::Result(_)));
}

// ───────────────────────────────────────────────
// Unknown 변형이 스트림에서 에러 없이 통과하는지 테스트
// ───────────────────────────────────────────────

#[tokio::test]
async fn test_stream_unknown_passes_through() {
    let ndjson = concat!(
        r#"{"type":"future_type","data":42}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ok"}]}}"#,
        "\n",
    );

    let cursor = std::io::Cursor::new(ndjson.as_bytes().to_vec());
    let tokio_reader = tokio::io::BufReader::new(cursor);
    let mut stream: SdkMessageStream<_> = FramedRead::new(tokio_reader, SdkMessageCodec::new());

    let mut messages = Vec::new();
    while let Some(result) = stream.next().await {
        messages.push(result.expect("파싱 실패"));
    }

    assert_eq!(messages.len(), 2);
    assert!(matches!(&messages[0], SDKMessage::Unknown(_)));
    assert!(matches!(&messages[1], SDKMessage::Assistant(_)));
}

// ───────────────────────────────────────────────
// 빈 줄은 건너뛰고 처리되는지 테스트
// ───────────────────────────────────────────────

#[tokio::test]
async fn test_stream_skips_empty_lines() {
    let ndjson = concat!(
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"a"}]}}"#,
        "\n",
        "\n", // 빈 줄
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"b"}]}}"#,
        "\n",
    );

    let cursor = std::io::Cursor::new(ndjson.as_bytes().to_vec());
    let tokio_reader = tokio::io::BufReader::new(cursor);
    let mut stream: SdkMessageStream<_> = FramedRead::new(tokio_reader, SdkMessageCodec::new());

    let mut messages = Vec::new();
    while let Some(result) = stream.next().await {
        messages.push(result.expect("파싱 실패"));
    }

    // 빈 줄 제외하고 2개
    assert_eq!(messages.len(), 2);
}
