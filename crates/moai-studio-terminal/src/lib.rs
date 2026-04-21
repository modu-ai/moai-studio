//! MoAI Studio Terminal 레이어 (스캐폴드).
//!
//! Phase 2 (SPEC-V3-002) 에서 구현 예정:
//! - libghostty-vt 기반 VT state/parser 바인딩
//! - portable-pty 로 cross-platform PTY spawn
//! - Multi-shell 지원 (zsh/bash/fish/nu/pwsh/cmd)
//! - tmux 호환성 (OSC 8, mouse, bracketed paste, 256+24bit color)
//! - GPUI 기반 셀 그리드 렌더
//! - 세션 persistence (재시작 시 pane tree 복원)
//!
//! 전제: Zig 0.15.x 설치 필수 (libghostty-vt 빌드 체인 의존).

use tracing::info;

pub fn hello() {
    info!("moai-studio-terminal: scaffold (Phase 0.4). libghostty-vt → Phase 2 (SPEC-V3-002).");
}
