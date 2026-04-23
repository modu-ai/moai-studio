//! Pty trait 계약 테스트 (AC-T-8(a)(d))
//!
//! MockPty 를 사용해 Pty trait 계약을 검증한다.
//! libghostty/Zig 없이 실행 가능.

use moai_studio_terminal::pty::{MockPty, Pty};

/// MockPty: feed → read_available 라운드트립
///
/// AC-T-8(a): MockPty fd clone+script
#[test]
fn mock_feed_read_roundtrip() {
    let mut pty = MockPty::new(vec![b"hello world\n".to_vec()]);
    // MockPty 는 스크립트된 응답을 읽어온다
    let data = pty.read_available().expect("read_available 실패");
    assert_eq!(data, b"hello world\n", "스크립트 응답과 일치해야 함");
}

/// MockPty: set_window_size 후 is_alive = true
///
/// AC-T-8(d): Pty trait contract
#[test]
fn set_window_size_propagates() {
    let mut pty = MockPty::new(vec![]);
    pty.set_window_size(30, 120).expect("set_window_size 실패");
    assert!(pty.is_alive(), "set_window_size 후에도 alive 여야 함");
}

/// MockPty: close 후 is_alive = false
///
/// AC-T-8(d): Pty trait contract
#[test]
fn is_alive_after_exit_returns_false() {
    let mut pty = MockPty::new(vec![]);
    pty.close();
    assert!(!pty.is_alive(), "close 후 is_alive 가 false 여야 함");
}

/// Pty trait이 dyn-safe 해야 함 (Box<dyn Pty> 로 사용 가능)
#[test]
fn pty_trait_is_dyn_safe() {
    let pty: Box<dyn Pty> = Box::new(MockPty::new(vec![]));
    assert!(pty.is_alive());
}
