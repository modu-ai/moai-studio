//! PTY FD 정리 테스트 (AC-T-5)
//!
//! shell crash/exit 시 1초 이내 PTY FD 정리 검증.
//! Unix 전용 — #[cfg(unix)]

#[cfg(unix)]
mod unix_fd_cleanup {
    use moai_studio_terminal::pty::{Pty, UnixPty};
    use std::time::{Duration, Instant};

    /// shell exit 시 1초 이내 is_alive() = false
    ///
    /// AC-T-5: PTY FD 정리 검증
    #[test]
    fn shell_exit_cleanup_within_one_second() {
        // 실제 PTY spawn (Zig 불필요 — portable-pty 만 사용)
        let mut pty = UnixPty::spawn_shell().expect("shell spawn 실패");

        // 즉시 종료 명령 전송
        pty.feed(b"exit\n").expect("feed 실패");

        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if !pty.is_alive() {
                return; // 정상 종료
            }
            if Instant::now() >= deadline {
                panic!("1초 내에 PTY 가 종료되지 않음 (AC-T-5 실패)");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
