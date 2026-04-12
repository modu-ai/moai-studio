//! NDJSON 코덱: 각 라인을 SDKMessage로 파싱하는 tokio-util 코덱
//!
//! LinesCodec을 래핑하여 최대 1MB 라인 길이를 지원하고,
//! 각 라인을 SDKMessage로 역직렬화합니다.

use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LinesCodec, LinesCodecError};

use crate::message::SDKMessage;

/// 최대 라인 길이: 1MB
const MAX_LINE_LENGTH: usize = 1024 * 1024;

/// NDJSON 스트림을 SDKMessage로 디코딩하는 코덱
pub struct SdkMessageCodec {
    inner: LinesCodec,
}

impl SdkMessageCodec {
    /// 기본 설정으로 코덱을 생성 (최대 1MB 라인 길이)
    pub fn new() -> Self {
        Self {
            inner: LinesCodec::new_with_max_length(MAX_LINE_LENGTH),
        }
    }
}

impl Default for SdkMessageCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// 코덱 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// JSON 파싱 에러
    #[error("JSON 파싱 실패: {0}")]
    Json(#[from] serde_json::Error),
    /// 라인 길이 초과 에러
    #[error("라인 길이 초과: {0}")]
    Lines(#[from] LinesCodecError),
    /// IO 에러
    #[error("IO 에러: {0}")]
    Io(#[from] std::io::Error),
}

impl Decoder for SdkMessageCodec {
    type Item = SDKMessage;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // LinesCodec으로 한 줄씩 읽기
        loop {
            match self.inner.decode(src)? {
                None => return Ok(None),
                Some(line) => {
                    // 빈 줄은 건너뜀
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    // 각 라인을 SDKMessage로 파싱
                    let msg: SDKMessage = serde_json::from_str(trimmed)?;
                    return Ok(Some(msg));
                }
            }
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // EOF 시 남은 데이터 처리
        loop {
            match self.inner.decode_eof(buf)? {
                None => return Ok(None),
                Some(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let msg: SDKMessage = serde_json::from_str(trimmed)?;
                    return Ok(Some(msg));
                }
            }
        }
    }
}

impl<T> Encoder<T> for SdkMessageCodec {
    type Error = CodecError;

    fn encode(&mut self, _item: T, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        // 인코딩은 현재 지원하지 않음 (읽기 전용 코덱)
        unimplemented!("SdkMessageCodec은 읽기 전용입니다")
    }
}

/// SdkMessageStream 타입 별칭: FramedRead와 SdkMessageCodec의 조합
pub type SdkMessageStream<R> = tokio_util::codec::FramedRead<R, SdkMessageCodec>;
