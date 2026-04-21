//! T-013: stdin 사용자 메시지 전송 API.
//!
//! SDKUserMessage JSON을 `{"type":"user",...}` 형태로 직렬화하여
//! claude --bare stdin에 줄 단위로 전달한다.
//!
//! @MX:ANCHOR [AUTO] stdin writer 단일 진입점 (fan_in>=2: ffi + supervisor)
//! @MX:WARN [AUTO] stdin writer는 단일 태스크 내에서만 호출해야 한다.
//! @MX:REASON [AUTO] ChildStdin을 BufWriter로 감싸므로 병렬 write는 메시지 섞임을 유발한다.

pub use crate::process::SDKUserMessage;
