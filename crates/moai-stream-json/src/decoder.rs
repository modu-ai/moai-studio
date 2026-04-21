//! T-014: stdout → SDKMessage 13종 실시간 디코더 + EventBus 발행 헬퍼.
//!
//! `codec.rs`의 `SdkMessageCodec`을 감싸 AsyncRead 스트림에서 `SDKMessage`를
//! 하나씩 추출하고, 주어진 `tokio::sync::broadcast::Sender`에 JSON 페이로드로
//! 발행한다.
//!
//! @MX:ANCHOR [AUTO] stream-json decoder publish 단일 진입점
//!   fan_in>=2 (claude-host monitor + integration-tests subprocess_stream)
//! @MX:NOTE [AUTO] SDKMessage discriminator는 message.rs Deserialize impl에서 수행.

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::broadcast::Sender;

use crate::message::SDKMessage;

/// 디코더 실행 결과 요약.
#[derive(Debug, Default, Clone)]
pub struct DecodeStats {
    /// 성공적으로 디코딩된 메시지 수
    pub decoded: u64,
    /// 디코딩 실패(JSON 파싱 오류 등) 카운트 — 로깅 후 계속 진행
    pub errors: u64,
    /// broadcast 발행 실패 수 (receiver가 없을 때)
    pub publish_errors: u64,
}

/// `reader`에서 NDJSON을 읽어 `SDKMessage`로 디코딩 후
/// 원본 NDJSON 라인을 `bus`에 발행한다.
///
/// SDKMessage는 Deserialize 전용이므로 EventBus에는 원본 라인을 그대로
/// 전달한다 (Claude CLI가 생성한 JSON을 UI/FFI가 그대로 파싱).
///
/// EOF에 도달하면 `DecodeStats`를 반환한다. 개별 라인의 파싱 실패는
/// `tracing::warn`으로 로깅한 뒤 카운팅만 하고 계속 진행한다.
pub async fn decode_and_publish<R>(reader: R, bus: &Sender<String>) -> DecodeStats
where
    R: AsyncRead + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut stats = DecodeStats::default();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<SDKMessage>(trimmed) {
                    Ok(_msg) => {
                        stats.decoded += 1;
                        if bus.send(trimmed.to_string()).is_err() {
                            stats.publish_errors += 1;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("NDJSON SDKMessage 파싱 실패(skip): {e}");
                        stats.errors += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("decoder IO 오류로 중단: {e}");
                break;
            }
        }
    }

    stats
}

/// 단일 NDJSON 라인을 `SDKMessage`로 디코딩한다 (테스트/배치 용도).
pub fn decode_line(line: &str) -> Result<SDKMessage, serde_json::Error> {
    serde_json::from_str(line)
}
