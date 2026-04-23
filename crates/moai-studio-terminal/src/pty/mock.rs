//! @MX:NOTE(mock-pty-test-only)
//! MockPty — 테스트 전용 PTY 구현 (#[cfg(test)] 로 격리).
//!
//! 스크립트된 응답 목록을 순서대로 반환하여
//! Pty trait 계약을 격리된 환경에서 검증한다 (AC-T-8(a)).

use super::Pty;
use std::io;

/// 테스트용 MockPty — 스크립트된 응답을 반환.
pub struct MockPty {
    /// 순서대로 반환할 응답 목록
    responses: Vec<Vec<u8>>,
    /// 현재 응답 인덱스
    idx: usize,
    /// is_alive 상태
    alive: bool,
    /// 마지막 set_window_size 값 (rows, cols)
    pub last_size: Option<(u16, u16)>,
}

impl MockPty {
    /// 스크립트된 응답 목록으로 MockPty 를 생성한다.
    pub fn new(responses: Vec<Vec<u8>>) -> Self {
        Self {
            responses,
            idx: 0,
            alive: true,
            last_size: None,
        }
    }

    /// PTY 를 강제 종료한다 (is_alive → false).
    pub fn close(&mut self) {
        self.alive = false;
    }
}

impl Pty for MockPty {
    fn feed(&mut self, _buf: &[u8]) -> io::Result<()> {
        // 테스트 목적으로 쓰기를 무시
        Ok(())
    }

    fn read_available(&mut self) -> io::Result<Vec<u8>> {
        if self.idx < self.responses.len() {
            let data = self.responses[self.idx].clone();
            self.idx += 1;
            Ok(data)
        } else {
            Ok(vec![])
        }
    }

    fn set_window_size(&mut self, rows: u16, cols: u16) -> io::Result<()> {
        self.last_size = Some((rows, cols));
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive
    }
}
