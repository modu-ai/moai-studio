//! MoAI Studio Terminal 레이어 — SPEC-V3-002 Terminal Core.
//!
//! libghostty-vt 기반 VT 파싱 + portable-pty PTY spawn + adaptive buffer worker.
//! Zig 0.15.x 필수 (libghostty-vt 빌드체인, AC-T-1).

pub mod events;
pub mod libghostty_ffi;
pub mod pty;
pub mod vt;
pub mod worker;
