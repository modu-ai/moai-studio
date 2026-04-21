//! MoAI Studio Multi-Project Workspace (스캐폴드).
//!
//! Phase 5 (SPEC-V3-004) 에서 구현 예정:
//! - `~/.moai/studio/workspaces.json` persistence (VS Code `.code-workspace` 모델)
//! - 사이드바 workspace 스위처 (드롭다운 + 최근 4)
//! - 프로젝트 전환 시 pane tree + tab state 복원
//! - 글로벌 검색 across workspaces
//! - 드래그앤드롭으로 workspace 추가
//! - Workspace 별 색상 태그
//!
//! Foundation: `moai-store` crate (SQLite) 재사용.

use tracing::info;

pub fn hello() {
    info!("moai-studio-workspace: scaffold (Phase 0.4). Multi-project → Phase 5 (SPEC-V3-004).");
}
