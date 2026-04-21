//! MoAI Studio UI 컴포넌트 라이브러리 (스캐폴드).
//!
//! Phase 1 (SPEC-V3-001 RG-V3-2) 에서 GPUI 기반 컴포넌트 구현 예정:
//! - Sidebar (Workspace 스위처 + SPECs + Worktrees)
//! - Toolbar (7 primary actions)
//! - StatusBar (Agent pill, Git, LSP, ⌘K 힌트)
//! - PaneTree (binary tree split + draggable divider)
//! - TabBar (per-pane tabs)
//! - CommandPalette (⌘K nested + @/# mention)
//! - EmptyState CTA

use tracing::info;

pub fn hello() {
    info!("moai-studio-ui: scaffold (Phase 0.4). GPUI integration → Phase 1.");
}
