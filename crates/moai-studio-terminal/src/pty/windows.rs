//! Windows PTY stub — Phase 7 (GPUI Windows GA) 으로 이관.
//!
//! Pty trait 의 spawn() / read_available() 호출 사이트를
//! #[cfg(windows)] compile_error! 로 차단 (AC-T-10 iter 2 D4).
//! todo!() 는 사용하지 않는다 — 컴파일 타임 강제.
//!
//! @MX:TODO(conpty-phase-7)
//! @MX:REASON(compile-gate-deferred): Windows ConPTY 실 구현은 Phase 7 (GPUI Windows GA) 대기.
//!   현재는 compile_error! 단일 enforcement 로 차단하여 런타임 panic 을 방지한다.

use super::Pty;
use std::io;

/// Windows ConPTY stub — Phase 7 구현 전까지 compile gate.
///
/// Windows target 에서 spawn() / read_available() 호출 시 compile_error! 가 발생한다.
/// Unix 에서는 이 타입이 존재하지만 Pty trait 구현만 제공 (미지원 에러 반환).
pub struct ConPtyStub;

impl Pty for ConPtyStub {
    fn feed(&mut self, _buf: &[u8]) -> io::Result<()> {
        #[cfg(windows)]
        compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)");
        #[allow(unreachable_code)]
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "ConPtyStub: Phase 7 미구현 (Windows 전용)",
        ))
    }

    fn read_available(&mut self) -> io::Result<Vec<u8>> {
        #[cfg(windows)]
        compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)");
        #[allow(unreachable_code)]
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "ConPtyStub: Phase 7 미구현 (Windows 전용)",
        ))
    }

    fn set_window_size(&mut self, _rows: u16, _cols: u16) -> io::Result<()> {
        #[cfg(windows)]
        compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)");
        #[allow(unreachable_code)]
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "ConPtyStub: Phase 7 미구현 (Windows 전용)",
        ))
    }

    fn is_alive(&self) -> bool {
        false
    }
}
