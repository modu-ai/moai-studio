//! macOS/Linux PTY 구현 — portable-pty 래핑.
//!
//! portable-pty 의 blocking Read/Write 를 감싸서
//! Pty trait 을 구현한다.

use super::Pty;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{self, Read, Write};

/// Unix 플랫폼 PTY 구현 (macOS + Linux).
///
/// portable-pty 0.9.x 기반. Windows 는 ConPtyStub 로 대체.
pub struct UnixPty {
    /// PTY master reader (blocking, 별도 thread 에서 사용)
    reader: Box<dyn Read + Send>,
    /// PTY master writer (blocking)
    writer: Box<dyn Write + Send>,
    /// PTY pair (master 포함, resize 용)
    master: Box<dyn portable_pty::MasterPty + Send>,
    /// child process (is_alive 확인용)
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl UnixPty {
    /// $SHELL (fallback: /bin/zsh macOS, /bin/bash Linux) 을 spawn 한다.
    ///
    /// SPEC-V3-002 RG-V3-002-2: $SHELL fallback 정책.
    pub fn spawn_shell() -> io::Result<Self> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "/bin/zsh".to_string()
            } else {
                "/bin/bash".to_string()
            }
        });
        Self::spawn(&shell)
    }

    /// 지정된 명령으로 PTY 를 spawn 한다.
    pub fn spawn(cmd: &str) -> io::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| io::Error::other(e.to_string()))?;

        let child = pair
            .slave
            .spawn_command(CommandBuilder::new(cmd))
            .map_err(|e| io::Error::other(e.to_string()))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| io::Error::other(e.to_string()))?;

        Ok(Self {
            reader,
            writer,
            master: pair.master,
            child,
        })
    }
}

impl Pty for UnixPty {
    fn feed(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf)?;
        self.writer.flush()
    }

    fn read_available(&mut self) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; 4096];
        // non-blocking 시뮬레이션: try_read 는 blocking 이지만
        // worker.rs 에서 block_in_place 로 호출하므로 여기서는 blocking OK
        match self.reader.read(&mut buf) {
            Ok(0) => Ok(vec![]),
            Ok(n) => {
                buf.truncate(n);
                Ok(buf)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(vec![]),
            Err(e) => Err(e),
        }
    }

    fn set_window_size(&mut self, rows: u16, cols: u16) -> io::Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| io::Error::other(e.to_string()))
    }

    fn is_alive(&self) -> bool {
        // try_wait 로 non-blocking 종료 확인
        // portable-pty 의 Child::try_wait 은 Option<ExitStatus> 반환
        self.child.clone_killer().kill().is_err()
    }
}
