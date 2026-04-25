//! moai-studio-agent — Agent Progress Dashboard 도메인 crate (SPEC-V3-010 MS-1)
//!
//! ## 모듈 구조
//! - `events`: 도메인 타입 (AgentEvent, EventKind, AgentRunId 등)
//! - `ring_buffer`: 고정 용량 원형 버퍼 (capacity 1000 default)
//! - `stream_ingest`: stream-json NDJSON 라인 수신 경로
//! - `sse_ingest`: SSE hook 이벤트 수신 경로 (MS-1 scaffold, MS-2 에서 HTTP 연결)
//! - `view`: EventTimelineView 골격 (MS-1 minimal render)

pub mod events;
pub mod ring_buffer;
pub mod sse_ingest;
pub mod stream_ingest;
pub mod view;

// 자주 쓰는 타입 re-export
pub use events::{AgentEvent, AgentRunId, EventKind, HookEvent, StreamJsonEvent, TokenUsage};
pub use ring_buffer::{RingBuffer, RingBufferError};
