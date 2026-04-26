//! moai-studio-spec — SPEC 문서 파싱 + AC 상태 추적 + 파일 변경 감시.
//!
//! SPEC-V3-009 MS-1 (RG-SU-1 ~ RG-SU-2) 산출물.
//!
//! ## 모듈 구조
//! - [`parser`] — pulldown-cmark 기반 spec.md EARS/AC 표 파싱
//! - [`state`] — AcState enum + AcStateTracker + SpecIndex
//! - [`watch`] — polling 기반 SPEC 디렉터리 변경 감시 + debounce

pub mod parser;
pub mod state;
pub mod watch;

// ── 공개 API 재export (SPEC-V3-009 §12 외부 인터페이스) ──
pub use parser::{ParsedSpec, parse_spec_md};
pub use state::{
    AcRecord, AcState, AcSummary, KanbanStage, SpecFileKind, SpecId, SpecIndex, SpecRecord,
    parse_ac_states_from_progress,
};
pub use watch::{SpecChangeEvent, SpecWatcher};
