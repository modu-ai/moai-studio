//! moai-studio-agent — Agent Progress Dashboard 도메인 crate (SPEC-V3-010)
//!
//! ## 모듈 구조
//! - `events`: 도메인 타입 (AgentEvent, EventKind, AgentRunId 등)
//! - `ring_buffer`: 고정 용량 원형 버퍼 (capacity 1000 default)
//! - `stream_ingest`: stream-json NDJSON 라인 수신 경로
//! - `sse_ingest`: SSE hook 이벤트 수신 경로 (MS-1 scaffold, MS-2 에서 HTTP 연결)
//! - `view`: EventTimelineView 골격 (MS-1 minimal render, deprecated — UI 측 view 사용)
//! - `cost`: CostTracker — API self-report 비용 집계 (MS-2)
//! - `filter`: EventFilter — 이벤트 종류/run 필터 (MS-2)
//! - `instructions`: InstructionScanner — 6-layer instruction tree 빌드 (MS-3, RG-AD-4)
//! - `control`: ControlEnvelope — pause/resume/kill stdin envelope (MS-3, RG-AD-5)

pub mod control;
pub mod cost;
pub mod events;
pub mod filter;
pub mod instructions;
pub mod ring_buffer;
pub mod sse_ingest;
pub mod stream_ingest;
pub mod view;

// 자주 쓰는 타입 re-export
pub use control::{ControlAction, ControlEnvelope, write_envelope};
pub use cost::{CostSnapshot, CostTracker, extract_from_stream_json, unix_secs_to_system_time};
pub use events::{
    AgentEvent, AgentRunId, AgentRunStatus, EventKind, HookEvent, StreamJsonEvent, TokenUsage,
};
pub use filter::{EventFilter, EventKindDiscriminant, apply_filter};
pub use instructions::{
    InstructionKind, InstructionNode, InstructionRebuildTrigger, InstructionScanner, ScanPaths,
};
pub use ring_buffer::{RingBuffer, RingBufferError};
