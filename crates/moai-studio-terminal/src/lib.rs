//! MoAI Studio Terminal 레이어 — SPEC-V3-002 Terminal Core.
//!
//! libghostty-vt 기반 VT 파싱 + portable-pty PTY spawn + adaptive buffer worker.
//! Zig 0.15.x 필수 (libghostty-vt 빌드체인, AC-T-1).

pub mod events;
pub mod libghostty_ffi;
pub mod link;
pub mod pty;
pub mod shell;
pub mod vt;
pub mod worker;

/// 스캐폴드 진입점 (moai-studio-app 의 `--scaffold` 로그에서 crate 존재 확인용).
///
/// SPEC-V3-001 부터 유지된 crate presence marker 패턴. 실제 터미널 초기화는
/// `libghostty_ffi::new_terminal` / `worker::PtyWorker` 에서 수행한다.
pub fn hello() {
    tracing::info!("moai-studio-terminal: SPEC-V3-002 Terminal Core (libghostty-vt + PTY)");
}
