//! moai-adk 번들 플러그인 (스캐폴드).
//!
//! Phase 6 (SPEC-V3-013) 에서 구현 예정:
//! - Link parsers: SPEC-ID (`SPEC-[A-Z0-9]+-\d+`), @MX 태그
//! - Surfaces: SPEC card, TRUST 5 radar, Agent Run Viewer, Kanban, Memory Viewer
//! - Sidebar: SPECs, Worktrees
//! - StatusBar widgets: agent-pill, trust5-mini, model-pill
//! - Commands: /moai plan/run/sync/review/coverage/e2e/fix/loop/mx
//! - Hook event listener (27 이벤트 via axum WebSocket)
//!
//! 활성화: `cargo feature "moai-adk"` (moai-studio-app 의 default feature)
//! 비활성화: `cargo build --no-default-features` → 순수 터미널 모드

use tracing::info;

pub fn hello() {
    info!("moai-studio-plugin-moai-adk: scaffold (Phase 0.4). Bundle plugin → Phase 6 (SPEC-V3-013).");
}
