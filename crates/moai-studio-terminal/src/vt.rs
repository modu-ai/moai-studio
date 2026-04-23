//! 고수준 VT 인터페이스 — libghostty_ffi 를 감싸는 Rust-friendly API.
//!
//! @MX:ANCHOR(render-state-contract)
//! @MX:REASON: GPUI render thread 가 소비하는 RenderState 계약 경계.
//!   fan_in ≥ 3: TerminalSurface::render, worker::PtyWorker, libghostty_api_compat 테스트.

use crate::libghostty_ffi::{self, FfiError, RenderSnapshot, TerminalHandle};

/// VT 상태를 관리하는 고수준 터미널 객체.
///
/// 내부적으로 libghostty_ffi::TerminalHandle 을 소유하며,
/// PTY worker 단일 스레드에서만 접근해야 한다 (!Send 제약 → worker.rs 참고).
pub struct VtTerminal {
    handle: TerminalHandle,
    cols: u16,
    rows: u16,
}

impl VtTerminal {
    /// 새 VtTerminal 을 생성한다.
    pub fn new(cols: u16, rows: u16) -> Result<Self, FfiError> {
        let handle = libghostty_ffi::new_terminal(cols, rows)?;
        Ok(Self { handle, cols, rows })
    }

    /// PTY 출력 바이트를 VT parser 에 주입한다.
    pub fn feed(&mut self, data: &[u8]) {
        libghostty_ffi::feed(&mut self.handle, data);
    }

    /// 현재 렌더 상태 스냅샷을 반환한다.
    pub fn render_state(&self) -> RenderSnapshot {
        libghostty_ffi::render_state(&self.handle)
    }

    /// 터미널 크기를 조정한다.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), FfiError> {
        self.cols = cols;
        self.rows = rows;
        libghostty_ffi::resize(&mut self.handle, cols, rows)
    }

    /// 현재 열 수
    pub fn cols(&self) -> u16 {
        self.cols
    }

    /// 현재 행 수
    pub fn rows(&self) -> u16 {
        self.rows
    }
}
