//! @MX:ANCHOR(pty-trait-contract)
//! PTY 추상화 trait — cross-platform 계약.
//!
//! fan_in ≥ 3: UnixPty, ConPtyStub, MockPty(test), worker::PtyWorker.
//! 이 trait 을 구현하는 모든 타입은 Send 이어야 한다 (reader 스레드로 이동 가능).

use std::io;

pub mod unix;
pub mod windows;

/// 테스트 전용 MockPty — integration test 에서도 접근 가능하도록 항상 컴파일.
#[doc(hidden)]
pub mod mock;

#[cfg(unix)]
pub use unix::UnixPty;

pub use windows::ConPtyStub;

#[doc(hidden)]
pub use mock::MockPty;

/// PTY 인터페이스 trait.
///
/// 모든 구현체는 Send 이어야 한다 (reader 를 별도 thread 로 이동).
/// Sync 는 요구하지 않는다 — 단일 스레드 접근이 기본.
pub trait Pty: Send {
    /// PTY stdin 에 바이트를 쓴다 (키 입력 → shell).
    fn feed(&mut self, buf: &[u8]) -> io::Result<()>;

    /// PTY stdout 에서 읽을 수 있는 바이트를 모두 읽는다 (non-blocking).
    fn read_available(&mut self) -> io::Result<Vec<u8>>;

    /// 터미널 윈도우 크기를 변경한다 (SIGWINCH 유발).
    fn set_window_size(&mut self, rows: u16, cols: u16) -> io::Result<()>;

    /// shell 프로세스가 살아있는지 확인한다.
    fn is_alive(&self) -> bool;
}
